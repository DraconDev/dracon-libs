use crate::contracts::{
    RemoteConnectContract, RemoteConnectRequest, RemoteConnection, RemoteEntryMetadata,
    RemoteExecContract, RemoteFsContract,
};
use anyhow::{Context, anyhow};
use ssh2::RenameFlags;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct SshRemoteConnector;
pub struct SshRemoteFsProvider;
pub struct SshRemoteExecProvider;

impl RemoteConnectContract for SshRemoteConnector {
    fn connect(&self, request: &RemoteConnectRequest) -> anyhow::Result<RemoteConnection> {
        let timeout = Duration::from_millis(request.timeout_ms.max(500));
        let (session, auth_method) = connect_session(&request.bookmark, timeout)?;
        drop(session);

        Ok(RemoteConnection {
            name: request.bookmark.name.clone(),
            host: request.bookmark.host.clone(),
            user: request.bookmark.user.clone(),
            port: request.bookmark.port,
            key_path: request.bookmark.key_path.clone(),
            auth_method,
        })
    }
}

impl RemoteFsContract for SshRemoteFsProvider {
    fn read_dir_with_metadata(
        &self,
        connection: &RemoteConnection,
        path: &Path,
    ) -> io::Result<(Vec<PathBuf>, HashMap<PathBuf, RemoteEntryMetadata>)> {
        let sftp = open_sftp(connection)?;
        let mut files = Vec::new();
        let mut metadata = HashMap::new();

        let entries = sftp
            .readdir(path)
            .map_err(|e| io::Error::other(format!("remote readdir failed: {e}")))?;

        for (entry_path, stat) in entries {
            let is_dir = is_dir_from_stat(&stat);
            files.push(entry_path.clone());
            metadata.insert(
                entry_path,
                RemoteEntryMetadata {
                    size: stat.size.unwrap_or(0),
                    modified: system_time_from_unix(stat.mtime),
                    // atime (access time) as best approximation for "created"
                    // (ssh2::FileStat doesn't expose birth time)
                    created: system_time_from_unix(stat.atime),
                    permissions: stat.perm.unwrap_or(0),
                    is_dir,
                },
            );
        }

        Ok((files, metadata))
    }

    fn read_to_string(&self, connection: &RemoteConnection, path: &Path) -> io::Result<String> {
        let sftp = open_sftp(connection)?;
        let mut file = sftp
            .open(path)
            .map_err(|e| io::Error::other(format!("remote open failed: {e}")))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write_string(
        &self,
        connection: &RemoteConnection,
        path: &Path,
        content: &str,
    ) -> io::Result<()> {
        let sftp = open_sftp(connection)?;
        let mut file = sftp
            .create(path)
            .map_err(|e| io::Error::other(format!("remote create failed: {e}")))?;
        file.write_all(content.as_bytes())
    }

    fn create_file(&self, connection: &RemoteConnection, path: &Path) -> io::Result<()> {
        let sftp = open_sftp(connection)?;
        let _file = sftp
            .create(path)
            .map_err(|e| io::Error::other(format!("remote create file failed: {e}")))?;
        Ok(())
    }

    fn create_dir_all(&self, connection: &RemoteConnection, path: &Path) -> io::Result<()> {
        let sftp = open_sftp(connection)?;
        mkdir_all(&sftp, path)
    }

    fn rename(&self, connection: &RemoteConnection, old: &Path, new: &Path) -> io::Result<()> {
        let sftp = open_sftp(connection)?;
        sftp.rename(old, new, Some(RenameFlags::OVERWRITE))
            .map_err(|e| io::Error::other(format!("remote rename failed: {e}")))
    }

    fn remove_path(&self, connection: &RemoteConnection, path: &Path) -> io::Result<()> {
        let sftp = open_sftp(connection)?;
        remove_recursive(&sftp, path)
    }

    fn copy_recursive(
        &self,
        connection: &RemoteConnection,
        src: &Path,
        dst: &Path,
    ) -> io::Result<()> {
        let sftp = open_sftp(connection)?;
        copy_recursive_sftp(&sftp, src, dst)
    }

    fn is_dir(&self, connection: &RemoteConnection, path: &Path) -> io::Result<bool> {
        let sftp = open_sftp(connection)?;
        let stat = sftp
            .stat(path)
            .map_err(|e| io::Error::other(format!("remote stat failed: {e}")))?;
        Ok(is_dir_from_stat(&stat))
    }
}

impl RemoteExecContract for SshRemoteExecProvider {
    #[allow(deprecated)]
    fn run_command(&self, connection: &RemoteConnection, command: &str) -> io::Result<String> {
        let bookmark = crate::contracts::RemoteBookmark {
            name: connection.name.clone(),
            host: connection.host.clone(),
            user: connection.user.clone(),
            port: connection.port,
            key_path: connection.key_path.clone(),
        };
        let timeout = Duration::from_millis(12_000);
        let (session, _) = connect_session(&bookmark, timeout)
            .map_err(|e| io::Error::other(format!("remote connect failed: {e}")))?;

        let mut channel = session
            .channel_session()
            .map_err(|e| io::Error::other(format!("open channel failed: {e}")))?;
        channel
            .exec(command)
            .map_err(|e| io::Error::other(format!("exec failed: {e}")))?;

        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;

        let mut stderr = String::new();
        if let Ok(err) = channel.stderr().read_to_string(&mut stderr)
            && err > 0
        {}

        channel.wait_close().ok();
        let status = channel.exit_status().unwrap_or(1);
        if status == 0 {
            Ok(stdout)
        } else {
            Err(io::Error::other(format!(
                "remote command failed (exit {status}): {}",
                stderr.trim()
            )))
        }
    }

