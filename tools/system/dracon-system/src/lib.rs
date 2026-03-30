use crate::notification::NotificationConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

pub mod contracts;
pub mod monitor;
pub mod notification;
pub mod remote;
pub mod storage;
pub use contracts::{
    DiskSnapshot, ProcessControlContract, ProcessSnapshot, RemoteBookmark, RemoteConnectContract,
    RemoteConnectRequest, RemoteConnection, RemoteEntryMetadata, RemoteExecContract,
    RemoteFsContract, SystemSnapshot, SystemSnapshotContract,
};
pub use monitor::{ProcessController, SystemSnapshotProvider};
pub use remote::{SshRemoteConnector, SshRemoteExecProvider, SshRemoteFsProvider};
pub use storage::{DirUsage, HotspotUsage, WorkspaceStorageReport, analyze_workspace_storage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppNotification {
    TaskComplete(String),
    ManifestationTriggered(String),
    SecurityAlert(String),
    Sync(String),
    Error(String),
}

#[derive(Clone)]
pub struct SystemAgent {
    home_nix_path: Option<PathBuf>,
    notification_config: NotificationConfig,
}

impl SystemAgent {
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
        }
    }

    pub fn with_notification_config(mut self, config: NotificationConfig) -> Self {
        self.notification_config = config;
        self
    }

    pub async fn send_notification(
        &self,
        notification: AppNotification,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn get_system_info(
        &self,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut info = String::new();
        if let Ok(os) = tokio::fs::read_to_string("/etc/os-release").await
            && let Some(name) = os.lines().find(|l| l.starts_with("PRETTY_NAME="))
        {
            info.push_str(&format!(
                "OS: {}\n",
                name.replace("PRETTY_NAME=", "").replace('"', "")
            ));
        }
        Ok(info)
    }

    pub async fn run_command(
        &self,
        command: &str,
        args: &[String],
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new(command).args(args).output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn list_processes(
        &self,
        filter: Option<String>,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let _ = filter;
        tokio::task::spawn_blocking(
            move || -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
            },
        )
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
    }

    pub async fn read_config(
        &self,
        name: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if name == "home.nix" {
            let path = self.home_nix_path.as_ref().ok_or("home.nix not found")?;
            Ok(tokio::fs::read_to_string(path).await?)
        } else {
            Err("Unknown config name".into())
        }
    }

    pub async fn write_config(
        &self,
        name: &str,
        content: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if name == "home.nix" {
            let path = self.home_nix_path.as_ref().ok_or("home.nix not found")?;
            tokio::fs::write(path, content).await?;
            Ok(())
        } else {
            Err("Unknown config name".into())
        }
    }

    pub async fn apply_config(
        &self,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("home-manager").arg("switch").output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn install_package(
        &self,
        name: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
