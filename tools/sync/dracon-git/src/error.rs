use thiserror::Error;

/// Errors that can occur during git operations.
#[derive(Error, Debug)]
pub enum GitError {
    /// A libgit2 operation failed.
    #[error("Git operation failed: {0}")]
    LibGit2(#[from] git2::Error),

    /// Authentication failed (invalid credentials or SSH key).
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// No repository found at the given path.
    #[error("Repository not found at path: {0}")]
    NotFound(String),

    /// A merge conflict was detected.
    #[error("Merge conflict detected")]
    MergeConflict,

    /// The working directory has uncommitted changes.
    #[error("Working directory has uncommitted changes")]
    DirtyState,

    /// An I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// An uncategorized error.
    #[error("Unknown error: {0}")]
    Other(String),
}

/// Convenience alias for `Result<T, GitError>`.
pub type Result<T> = std::result::Result<T, GitError>;