    fn exec_program(
        &self,
        connection: &RemoteConnection,
        program: &str,
        args: &[&str],
    ) -> io::Result<String> {
        let bookmark = crate::contracts::RemoteBookmark {
            name: connection.name.clone(),
            host: connection.host.clone(),
            user: connection.user.clone(),
            port: connection.port,
            key_path: connection.key_path.clone(),
        };
        let timeout = Duration::from_millis(12_000);
        let (session, _) = connect_session(&bookmark, timeout)
            .map_err(|e| io::Error::other(format!("remote connect failed: {e}")))?;

        let mut channel = session
            .channel_session()
            .map_err(|e| io::Error::other(format!("open channel failed: {e}")))?;
        channel
            .exec(&format!("{} {}", program, args.iter().map(|a| shell_escape(a)).collect::<Vec<_>>().join(" ")))
            .map_err(|e| io::Error::other(format!("exec failed: {e}")))?;

        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;

        let mut stderr = String::new();
        if let Ok(err) = channel.stderr().read_to_string(&mut stderr)
            && err > 0
        {}

        channel.wait_close().ok();
        let status = channel.exit_status().unwrap_or(1);
        if status == 0 {
            Ok(stdout)
        } else {
            Err(io::Error::other(format!(
                "remote command failed (exit {status}): {}",
                stderr.trim()
            )))
        }
    }
}

fn open_sftp(connection: &RemoteConnection) -> io::Result<ssh2::Sftp> {
    let bookmark = crate::contracts::RemoteBookmark {
        name: connection.name.clone(),
        host: connection.host.clone(),
        user: connection.user.clone(),
        port: connection.port,
        key_path: connection.key_path.clone(),
    };
    let timeout = Duration::from_millis(8_000);
    let (session, _) = connect_session(&bookmark, timeout)
        .map_err(|e| io::Error::other(format!("remote connect failed: {e}")))?;
    session
        .sftp()
        .map_err(|e| io::Error::other(format!("open sftp failed: {e}")))
}

fn connect_session(
    bookmark: &crate::contracts::RemoteBookmark,
    timeout: Duration,
) -> anyhow::Result<(ssh2::Session, String)> {
    let addr = format!("{}:{}", bookmark.host, bookmark.port);
    let socket = resolve_addr(&addr)?;
    let tcp =
        TcpStream::connect_timeout(&socket, timeout).with_context(|| format!("connect {addr}"))?;
    tcp.set_read_timeout(Some(timeout)).ok();
    tcp.set_write_timeout(Some(timeout)).ok();

    let mut sess = ssh2::Session::new().context("failed to create ssh session")?;
    sess.set_tcp_stream(tcp);
    sess.set_blocking(true);
    sess.handshake().context("ssh handshake failed")?;

    if let Ok(mut agent) = sess.agent()
        && agent.connect().is_ok()
        && agent.list_identities().is_ok()
        && let Ok(identities) = agent.identities()
    {
        for identity in identities {
            if agent.userauth(&bookmark.user, &identity).is_ok() {
                return Ok((sess, "ssh-agent".to_string()));
            }
        }
    }

    if let Some(key_path) = &bookmark.key_path
        && sess
            .userauth_pubkey_file(&bookmark.user, None, key_path, None)
            .is_ok()
    {
        return Ok((sess, format!("key:{}", key_path.display())));
    }

    Err(anyhow!(
        "authentication failed for {}@{}:{}",
        bookmark.user,
        bookmark.host,
        bookmark.port
    ))
}

fn resolve_addr(addr: &str) -> anyhow::Result<SocketAddr> {
    addr.to_socket_addrs()
        .context("dns resolution failed")?
        .next()
        .ok_or_else(|| anyhow!("no socket addresses for {addr}"))
}

fn system_time_from_unix(secs: Option<u64>) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(secs.unwrap_or(0))
}

