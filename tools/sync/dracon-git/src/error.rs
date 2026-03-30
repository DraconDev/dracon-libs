use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git operation failed: {0}")]
    LibGit2(#[from] git2::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Repository not found at path: {0}")]
    NotFound(String),

    #[error("Merge conflict detected")]
    MergeConflict,

    #[error("Working directory has uncommitted changes")]
    DirtyState,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GitError>;
