#![deny(missing_docs)]

//! Dracon System Runtime — system diagnostics, SSH remote execution, and notifications.
//!
//! ## Crates
//!
//! - [`SystemSnapshotProvider`] — snapshot-based CPU, memory, and process info
//! - [`ProcessController`] — process control and listing
//! - [`SshRemoteConnector`], [`SshRemoteExecProvider`], [`SshRemoteFsProvider`] — SSH remoting
//! - [`WorkspaceStorageReport`] — workspace disk usage analysis
//!
//! ## Example
//!
//! ```ignore
//! use dracon_system::SystemSnapshotProvider;
//! let snap = provider.get_snapshot().await?;
//! ```
//!
//! ## Feature Flags
//!
//! - `unsafe-remote-shell` — opts into the deprecated raw remote shell execution API.

use crate::notification::NotificationConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::process::Command;

/// System contracts: data types and trait definitions for snapshots, processes, and remoting.
pub mod contracts;
/// System monitor: CPU, memory, disk, and process snapshot providers.
pub mod monitor;
/// Desktop notification configuration and dispatch.
pub mod notification;
/// SSH remote execution and filesystem providers.
pub mod remote;
/// Workspace storage analysis and hotspot detection.
pub mod storage;
pub use contracts::{
    DiskSnapshot, ProcessControlContract, ProcessSnapshot, RemoteBookmark, RemoteConnectContract,
    RemoteConnectRequest, RemoteConnection, RemoteEntryMetadata, RemoteExecContract,
    RemoteFsContract, SystemSnapshot, SystemSnapshotContract,
};
pub use monitor::{ProcessController, SystemSnapshotProvider};
pub use remote::{SshRemoteConnector, SshRemoteExecProvider, SshRemoteFsProvider};
pub use storage::{analyze_workspace_storage, DirUsage, HotspotUsage, WorkspaceStorageReport};

/// Application-level notification variants dispatched through the system agent.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppNotification {
    /// A background task has completed successfully.
    TaskComplete(String),
    /// A manifestation (workspace change) has been triggered.
    ManifestationTriggered(String),
    /// A security-related alert requiring user attention.
    SecurityAlert(String),
    /// A sync operation status update.
    Sync(String),
    /// An error notification.
    Error(String),
}

/// Exact local command approved for execution by [`SystemAgent::run_command`].
#[derive(Clone, Debug, PartialEq, Eq)]
struct ApprovedCommand {
    /// Program string supplied by the caller.
    program: String,
    /// Absolute executable path resolved at approval time.
    path: PathBuf,
    /// Exact argument list approved with the command.
    args: Vec<String>,
}

/// Top-level agent for system diagnostics, configuration, and notifications.
#[non_exhaustive]
#[derive(Clone)]
pub struct SystemAgent {
    /// Path to the user's home-manager or nixpkgs home.nix, if found.
    home_nix_path: Option<PathBuf>,
    /// Desktop notification preferences.
    notification_config: NotificationConfig,
    /// Exact local commands approved for execution by [`SystemAgent::run_command`].
    approved_commands: Arc<Mutex<HashMap<String, ApprovedCommand>>>,
}

