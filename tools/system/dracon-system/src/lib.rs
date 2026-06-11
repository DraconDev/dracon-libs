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
//! - `notify` — enables desktop notification support via `notify-rust`

use crate::notification::NotificationConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
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

/// Top-level agent for system diagnostics, configuration, and notifications.
#[derive(Clone)]
pub struct SystemAgent {
    /// Path to the user's home-manager or nixpkgs home.nix, if found.
    home_nix_path: Option<PathBuf>,
    /// Desktop notification preferences.
    notification_config: NotificationConfig,
    /// Exact local commands approved for execution by [`SystemAgent::run_command`].
    approved_commands: Arc<Mutex<HashSet<String>>>,
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
            approved_commands: Arc::new(Mutex::new(HashSet::new())),
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
    /// [`run_command`](Self::run_command). This is intentionally narrow so callers
    /// cannot approve a broad shell prefix and then append unreviewed arguments.
    pub fn approve_command(&self, command: &str, args: &[String]) -> anyhow::Result<()> {
        if command.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid command: program must not be empty"
            ));
        }

        self.approved_commands
            .lock()
            .map_err(|_| anyhow::anyhow!("command approval lock poisoned"))?
            .insert(command_key(command, args));
        Ok(())
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
    /// methods like [`install_package()`] or SSH-based remote execution when a
    /// narrower API exists.
    pub unsafe fn run_command(&self, command: &str, args: &[String]) -> anyhow::Result<String> {
        self.run_command_checked(command, args)
    }

    fn run_command_checked(&self, command: &str, args: &[String]) -> anyhow::Result<String> {
        if !self
            .approved_commands
            .lock()
            .map_err(|_| anyhow::anyhow!("command approval lock poisoned"))?
            .contains(&command_key(command, args))
        {
            return Err(anyhow::anyhow!(
                "Command was not approved: {command} {}",
                args.join(" ")
            ));
        }

        let output = Command::new(command).args(args).output()?;
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
    pub async fn apply_config(&self) -> anyhow::Result<String> {
        let output = Command::new("home-manager").arg("switch").output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Install a package via nix profile.
    ///
    /// The package name is sanitized: only alphanumeric characters and hyphens
    /// are allowed, with a maximum length of 100 characters.
    pub async fn install_package(&self, name: &str) -> anyhow::Result<String> {
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
        let output = Command::new("nix")
            .args(["profile", "install", &format!("nixpkgs#{}", name)])
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_command_requires_prior_approval() {
        let agent = SystemAgent::new();
        let result = unsafe { agent.run_command("printf", &vec!["ok".to_string()]) }.await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_command_executes_only_approved_exact_pair() {
        let agent = SystemAgent::new();
        agent
            .approve_command("printf", &vec!["ok".to_string()])
            .unwrap();

        let output = unsafe { agent.run_command("printf", &vec!["ok".to_string()]) }
            .await
            .unwrap();
        assert_eq!(output, "ok");

        let rejected = unsafe { agent.run_command("printf", &vec!["other".to_string()]) }.await;
        assert!(rejected.is_err());
    }

    #[test]
    fn approve_command_rejects_empty_program() {
        let agent = SystemAgent::new();
        assert!(agent.approve_command("", &[]).is_err());
    }
}
