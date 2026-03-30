use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitCommitRecord {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub decorations: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone)]
pub struct GitPendingRecord {
    pub status: String,
    pub path: String,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone)]
pub struct GitRepoSnapshot {
    pub history: Vec<GitCommitRecord>,
    pub pending: Vec<GitPendingRecord>,
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub summary: String,
    pub remotes: Vec<String>,
    pub stashes: Vec<String>,
}

pub trait GitSnapshotContract {
    fn fetch_snapshot(&self, path: &Path) -> std::io::Result<Option<GitRepoSnapshot>>;
}

pub trait GitPreviewContract {
    fn show_commit_patch(&self, repo_path: &Path, hash: &str) -> std::io::Result<String>;
    fn show_file_diff(&self, repo_path: &Path, file_path: &str) -> std::io::Result<String>;
}