impl SystemAgent {
    /// Creates a new `SystemAgent`, auto-detecting the home.nix path.
    pub fn new() -> Self {
        let home = dirs::home_dir();
        let home_nix_path = home.as_ref().and_then(|h| {
            let paths = [
                h.join(".config/home-manager/home.nix"),
                h.join(".config/nixpkgs/home.nix"),
            ];
            paths.into_iter().find(|p| p.exists())
        });

        Self {
            home_nix_path,
            notification_config: NotificationConfig::default(),
            approved_commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Sets a custom notification configuration, consuming and returning self.
    pub fn with_notification_config(mut self, config: NotificationConfig) -> Self {
        self.notification_config = config;
        self
    }

    /// Sends a desktop notification based on the variant and notification preferences.
    pub async fn send_notification(&self, notification: AppNotification) -> anyhow::Result<()> {
        if !self.notification_config.enabled {
            return Ok(());
        }

        let (title, body, icon, should_send) = match notification {
            AppNotification::TaskComplete(task) => (
                " Dracon: Task Complete",
                task,
                "emblem-success",
                self.notification_config.on_task_complete,
            ),
            AppNotification::ManifestationTriggered(workspace) => (
                " Dracon: Manifestation",
                format!("Triggered in {}", workspace),
                "system-run",
                self.notification_config.on_manifestation,
            ),
            AppNotification::SecurityAlert(alert) => (
                " Dracon: Security Alert",
                alert,
                "security-high",
                self.notification_config.on_security_alert,
            ),
            AppNotification::Sync(msg) => (
                " Dracon: Sync",
                msg,
                "emblem-synchronizing",
                self.notification_config.on_sync,
            ),
            AppNotification::Error(err) => (
                " Dracon: Error",
                err,
                "dialog-error",
                self.notification_config.on_error,
            ),
        };

        if should_send {
            use notify_rust::Notification;
            Notification::new()
                .summary(title)
                .body(&body)
                .icon(icon)
                .appname("Dracon")
                .show()?;
        }

        Ok(())
    }

    /// Returns basic OS information parsed from `/etc/os-release`.
    pub async fn get_system_info(&self) -> anyhow::Result<String> {
        let mut info = String::new();
        if let Ok(os) = tokio::fs::read_to_string("/etc/os-release").await {
            if let Some(name) = os.lines().find(|l| l.starts_with("PRETTY_NAME=")) {
                info.push_str(&format!(
                    "OS: {}\n",
                    name.replace("PRETTY_NAME=", "").replace('"', "")
                ));
            }
        }
        Ok(info)
    }

    /// Approve one exact local command for later execution.
    ///
    /// Approval is exact: the same program and argument list must be passed to
    /// [`run_command`](Self::run_command). The executable is resolved to an
    /// absolute path at approval time and re-checked before execution so `PATH`
    /// changes cannot redirect an approved command.
    pub fn approve_command(&self, command: &str, args: &[String]) -> anyhow::Result<()> {
        if command.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid command: program must not be empty"
            ));
        }

