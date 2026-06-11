use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

/// Metadata captured for a file entry during search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    /// File size in bytes.
    pub size: u64,
    /// Last modification time.
    pub modified: SystemTime,
    /// Creation time when available.
    pub created: SystemTime,
    /// Platform permission bits, or a portable fallback on non-Unix platforms.
    pub permissions: u32,
    /// Whether the entry is a directory.
    pub is_dir: bool,
}

/// Coarse file category inferred from extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    /// Archive or compressed file.
    Archive,
    /// Image file.
    Image,
    /// Script or source-code file.
    Script,
    /// Plain text or configuration file.
    Text,
    /// Document file.
    Document,
    /// Audio file.
    Audio,
    /// Video file.
    Video,
    /// Unclassified file.
    Other,
}

/// Suitability result used to decide whether a file should be read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    /// Whether a binary marker was detected.
    pub is_binary: bool,
    /// Whether the file exceeds the configured byte limit.
    pub is_too_large: bool,
    /// File size in mebibytes.
    pub size_mb: u64,
}

/// Contract for classifying files.
pub trait FileInspectContract {
    /// Return the inferred category for `path`.
    fn get_file_category(&self, path: &Path) -> FileCategory;
}

/// Contract for recursive file search.
pub trait FileSearchContract {
    /// Recursively search `root` for file names containing `query`.
    fn global_search(
        &self,
        root: &Path,
        query: &str,
    ) -> std::io::Result<(
        Vec<std::path::PathBuf>,
        HashMap<std::path::PathBuf, EntryMetadata>,
    )>;
}

/// Contract for recursive copy operations.
pub trait FileCopyContract {
    /// Recursively copy `src` to `dst`.
    fn copy_recursive(&self, src: &Path, dst: &Path) -> std::io::Result<()>;
}

/// Contract for determining whether a file is suitable for reading.
pub trait FileSuitabilityContract {
    /// Check whether `path` is binary or exceeds `max_bytes`.
    fn check_file_suitability(&self, path: &Path, max_bytes: u64) -> FileSearchResult;
}
