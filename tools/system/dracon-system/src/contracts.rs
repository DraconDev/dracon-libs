use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Snapshot of a single disk device or mount point.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskSnapshot {
    /// Mount point or display name of the disk.
    pub name: String,
    /// Block device path (e.g. `/dev/sda1`).
    pub device: String,
    /// Used space in bytes.
    pub used_space: f64,
    /// Available space in bytes.
    pub available_space: f64,
    /// Total space in bytes.
    pub total_space: f64,
    /// Whether the disk is currently mounted.
    pub is_mounted: bool,
}

/// Snapshot of a single running process.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    /// Process identifier.
    pub pid: u32,
    /// Process name.
    pub name: String,
    /// CPU usage as a percentage.
    pub cpu: f32,
    /// Memory usage in MiB.
    pub mem: f32,
    /// Owning user name.
    pub user: String,
    /// Process status string (e.g. "Sleep", "Run").
    pub status: String,
}

/// Full system snapshot aggregating CPU, memory, disk, network, and process data.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Global CPU usage percentage.
    pub cpu_usage: f32,
    /// Per-core CPU usage percentages.
    pub cpu_cores: Vec<f32>,
    /// Used memory in GiB.
    pub mem_usage: f64,
    /// Total memory in GiB.
    pub total_mem: f64,
    /// Used swap in GiB.
    pub swap_usage: f64,
    /// Total swap in GiB.
    pub total_swap: f64,
    /// Disk snapshots for all detected mounts.
    pub disks: Vec<DiskSnapshot>,
    /// Top processes sorted by CPU usage.
    pub processes: Vec<ProcessSnapshot>,
    /// Total bytes received across all network interfaces.
    pub net_in: u64,
    /// Total bytes transmitted across all network interfaces.
    pub net_out: u64,
    /// System uptime in seconds.
    pub uptime: u64,
    /// Operating system name.
    pub os_name: String,
    /// Operating system version.
    pub os_version: String,
    /// Kernel version string.
    pub kernel_version: String,
    /// Hostname of the machine.
    pub hostname: String,
}

/// Contract for capturing a full system snapshot.
pub trait SystemSnapshotContract {
    /// Captures and returns a fresh system snapshot.
    fn capture_snapshot(&mut self) -> std::io::Result<SystemSnapshot>;
}

/// Contract for sending signals to processes.
pub trait ProcessControlContract {
    /// Sends a signal to the process with the given PID.
    ///
    /// If `signal` is `None`, SIGKILL (9) is used.
    /// Fails if the process is owned by a different user.
    fn kill_process(&self, pid: u32, signal: Option<i32>) -> std::io::Result<()>;
}

/// Bookmark for a remote SSH connection target.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteBookmark {
    /// Human-readable bookmark name.
    pub name: String,
    /// Hostname or IP address of the remote host.
    pub host: String,
    /// Username for the SSH connection.
    pub user: String,
    /// SSH port number.
    pub port: u16,
    /// Optional path to a private key file for authentication.
    pub key_path: Option<PathBuf>,
}

/// Request to establish a remote connection using a bookmark.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteConnectRequest {
    /// The bookmark describing the remote target.
    pub bookmark: RemoteBookmark,
    /// Connection timeout in milliseconds (minimum 500ms enforced).
    pub timeout_ms: u64,
}

/// Established remote connection metadata.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteConnection {
    /// Bookmark name of the connection.
    pub name: String,
    /// Remote hostname or IP address.
    pub host: String,
    /// Username used for the connection.
    pub user: String,
    /// SSH port number.
    pub port: u16,
    /// Private key path used, if any.
    pub key_path: Option<PathBuf>,
    /// Authentication method that succeeded (e.g. "ssh-agent", "key:…").
    pub auth_method: String,
}

/// Contract for establishing a remote SSH connection.
pub trait RemoteConnectContract {
    /// Connects to the remote host described in the request.
    fn connect(&self, request: &RemoteConnectRequest) -> anyhow::Result<RemoteConnection>;
}

/// Metadata for a single remote filesystem entry.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteEntryMetadata {
    /// File size in bytes.
    pub size: u64,
    /// Last modification time.
    pub modified: SystemTime,
    /// Creation time (best-effort; uses atime on SSH2).
    pub created: SystemTime,
    /// Unix permission bits.
    pub permissions: u32,
    /// Whether the entry is a directory.
    pub is_dir: bool,
}

/// Contract for remote filesystem operations over SFTP.
pub trait RemoteFsContract {
    /// Lists directory entries with their metadata.
    fn read_dir_with_metadata(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<(Vec<PathBuf>, HashMap<PathBuf, RemoteEntryMetadata>)>;

    /// Reads a remote file's entire contents as a UTF-8 string.
    fn read_to_string(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<String>;

    /// Writes a string to a remote file, creating or truncating it.
    fn write_string(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
        content: &str,
    ) -> std::io::Result<()>;

    /// Creates an empty remote file at the given path.
    fn create_file(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<()>;

    /// Recursively creates directories for the given path.
    fn create_dir_all(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<()>;

    /// Renames a remote file or directory, overwriting the destination if it exists.
    fn rename(
        &self,
        connection: &RemoteConnection,
        old: &std::path::Path,
        new: &std::path::Path,
    ) -> std::io::Result<()>;

    /// Removes a remote file or directory recursively.
    fn remove_path(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<()>;

    /// Copies a remote file or directory tree recursively.
    fn copy_recursive(
        &self,
        connection: &RemoteConnection,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> std::io::Result<()>;

    /// Checks whether the given remote path is a directory.
    fn is_dir(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<bool>;
}

/// Contract for remote command execution over SSH.
pub trait RemoteExecContract {
    /// Executes a raw shell command on the remote host.
    #[cfg(feature = "unsafe-remote-shell")]
    #[deprecated(note = "Use exec_program() instead to avoid shell injection")]
    fn run_command(&self, connection: &RemoteConnection, command: &str) -> std::io::Result<String>;

    /// Executes a program with arguments on the remote host, safely escaping each argument.
    fn exec_program(
        &self,
        connection: &RemoteConnection,
        program: &str,
        args: &[&str],
    ) -> std::io::Result<String>;
}