        let path = resolve_command_path(command)?;
        let approved = ApprovedCommand {
            program: command.to_string(),
            path,
            args: args.to_vec(),
        };
        self.approved_commands
            .lock()
            .map_err(|_| anyhow::anyhow!("command approval lock poisoned"))?
            .insert(command_key(command, args), approved);
        Ok(())
    }

    /// Approve `home-manager switch` for later execution through [`Self::apply_config`].
    pub fn approve_config_apply(&self) -> anyhow::Result<()> {
        self.approve_command("home-manager", &["switch".to_string()])
    }

    /// Approve a sanitized `nix profile install nixpkgs#<name>` command.
    pub fn approve_package_install(&self, name: &str) -> anyhow::Result<()> {
        let package_ref = package_ref(name)?;
        self.approve_command(
            "nix",
            &["profile".to_string(), "install".to_string(), package_ref],
        )
    }

    /// Execute an approved exact local command with exact arguments.
    ///
    /// # Safety
    ///
    /// Callers may only invoke this after [`approve_command`](Self::approve_command)
    /// has approved the exact `(command, args)` pair. This method still returns an
    /// error if approval is missing, but the `unsafe` boundary documents that
    /// command execution is a privileged operation and must not be called with
    /// untrusted input.
    ///
    /// # Security Warning
    ///
    /// This method executes an allowlisted command without a shell. Prefer specific
    /// methods like [`Self::install_package`] or SSH-based remote execution when a
    /// narrower API exists.
    pub async unsafe fn run_command(
        &self,
        command: &str,
        args: &[String],
    ) -> anyhow::Result<String> {
        self.run_command_checked(command, args).await
    }

    async fn run_command_checked(&self, command: &str, args: &[String]) -> anyhow::Result<String> {
        let approved = {
            self.approved_commands
                .lock()
                .map_err(|_| anyhow::anyhow!("command approval lock poisoned"))?
                .get(&command_key(command, args))
                .cloned()
                .ok_or_else(|| {
                    anyhow::anyhow!("Command was not approved: {command} {}", args.join(" "))
                })?
        };

        if approved.program != command || approved.args != args {
            return Err(anyhow::anyhow!("Command approval key mismatch"));
        }

        let execution_path = match resolve_command_path(command) {
            Ok(path) => path,
            Err(_) if approved.path.exists() => approved.path.clone(),
            Err(err) => return Err(err),
        };
        if execution_path != approved.path {
            return Err(anyhow::anyhow!(
                "Approved command path changed: expected {}, resolved {}",
                approved.path.display(),
                execution_path.display()
            ));
        }

        let output = Command::new(&approved.path).args(args).output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(anyhow::anyhow!(
                "Command failed with exit code {:?}: {}",
                output.status.code(),
                stderr
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Lists running processes (top 10 by default), optionally filtered.
    pub async fn list_processes(&self, filter: Option<String>) -> anyhow::Result<String> {
        let _ = filter;
        tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};
            let mut sys = System::new_with_specifics(
                RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
            );
            sys.refresh_processes(ProcessesToUpdate::All, true);
            let mut output = String::new();
            for proc in sys.processes().values().take(10) {
                output.push_str(&format!(
                    "{} | {}\n",
                    proc.pid(),
                    proc.name().to_string_lossy()
                ));
            }
            Ok(output)
        })
        .await
        .map_err(|e| anyhow::anyhow!("spawn blocking failed: {}", e))
        .and_then(|r| r)
    }

    /// Reads a named configuration file (currently only "home.nix" is supported).
    pub async fn read_config(&self, name: &str) -> anyhow::Result<String> {
        if name == "home.nix" {
            let path = self.home_nix_path.as_ref().context("home.nix not found")?;
            Ok(tokio::fs::read_to_string(path).await?)
        } else {
            Err(anyhow::anyhow!("Unknown config name"))
        }
    }

    /// Writes content to a named configuration file (currently only "home.nix" is supported).
    pub async fn write_config(&self, name: &str, content: &str) -> anyhow::Result<()> {
        if name == "home.nix" {
            let path = self.home_nix_path.as_ref().context("home.nix not found")?;
            tokio::fs::write(path, content).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Unknown config name"))
        }
    }

    /// Applies the current home-manager configuration by running `home-manager switch`.
    ///
    /// Callers must first approve this exact command with [`Self::approve_config_apply`].
    pub async fn apply_config(&self) -> anyhow::Result<String> {
        ensure_approved(self, "home-manager", &["switch".to_string()])?;
        self.run_command_checked("home-manager", &["switch".to_string()])
            .await
    }

    /// Install a package via nix profile.
    ///
    /// The package name is sanitized: only alphanumeric characters and hyphens
    /// are allowed, with a maximum length of 100 characters. Callers must first
    /// approve the resulting command with [`Self::approve_package_install`].
    pub async fn install_package(&self, name: &str) -> anyhow::Result<String> {
        let package_ref = package_ref(name)?;
        ensure_approved(
            self,
            "nix",
            &[
                "profile".to_string(),
                "install".to_string(),
                package_ref.clone(),
            ],
        )?;
        self.run_command_checked(
            "nix",
            &["profile".to_string(), "install".to_string(), package_ref],
        )
        .await
    }
}

impl Default for SystemAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn command_key(command: &str, args: &[String]) -> String {
    let mut key = command.to_string();
    for arg in args {
        key.push('\0');
        key.push_str(arg);
    }
    key
}

fn ensure_approved(agent: &SystemAgent, command: &str, args: &[String]) -> anyhow::Result<()> {
    agent
        .approved_commands
        .lock()
        .map_err(|_| anyhow::anyhow!("command approval lock poisoned"))?
        .get(&command_key(command, args))
        .map(|_| ())
        .ok_or_else(|| anyhow::anyhow!("Command was not approved: {command} {}", args.join(" ")))
}

