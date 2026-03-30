use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct DirUsage {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotspotUsage {
    pub path: PathBuf,
    pub kind: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceStorageReport {
    pub root: PathBuf,
    pub top_projects: Vec<DirUsage>,
    pub top_hotspots: Vec<HotspotUsage>,
}

fn hotspot_kind(name: &str) -> Option<&'static str> {
    match name {
        "target" => Some("rust-build"),
        "node_modules" => Some("node-deps"),
        ".git" => Some("git-db"),
        ".cache" => Some("cache"),
        "dist" => Some("build-output"),
        "build" => Some("build-output"),
        _ => None,
    }
}

async fn du_bytes(path: &Path) -> Result<u64> {
    let output = Command::new("du")
        .arg("-xsb")
        .arg(path.as_os_str())
        .output()
        .await
        .with_context(|| format!("failed to run du for {}", path.display()))?;

    if !output.status.success() {
        return Err(anyhow!(
            "du failed for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let bytes = stdout
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("unexpected du output for {}", path.display()))?
        .parse::<u64>()
        .with_context(|| format!("failed to parse du output for {}", path.display()))?;

    Ok(bytes)
}

pub async fn analyze_workspace_storage(
    root: &Path,
    top_n_projects: usize,
    top_n_hotspots: usize,
) -> Result<WorkspaceStorageReport> {
    if !root.exists() {
        return Err(anyhow!("root does not exist: {}", root.display()));
    }

    let mut project_usages = Vec::new();
    let mut dir_entries = tokio::fs::read_dir(root)
        .await
        .with_context(|| format!("failed to read {}", root.display()))?;

    while let Some(entry) = dir_entries.next_entry().await? {
        let path = entry.path();
        if !entry.file_type().await?.is_dir() {
            continue;
        }
        match du_bytes(&path).await {
            Ok(bytes) => project_usages.push(DirUsage { path, bytes }),
            Err(_) => continue,
        }
    }

    project_usages.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    project_usages.truncate(top_n_projects);

    let mut hotspot_paths = Vec::new();
    let mut seen = HashSet::new();
    let mut walker = WalkDir::new(root)
        .max_depth(6)
        .follow_links(false)
        .into_iter();
    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy();
        if hotspot_kind(&name).is_some() {
            let p = entry.path().to_path_buf();
            if seen.insert(p.clone()) {
                hotspot_paths.push(p);
            }
            walker.skip_current_dir();
        }
    }

    let mut hotspot_usages = Vec::new();
    for path in hotspot_paths {
        let kind = path
            .file_name()
            .and_then(OsStr::to_str)
            .and_then(hotspot_kind)
            .unwrap_or("unknown")
            .to_string();
        match du_bytes(&path).await {
            Ok(bytes) => hotspot_usages.push(HotspotUsage { path, kind, bytes }),
            Err(_) => continue,
        }
    }

    hotspot_usages.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    hotspot_usages.truncate(top_n_hotspots);

    Ok(WorkspaceStorageReport {
        root: root.to_path_buf(),
        top_projects: project_usages,
        top_hotspots: hotspot_usages,
    })
}
