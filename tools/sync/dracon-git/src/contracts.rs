use std::path::Path;

/// A single commit record from `git log` output.
#[derive(Debug, Clone)]
pub struct GitCommitRecord {
    /// Commit hash (short or full).
    pub hash: String,
    /// Author name and email.
    pub author: String,
    /// Commit date in ISO format.
    pub date: String,
    /// Commit subject line.
    pub message: String,
    /// Ref decorations (tags, branches).
    pub decorations: String,
    /// Number of files changed in this commit.
    pub files_changed: usize,
    /// Lines inserted.
    pub insertions: usize,
    /// Lines deleted.
    pub deletions: usize,
}

/// A pending (unstaged or staged) change record.
#[derive(Debug, Clone)]
pub struct GitPendingRecord {
    /// Status code (e.g., "M", "A", "D").
    pub status: String,
    /// File path relative to repo root.
    pub path: String,
    /// Lines inserted.
    pub insertions: usize,
    /// Lines deleted.
    pub deletions: usize,
}

/// A complete snapshot of a repository's current state.
#[derive(Debug, Clone)]
pub struct GitRepoSnapshot {
    /// Recent commit history.
    pub history: Vec<GitCommitRecord>,
    /// Pending (uncommitted) changes.
    pub pending: Vec<GitPendingRecord>,
    /// Current branch name.
    pub branch: String,
    /// Commits ahead of tracking branch.
    pub ahead: usize,
    /// Commits behind tracking branch.
    pub behind: usize,
    /// One-line status summary.
    pub summary: String,
    /// List of remote names.
    pub remotes: Vec<String>,
    /// List of stash entries.
    pub stashes: Vec<String>,
}

/// Contract for fetching a repository snapshot.
pub trait GitSnapshotContract {
    /// Fetch a full status snapshot for the repository at `path`.
    fn fetch_snapshot(&self, path: &Path) -> std::io::Result<Option<GitRepoSnapshot>>;
}

/// Contract for previewing git content (diffs, patches).
pub trait GitPreviewContract {
    /// Show the patch for a specific commit.
    fn show_commit_patch(&self, repo_path: &Path, hash: &str) -> std::io::Result<String>;
    /// Show the diff for a specific file.
    fn show_file_diff(&self, repo_path: &Path, file_path: &str) -> std::io::Result<String>;
}
