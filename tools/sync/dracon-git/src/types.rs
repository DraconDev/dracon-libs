use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Status summary for a git repository.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoStatus {
    /// Current branch name.
    pub branch: String,
    /// Number of commits ahead of the tracking branch.
    pub ahead: usize,
    /// Number of commits behind the tracking branch.
    pub behind: usize,
    /// Number of modified (unstaged) files.
    pub modified_files: usize,
    /// Number of staged files.
    pub staged_files: usize,
    /// Whether the working tree is clean.
    pub is_clean: bool,
    /// Subject line of the most recent commit.
    pub last_commit_msg: Option<String>,
    /// Short hash of the most recent commit.
    pub last_commit_hash: Option<String>,
}

impl RepoStatus {
    /// Create a new default `RepoStatus`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Status of a file in a git diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    /// File was modified.
    Modified,
    /// File was added.
    Added,
    /// File was deleted.
    Deleted,
    /// File was renamed.
    Renamed,
    /// File type changed (e.g., symlink to regular file).
    TypeChange,
    /// Unknown or unparseable status.
    Unknown,
}

/// A single file entry in a diff, with its path and change status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    /// Path of the changed file (relative to repo root).
    pub path: PathBuf,
    /// Change status of the file.
    pub status: FileStatus,
}
