use anyhow::Result;
use dracon_system::analyze_workspace_storage;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut idx = 0usize;
    while value >= 1024.0 && idx < UNITS.len() - 1 {
        value /= 1024.0;
        idx += 1;
    }
    format!("{value:.1} {}", UNITS[idx])
}

#[derive(Debug, Clone)]
struct CleanupConfig {
    apply: bool,
    allow_tracked: bool,
    min_size_mb: u64,
    kinds: HashSet<String>,
}

fn default_cleanup_kinds() -> HashSet<String> {
    ["rust-build", "node-deps", "build-output", "cache"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn parse_kinds(csv: &str) -> HashSet<String> {
    csv.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut json = false;
    let mut cleanup = false;
    let mut apply = false;
    let mut allow_tracked = false;
    let mut min_size_mb = 512u64;
    let mut kinds = default_cleanup_kinds();
    let mut root = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home"))
        .join("Dev");

    for arg in args.iter().skip(1) {
        if arg == "--json" {
            json = true;
        } else if arg == "--cleanup" {
            cleanup = true;
        } else if arg == "--apply" {
            apply = true;
        } else if arg == "--allow-tracked" {
            allow_tracked = true;
        } else if let Some(v) = arg.strip_prefix("--min-size-mb=") {
            min_size_mb = v.parse().unwrap_or(512);
        } else if let Some(v) = arg.strip_prefix("--kinds=") {
            kinds = parse_kinds(v);
        } else {
            root = PathBuf::from(arg);
        }
    }

    let report = analyze_workspace_storage(&root, 15, 25).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Workspace: {}", report.root.display());
    println!();
    println!("Top projects:");
    for item in &report.top_projects {
        println!("  {:>10}  {}", human_bytes(item.bytes), item.path.display());
    }

    println!();
    println!("Top hotspots:");
    for item in &report.top_hotspots {
        println!(
            "  {:>10}  {:<12} {}",
            human_bytes(item.bytes),
            item.kind,
            item.path.display()
        );
    }

    if cleanup {
        let cfg = CleanupConfig {
            apply,
            allow_tracked,
            min_size_mb,
            kinds,
        };
        let threshold = cfg.min_size_mb.saturating_mul(1024 * 1024);
        let selected: Vec<_> = report
            .top_hotspots
            .iter()
            .filter(|h| cfg.kinds.contains(&h.kind) && h.bytes >= threshold)
            .collect();

        println!();
        println!(
            "Cleanup mode: {}",
            if cfg.apply { "APPLY" } else { "DRY-RUN" }
        );
        println!("Kinds: {}", {
            let mut v: Vec<_> = cfg.kinds.iter().cloned().collect();
            v.sort();
            v.join(",")
        });
        println!("Min size: {} MiB", cfg.min_size_mb);
        println!("Allow tracked: {}", cfg.allow_tracked);
        println!("Selected paths: {}", selected.len());

        let mut total = 0u64;
        let mut actionable = Vec::new();
        for item in selected {
            let tracked = is_git_tracked_dir(&item.path).await.unwrap_or(false);
            if tracked && !cfg.allow_tracked {
                println!(
                    "  {:>10}  {:<12} {}  [SKIP tracked]",
                    human_bytes(item.bytes),
                    item.kind,
                    item.path.display()
                );
                continue;
            }
            total += item.bytes;
            println!(
                "  {:>10}  {:<12} {}{}",
                human_bytes(item.bytes),
                item.kind,
                item.path.display(),
                if tracked { "  [tracked]" } else { "" }
            );
            actionable.push(item);
        }
        println!("Estimated reclaimed: {}", human_bytes(total));

        if cfg.apply {
            for item in actionable {
                if item.path.exists() {
                    println!("Deleting {}", item.path.display());
                    tokio::fs::remove_dir_all(&item.path).await?;
                }
            }
        } else {
            println!("No changes made. Re-run with --apply to execute cleanup.");
        }
    }

    Ok(())
}

async fn is_git_tracked_dir(path: &Path) -> Result<bool> {
    let parent = match path.parent() {
        Some(p) => p,
        None => return Ok(false),
    };
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy().to_string(),
        None => return Ok(false),
    };

    let top_out = tokio::process::Command::new("git")
        .arg("-C")
        .arg(parent)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .await?;
    if !top_out.status.success() {
        return Ok(false);
    }
    let repo_root = String::from_utf8_lossy(&top_out.stdout).trim().to_string();
    if repo_root.is_empty() {
        return Ok(false);
    }

    let ls_out = tokio::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-files", "--", &name])
        .output()
        .await?;
    if !ls_out.status.success() {
        return Ok(false);
    }
    Ok(!String::from_utf8_lossy(&ls_out.stdout).trim().is_empty())
}