fn is_dir_from_stat(stat: &ssh2::FileStat) -> bool {
    stat.perm
        .map(|perm| (perm & libc::S_IFMT) == libc::S_IFDIR)
        .unwrap_or(false)
}

fn mkdir_all(sftp: &ssh2::Sftp, path: &Path) -> io::Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if current.as_os_str().is_empty() {
            continue;
        }
        if sftp.stat(&current).is_err() {
            sftp.mkdir(&current, 0o755)
                .map_err(|e| io::Error::other(format!("remote mkdir failed: {e}")))?;
        }
    }
    Ok(())
}

fn remove_recursive(sftp: &ssh2::Sftp, path: &Path) -> io::Result<()> {
    let stat = sftp
        .stat(path)
        .map_err(|e| io::Error::other(format!("remote stat failed: {e}")))?;
    if is_dir_from_stat(&stat) {
        let entries = sftp
            .readdir(path)
            .map_err(|e| io::Error::other(format!("remote readdir failed: {e}")))?;
        for (entry, _) in entries {
            if let Some(name) = entry.file_name().and_then(|n| n.to_str())
                && (name == "." || name == "..")
            {
                continue;
            }
            remove_recursive(sftp, &entry)?;
        }
        sftp.rmdir(path)
            .map_err(|e| io::Error::other(format!("remote rmdir failed: {e}")))?;
        Ok(())
    } else {
        sftp.unlink(path)
            .map_err(|e| io::Error::other(format!("remote unlink failed: {e}")))
    }
}

fn copy_recursive_sftp(sftp: &ssh2::Sftp, src: &Path, dst: &Path) -> io::Result<()> {
    let stat = sftp
        .stat(src)
        .map_err(|e| io::Error::other(format!("remote stat failed: {e}")))?;
    if is_dir_from_stat(&stat) {
        let _ = sftp.mkdir(dst, 0o755);
        let entries = sftp
            .readdir(src)
            .map_err(|e| io::Error::other(format!("remote readdir failed: {e}")))?;
        for (entry, _) in entries {
            if let Some(name) = entry.file_name().and_then(|n| n.to_str())
                && (name == "." || name == "..")
            {
                continue;
            }
            let target = dst.join(entry.file_name().unwrap_or_default());
            copy_recursive_sftp(sftp, &entry, &target)?;
        }
        Ok(())
    } else {
        let mut input = sftp
            .open(src)
            .map_err(|e| io::Error::other(format!("remote open failed: {e}")))?;
        let mut output = sftp
            .create(dst)
            .map_err(|e| io::Error::other(format!("remote create failed: {e}")))?;
        io::copy(&mut input, &mut output)?;
        Ok(())
    }
}

fn shell_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('\'');
    for c in s.chars() {
        if c == '\'' {
            result.push_str("'\"'\"'");
        } else {
            result.push(c);
        }
    }
    result.push('\'');
    result
}
