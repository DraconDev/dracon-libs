use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub created: SystemTime,
    pub permissions: u32,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    Archive,
    Image,
    Script,
    Text,
    Document,
    Audio,
    Video,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub is_binary: bool,
    pub is_too_large: bool,
    pub size_mb: u64,
}

pub trait FileInspectContract {
    fn get_file_category(&self, path: &Path) -> FileCategory;
}

pub trait FileSearchContract {
    fn global_search(
        &self,
        root: &Path,
        query: &str,
    ) -> std::io::Result<(Vec<std::path::PathBuf>, HashMap<std::path::PathBuf, EntryMetadata>)>;
}

pub trait FileCopyContract {
    fn copy_recursive(&self, src: &Path, dst: &Path) -> std::io::Result<()>;
}

pub trait FileSuitabilityContract {
    fn check_file_suitability(&self, path: &Path, max_bytes: u64) -> FileSearchResult;
}