fn resolve_command_path(command: &str) -> anyhow::Result<PathBuf> {
    let candidate = Path::new(command);
    if candidate.components().count() > 1 {
        return candidate
            .canonicalize()
            .with_context(|| format!("failed to resolve command path: {}", candidate.display()));
    }

    let Some(paths) = env::var_os("PATH") else {
        anyhow::bail!("PATH is not set; cannot resolve command: {command}");
    };
    for dir in env::split_paths(&paths) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return candidate.canonicalize().with_context(|| {
                format!(
                    "failed to canonicalize command path: {}",
                    candidate.display()
                )
            });
        }
    }
    anyhow::bail!("command not found on PATH: {command}")
}

fn package_ref(name: &str) -> anyhow::Result<String> {
    if name.is_empty() || name.len() > 100 {
        return Err(anyhow::anyhow!(
            "Invalid package name: length must be 1-100"
        ));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(anyhow::anyhow!(
            "Invalid package name: only alphanumeric and hyphen allowed"
        ));
    }
    Ok(format!("nixpkgs#{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex as StdMutex;

    static PATH_LOCK: StdMutex<()> = StdMutex::new(());

    #[cfg(unix)]
    fn make_executable(path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn approved_command_uses_approved_path_when_path_changes() {
        let old_path = std::env::var_os("PATH");
        let temp_dir = std::env::temp_dir().join(format!(
            "dracon-system-approved-path-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let script = temp_dir.join("dracon-approved-path-test");
        std::fs::write(&script, "#!/bin/sh\nprintf ok\n").unwrap();
        make_executable(&script);
        let agent = SystemAgent::new();

        {
            let _guard = PATH_LOCK.lock().unwrap();
            std::env::set_var("PATH", &temp_dir);
            agent
                .approve_command("dracon-approved-path-test", &[])
                .unwrap();

            std::env::set_var("PATH", "");
        }

        let output = agent
            .run_command_checked("dracon-approved-path-test", &[])
            .await
            .unwrap();
        assert_eq!(output, "ok");

        match old_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn run_command_requires_prior_approval() {
        let agent = SystemAgent::new();
        let result = agent
            .run_command_checked("sh", &["-c".to_string(), "printf ok".to_string()])
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_command_executes_only_approved_exact_pair() {
        let agent = SystemAgent::new();
        agent
            .approve_command("sh", &["-c".to_string(), "printf ok".to_string()])
            .unwrap();

        let output = agent
            .run_command_checked("sh", &["-c".to_string(), "printf ok".to_string()])
            .await
            .unwrap();
        assert_eq!(output, "ok");

        let rejected = agent
            .run_command_checked("sh", &["-c".to_string(), "printf other".to_string()])
            .await;
        assert!(rejected.is_err());
    }

    #[tokio::test]
    async fn privileged_operations_require_specific_approval() {
        let agent = SystemAgent::new();
        assert!(agent.apply_config().await.is_err());
        assert!(agent.install_package("hello").await.is_err());
    }

    #[tokio::test]
    async fn package_name_sanitizer_rejects_shell_metacharacters() {
        let agent = SystemAgent::new();
        assert!(agent.install_package("hello;rm -rf /").await.is_err());
    }

    #[tokio::test]
    async fn run_command_checked_surfaces_non_zero_exit() {
        let agent = SystemAgent::new();
        agent
            .approve_command(
                "sh",
                &["-c".to_string(), "printf partial >&2; exit 7".to_string()],
            )
            .unwrap();

        let result = agent
            .run_command_checked(
                "sh",
                &["-c".to_string(), "printf partial >&2; exit 7".to_string()],
            )
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn approve_command_rejects_empty_program() {
        let agent = SystemAgent::new();
        assert!(agent.approve_command("", &[]).is_err());
    }
}
