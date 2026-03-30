use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub modified_files: usize,
    pub staged_files: usize,
    pub is_clean: bool,
    pub last_commit_msg: Option<String>,
    pub last_commit_hash: Option<String>,
}

impl RepoStatus {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    TypeChange,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: PathBuf,
    pub status: FileStatus,
}
