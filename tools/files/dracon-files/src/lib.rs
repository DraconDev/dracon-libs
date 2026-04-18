mod contracts;

pub use contracts::*;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

fn permissions_bits(meta: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode()
    }
    #[cfg(not(unix))]
    {
        if meta.permissions().readonly() {
            0o444
        } else {
            0o666
        }
    }
}

fn get_file_category_from_extension(path: &Path) -> FileCategory {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "zip" | "tar" | "gz" | "7z" | "rar" | "bz2" | "xz" | "zst" => FileCategory::Archive,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" => {
            FileCategory::Image
        }
        "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" | "py" | "rb" | "js" | "ts"
        | "c" | "cpp" | "h" | "hpp" | "go" | "java" | "swift" | "kt" => {
            FileCategory::Script
        }
        "txt" | "md" | "rst" | "log" | "json" | "xml" | "yaml" | "yml" | "toml" | "ini"
        | "cfg" | "conf" | "env" | "gitignore" | "dockerfile" | "makefile" => {
            FileCategory::Text
        }
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "tex" | "epub" => FileCategory::Document,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => FileCategory::Audio,
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => FileCategory::Video,
        _ => FileCategory::Other,
    }
}

fn is_binary_file(path: &Path) -> bool {
    if let Ok(content) = fs::read(path) {
        let sample = &content[..content.len().min(8192)];
        sample.iter().any(|&b| b == 0)
    } else {
        false
    }
}

pub struct FsCatalog;

impl FileInspectContract for FsCatalog {
    fn get_file_category(&self, path: &Path) -> FileCategory {
        get_file_category_from_extension(path)
    }
}

impl FileSearchContract for FsCatalog {
    fn global_search(
        &self,
        root: &Path,
        query: &str,
    ) -> std::io::Result<(Vec<PathBuf>, HashMap<PathBuf, EntryMetadata>)> {
        let mut files = Vec::new();
        let mut metadata = HashMap::new();
        let query_lower = query.to_lowercase();

        for entry in WalkDir::new(root)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase())
                    .unwrap_or_default();

                if name.contains(&query_lower) {
                    if let Ok(meta) = fs::metadata(&path) {
                        let entry_meta = EntryMetadata {
                            size: meta.len(),
                            modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            created: meta.created().unwrap_or(SystemTime::UNIX_EPOCH),
                            permissions: permissions_bits(&meta),
                            is_dir: meta.is_dir(),
                        };
                        metadata.insert(path.clone(), entry_meta);
                        files.push(path);
                    }
                }
            }
        }

        Ok((files, metadata))
    }
}

impl FileCopyContract for FsCatalog {
    fn copy_recursive(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
        if src.is_dir() {
            fs::create_dir_all(dst)?;
            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());
                self.copy_recursive(&src_path, &dst_path)?;
            }
        } else {
            fs::copy(src, dst)?;
        }
        Ok(())
    }
}

impl FileSuitabilityContract for FsCatalog {
    fn check_file_suitability(&self, path: &Path, max_bytes: u64) -> FileSearchResult {
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => {
                return FileSearchResult {
                    is_binary: false,
                    is_too_large: true,
                    size_mb: 0,
                }
            }
        };

        let size_mb = meta.len() / (1024 * 1024);
        let is_too_large = meta.len() > max_bytes;
        let is_binary = is_binary_file(path);

        FileSearchResult {
            is_binary,
            is_too_large,
            size_mb,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_category_detection() {
        let catalog = FsCatalog;

        assert_eq!(
            catalog.get_file_category(Path::new("test.zip")),
            FileCategory::Archive
        );
        assert_eq!(
            catalog.get_file_category(Path::new("image.png")),
            FileCategory::Image
        );
        assert_eq!(
            catalog.get_file_category(Path::new("script.py")),
            FileCategory::Script
        );
        assert_eq!(
            catalog.get_file_category(Path::new("readme.txt")),
            FileCategory::Text
        );
    }
}
