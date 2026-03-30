use crate::contracts::{
    GitCommitRecord, GitPendingRecord, GitPreviewContract, GitRepoSnapshot, GitSnapshotContract,
};
use std::collections::HashMap;
use std::path::Path;

/// Create a `git` command with direnv and other env hooks blocked.
/// This prevents per-subprocess overhead from shell integrations.
fn git_cmd() -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    // Block direnv, zsh hooks, and other shell integrations from spawning
    cmd.env_remove("DIRENV_DIR")
        .env_remove("DIRENV_FILE")
        .env_remove("DIRENV_WATCHES")
        .env_remove("DIRENV_DIFF")
        .env("DIRENV_LOG_FORMAT", "")
        // Disable git hooks that might trigger external tools
        .env("GIT_HOOKS_PATH", "")
        // Disable SSH askpass popups
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("SSH_ASKPASS", "");
    cmd
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CliGitSnapshotProvider;

fn parse_git_shortstat(line: &str) -> (usize, usize, usize) {
    let mut files_changed = 0usize;
    let mut insertions = 0usize;
    let mut deletions = 0usize;

    for segment in line.split(',').map(str::trim) {
        let num = segment
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<usize>().ok());
        let Some(value) = num else {
            continue;
        };

        if segment.contains("file changed") || segment.contains("files changed") {
            files_changed = value;
        } else if segment.contains("insertion") {
            insertions = value;
        } else if segment.contains("deletion") {
            deletions = value;
        }
    }

    (files_changed, insertions, deletions)
}

fn parse_git_log_record(line: &str) -> Option<GitCommitRecord> {
    let parts: Vec<&str> = line.split('\x1f').collect();
    if parts.len() < 5 {
        return None;
    }

    Some(GitCommitRecord {
        hash: parts[0].to_string(),
        author: parts[1].to_string(),
        date: parts[2].to_string(),
        message: parts[3].to_string(),
        decorations: parts[4].to_string(),
        files_changed: 0,
        insertions: 0,
        deletions: 0,
    })
}

impl GitSnapshotContract for CliGitSnapshotProvider {
    fn fetch_snapshot(&self, path: &Path) -> std::io::Result<Option<GitRepoSnapshot>> {
        let output = git_cmd()
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let (ahead, behind) = if let Ok(out) = git_cmd()
            .args(["rev-list", "--left-right", "--count", "HEAD...@{u}"])
            .current_dir(path)
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.len() == 2 {
                    (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0))
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        let mut history = Vec::new();
        if let Ok(out) = git_cmd()
            .args([
                "--no-pager",
                "log",
                "-n",
                "100",
                "--pretty=format:%H%x1f%an%x1f%ar%x1f%s%x1f%d",
                "--shortstat",
            ])
            .current_dir(path)
            .output()
        {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let mut current_commit: Option<GitCommitRecord> = None;

            for line in out_str.lines() {
                let line = line.trim();
                if let Some(parsed) = parse_git_log_record(line) {
                    if let Some(c) = current_commit.take() {
                        history.push(c);
                    }
                    current_commit = Some(parsed);
                } else if let Some(ref mut c) = current_commit
                    && line.contains("changed")
                {
                    let (files_changed, insertions, deletions) = parse_git_shortstat(line);
                    c.files_changed = files_changed;
                    c.insertions = insertions;
                    c.deletions = deletions;
                }
            }
            if let Some(c) = current_commit {
                history.push(c);
            }
        }

        let mut pending = Vec::new();
        let mut stats_map = HashMap::new();

        if let Ok(out) = git_cmd()
            .args(["diff", "--numstat"])
            .current_dir(path)
            .output()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let ins = parts[0].parse::<usize>().unwrap_or(0);
                    let del = parts[1].parse::<usize>().unwrap_or(0);
                    let file = parts[2].to_string();
                    stats_map.insert(file, (ins, del));
                }
            }
        }
        if let Ok(out) = git_cmd()
            .args(["diff", "--staged", "--numstat"])
            .current_dir(path)
            .output()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let ins = parts[0].parse::<usize>().unwrap_or(0);
                    let del = parts[1].parse::<usize>().unwrap_or(0);
                    let file = parts[2].to_string();
                    let entry = stats_map.entry(file).or_insert((0, 0));
                    entry.0 += ins;
                    entry.1 += del;
                }
            }
        }

        if let Ok(out) = git_cmd()
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
        {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if line.len() > 3 {
                    let status = line[0..2].trim().to_string();
                    let file = line[3..].to_string();
                    let (ins, del) = stats_map.get(&file).cloned().unwrap_or((0, 0));
                    pending.push(GitPendingRecord {
                        status,
                        path: file,
                        insertions: ins,
                        deletions: del,
                    });
                }
            }
        }

        let summary = if let Ok(out) = git_cmd()
            .args(["diff", "HEAD", "--shortstat"])
            .current_dir(path)
            .output()
        {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        } else {
            String::new()
        };

        let remotes = if let Ok(out) = git_cmd().args(["remote", "-v"]).current_dir(path).output() {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        let stashes = if let Ok(out) = git_cmd().args(["stash", "list"]).current_dir(path).output()
        {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        Ok(Some(GitRepoSnapshot {
            history,
            pending,
            branch,
            ahead,
            behind,
            summary,
            remotes,
            stashes,
        }))
    }
}

impl GitPreviewContract for CliGitSnapshotProvider {
    fn show_commit_patch(&self, repo_path: &Path, hash: &str) -> std::io::Result<String> {
        let out = git_cmd()
            .args(["show", "--patch", "--stat", "--color=never", "--", hash])
            .current_dir(repo_path)
            .output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn show_file_diff(&self, repo_path: &Path, file_path: &str) -> std::io::Result<String> {
        let out = git_cmd()
            .args(["diff", "--", file_path])
            .current_dir(repo_path)
            .output()?;
        let content = String::from_utf8_lossy(&out.stdout).to_string();
        if content.is_empty() {
            Ok("(No changes or file only in index)".to_string())
        } else {
            Ok(content)
        }
    }
}
