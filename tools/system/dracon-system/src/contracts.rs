use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskSnapshot {
    pub name: String,
    pub device: String,
    pub used_space: f64,
    pub available_space: f64,
    pub total_space: f64,
    pub is_mounted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: f32,
    pub user: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub cpu_cores: Vec<f32>,
    pub mem_usage: f64,
    pub total_mem: f64,
    pub swap_usage: f64,
    pub total_swap: f64,
    pub disks: Vec<DiskSnapshot>,
    pub processes: Vec<ProcessSnapshot>,
    pub net_in: u64,
    pub net_out: u64,
    pub uptime: u64,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
}

pub trait SystemSnapshotContract {
    fn capture_snapshot(&mut self) -> std::io::Result<SystemSnapshot>;
}

pub trait ProcessControlContract {
    fn kill_process(&self, pid: u32, signal: Option<i32>) -> std::io::Result<()>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteBookmark {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub key_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteConnectRequest {
    pub bookmark: RemoteBookmark,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteConnection {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub key_path: Option<PathBuf>,
    pub auth_method: String,
}

pub trait RemoteConnectContract {
    fn connect(&self, request: &RemoteConnectRequest) -> anyhow::Result<RemoteConnection>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteEntryMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub created: SystemTime,
    pub permissions: u32,
    pub is_dir: bool,
}

pub trait RemoteFsContract {
    fn read_dir_with_metadata(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<(Vec<PathBuf>, HashMap<PathBuf, RemoteEntryMetadata>)>;

    fn read_to_string(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<String>;

    fn write_string(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
        content: &str,
    ) -> std::io::Result<()>;

    fn create_file(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<()>;

    fn create_dir_all(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<()>;

    fn rename(
        &self,
        connection: &RemoteConnection,
        old: &std::path::Path,
        new: &std::path::Path,
    ) -> std::io::Result<()>;

    fn remove_path(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<()>;

    fn copy_recursive(
        &self,
        connection: &RemoteConnection,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> std::io::Result<()>;

    fn is_dir(
        &self,
        connection: &RemoteConnection,
        path: &std::path::Path,
    ) -> std::io::Result<bool>;
}

pub trait RemoteExecContract {
    #[deprecated(note = "Use exec_program() instead to avoid shell injection")]
    fn run_command(&self, connection: &RemoteConnection, command: &str) -> std::io::Result<String>;

    fn exec_program(
        &self,
        connection: &RemoteConnection,
        program: &str,
        args: &[&str],
    ) -> std::io::Result<String>;
}
