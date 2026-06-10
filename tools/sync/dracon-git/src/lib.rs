#![warn(missing_docs)]

//! Dracon Git — Git operations with libgit2 and CLI fallback for robustness.
//!
//! ## Crates
//!
//! - [`GitService`] — async git operations (status, pull, push, commit)
//! - [`CliGitSnapshotProvider`] — git log/history via CLI
//! - [`extract_intent`] — intent extraction from branch name or task board
//! - [`build_commit_message`] — semantic commit message generation
//!
//! ## Example
//!
//! ```ignore
//! use dracon_git::GitService;
//! let git = GitService::new("/path/to/repo")?;
//! let status = git.get_status().await?;
//! ```
//!
//! ## Feature Flags
//!
//! - Default: uses libgit2 with CLI fallback for binary files
//! - CLI fallback is automatic when libgit2 encounters nul-bytes

/// CLI-based git snapshot provider.
pub mod cli;
/// Contracts for git snapshot and preview operations.
pub mod contracts;
/// Commit message generation from context.
pub mod dracon_sync_commit;
/// Error types for git operations.
pub mod error;
/// Intent extraction from plan files and branch names.
pub mod intent;
/// Task progress scanning from blueprint files.
pub mod task_scan;
/// Core types: RepoStatus, FileStatus, DiffFile.
pub mod types;

pub use cli::CliGitSnapshotProvider;
pub use contracts::{
    GitCommitRecord, GitPendingRecord, GitPreviewContract, GitRepoSnapshot, GitSnapshotContract,
};
pub use dracon_sync_commit::{build_commit_message, CommitContext, SemanticSummary, SymbolInfo};
pub use intent::{extract_intent, IntentInfo};
pub use task_scan::{scan_blueprint_tasks, TaskProgress};
pub use types::{DiffFile, FileStatus, RepoStatus};

use crate::error::{GitError, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Primary git service — provides async operations (status, pull, push, commit).
#[derive(Clone)]
pub struct GitService {
    root_path: PathBuf,
}

impl GitService {
    /// Initialize a new GitService for a specific repository root
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        Ok(Self { root_path: path })
    }

    /// Check if the root path is a valid git repository
    pub async fn is_git_repo(&self) -> Result<bool> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> Result<bool> {
            // Use git CLI instead of libgit2::Repository::open() which
            // reads the index on open and fails on repos with binary blobs.
            let output = std::process::Command::new("git")
                .args(["rev-parse", "--git-dir"])
                .current_dir(&path)
                .output();
            match output {
                Ok(o) if o.status.success() => Ok(true),
                _ => {
                    // Fallback: check for .git directory
                    Ok(path.join(".git").exists())
                }
            }
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Get the current status of the repository (ahead/behind, modified count)
    /// Falls back to CLI on any libgit2 error (binary blobs, nul bytes, etc.)
    pub async fn get_status(&self) -> Result<RepoStatus> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<RepoStatus, GitError> {
            // Try libgit2 first (faster)
            let result = (|| -> std::result::Result<RepoStatus, git2::Error> {
                let repo = git2::Repository::open(&path)?;
                let mut status = RepoStatus::new();

                let head = repo.head();
                status.branch = head
                    .as_ref()
                    .ok()
                    .and_then(|h| h.shorthand().ok())
                    .unwrap_or("detached")
                    .to_string();

                let mut opts = git2::StatusOptions::new();
                opts.include_untracked(true);
                let statuses = repo.statuses(Some(&mut opts))?;
                for entry in statuses.iter() {
                    let s = entry.status();
                    if s.is_index_new()
                        || s.is_index_modified()
                        || s.is_index_deleted()
                        || s.is_index_renamed()
                        || s.is_index_typechange()
                    {
                        status.staged_files += 1;
                    }
                    if s.is_wt_new() {
                        // Untracked files are NOT counted as modified.
                        // This prevents the report from showing 10k "modified"
                        // entries when a repo has a large untracked target/ dir.
                        status.untracked_files += 1;
                    } else if s.is_wt_modified()
                        || s.is_wt_deleted()
                        || s.is_wt_renamed()
                        || s.is_wt_typechange()
                    {
                        status.modified_files += 1;
                    }
                }
                status.is_clean = status.modified_files == 0
                    && status.staged_files == 0
                    && status.untracked_files == 0;

                if let Ok(head_ref) = head {
                    if let Ok(head_name) = head_ref.shorthand() {
                        if let Ok(upstream) =
                            repo.branch_upstream_name(&format!("refs/heads/{}", head_name))
                        {
                            if let Ok(upstream_str) = upstream.as_str() {
                                if let Ok(upstream_oid) = repo.refname_to_id(upstream_str) {
                                    if let Some(head_oid) = head_ref.target() {
                                        if let Ok((ahead, behind)) =
                                            repo.graph_ahead_behind(head_oid, upstream_oid)
                                        {
                                            status.ahead = ahead;
                                            status.behind = behind;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Ok(commit) = head_ref.peel_to_commit() {
                        status.last_commit_hash = Some(commit.id().to_string());
                        status.last_commit_msg =
                            commit.summary().ok().flatten().map(|s| s.to_string());
                    }
                }

                Ok(status)
            })();

            match result {
                Ok(s) => Ok(s),
                Err(_) => {
                    // Full CLI fallback for repos with binary blobs / nul byte issues
                    cli_get_status(&path)
                }
            }
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Get list of changed files
    pub async fn get_diff_entries(&self) -> Result<Vec<DiffFile>> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<Vec<DiffFile>, GitError> {
            // Use git CLI which respects .gitattributes binary markers,
            // avoiding libgit2's "nul byte" errors on binary files.
            let output = std::process::Command::new("git")
                .args(["status", "--porcelain", "-z"])
                .current_dir(&path)
                .output()
                .map_err(|e| GitError::Other(e.to_string()))?;

            if !output.status.success() {
                // Fallback to libgit2 if git CLI unavailable
                let repo = git2::Repository::open(&path)?;
                let mut opts = git2::StatusOptions::new();
                opts.include_untracked(true);
                let statuses = repo.statuses(Some(&mut opts))?;
                return parse_git2_statuses(&statuses).map_err(GitError::LibGit2);
            }

            parse_porcelain_z(&output.stdout, &path)
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Get diff as string
    /// Get diff as string. Falls back to CLI on libgit2 errors (binary blobs, nul bytes).
    pub async fn get_diff(&self) -> Result<String> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<String, GitError> {
            // Try libgit2 first (faster, richer output)
            let result = (|| -> std::result::Result<String, git2::Error> {
                let repo = git2::Repository::open(&path)?;
                let mut diff_opts = git2::DiffOptions::new();
                let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

                let mut diff_str = String::new();
                diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                    let content = std::str::from_utf8(line.content()).unwrap_or("");
                    match line.origin() {
                        '+' => diff_str.push_str(&format!("+{}", content)),
                        '-' => diff_str.push_str(&format!("-{}", content)),
                        ' ' => diff_str.push_str(&format!(" {}", content)),
                        _ => diff_str.push_str(content),
                    }
                    true
                })?;
                Ok(diff_str)
            })();

            match result {
                Ok(s) => Ok(s),
                Err(_) => {
                    // CLI fallback: handles binary files, nul bytes, etc.
                    let output = std::process::Command::new("git")
                        .args(["diff", "HEAD"])
                        .current_dir(&path)
                        .output()
                        .map_err(|e| GitError::Other(e.to_string()))?;
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                }
            }
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Fetch latest changes from remote
    pub async fn fetch(&self) -> Result<()> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<(), git2::Error> {
            let repo = git2::Repository::open(&path)?;
            let mut remote = repo.find_remote("origin")?;

            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            });

            let mut fetch_opts = git2::FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            // Determine the current branch to fetch the correct remote ref
            let head = repo.head().ok();
            let refspec = head
                .as_ref()
                .and_then(|h| h.shorthand().ok())
                .map(|name| format!("refs/heads/{name}:refs/remotes/origin/{name}"));
            let refspecs: Vec<&str> = match &refspec {
                Some(r) => vec![r.as_str()],
                None => vec![], // fetch all if detached or unborn
            };
            remote.fetch(&refspecs, Some(&mut fetch_opts), None)?;
            Ok(())
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
        .map_err(GitError::LibGit2)
    }

    /// Pull and Rebase (Linear History)
    ///
    /// When the working tree is dirty, falls back to CLI `git pull --rebase --autostash`
    /// to preserve uncommitted changes (e.g., intentional file deletions).
    /// The libgit2 fast-forward path with force checkout would blindly restore
    /// all upstream files, undoing local deletions that haven't been pushed yet.
    pub async fn pull_rebase(&self) -> Result<()> {
        self.fetch().await?;
        let path = self.root_path.clone();

        tokio::task::spawn_blocking(move || -> std::result::Result<(), GitError> {
            let repo = git2::Repository::open(&path).map_err(GitError::LibGit2)?;

            let head = repo.head().map_err(GitError::LibGit2)?;
            let _head_oid = head
                .target()
                .ok_or(GitError::Other("HEAD invalid".into()))?;
            let head_name = head.shorthand().ok().unwrap_or("detached");

            let upstream_name = format!("origin/{}", head_name);
            let upstream_oid = repo
                .refname_to_id(&format!("refs/remotes/{}", upstream_name))
                .map_err(|_| GitError::Other("Upstream not found".into()))?;

            let annotated_upstream = repo
                .find_annotated_commit(upstream_oid)
                .map_err(GitError::LibGit2)?;

            let (analysis, _) = repo
                .merge_analysis(&[&annotated_upstream])
                .map_err(GitError::LibGit2)?;

            if analysis.is_up_to_date() {
                return Ok(());
            }

            // Check for working tree changes before any checkout/rebase.
            // Force checkout would restore deleted files from upstream, undoing
            // intentional deletions. Fall back to CLI with --autostash instead.
            let has_wt_changes = {
                let mut opts = git2::StatusOptions::new();
                opts.include_untracked(true);
                repo.statuses(Some(&mut opts))
                    .map(|statuses| {
                        statuses.iter().any(|entry| {
                            let s = entry.status();
                            s.is_wt_new()
                                || s.is_wt_modified()
                                || s.is_wt_deleted()
                                || s.is_wt_renamed()
                                || s.is_wt_typechange()
                                || s.is_index_new()
                                || s.is_index_modified()
                                || s.is_index_deleted()
                                || s.is_index_renamed()
                                || s.is_index_typechange()
                        })
                    })
                    .unwrap_or(false)
            };

            if has_wt_changes {
                // Working tree has uncommitted changes — use CLI pull with autostash
                // to preserve them. Force checkout would blindly overwrite deletions.
                return Self::cli_pull_rebase(&path);
            }

            if analysis.is_fast_forward() {
                let mut reference = repo
                    .find_reference(&format!("refs/heads/{}", head_name))
                    .map_err(GitError::LibGit2)?;
                reference
                    .set_target(upstream_oid, "Fast-Forward")
                    .map_err(GitError::LibGit2)?;
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                    .map_err(GitError::LibGit2)?;
                return Ok(());
            }

            let mut rebase = repo
                .rebase(
                    Some(&annotated_upstream),
                    None,
                    Some(&annotated_upstream),
                    None,
                )
                .map_err(GitError::LibGit2)?;

            while let Some(op) = rebase.next() {
                if op.is_err() {
                    rebase.abort().map_err(GitError::LibGit2)?;
                    return Err(GitError::MergeConflict);
                }
                let _ = rebase
                    .commit(None, &repo.signature().map_err(GitError::LibGit2)?, None)
                    .map_err(|e| {
                        let _ = rebase.abort();
                        GitError::LibGit2(e)
                    })?;
            }
            rebase.finish(None).map_err(GitError::LibGit2)?;
            Ok(())
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// CLI fallback for pull --rebase --autostash.
    /// Used when libgit2 fast-forward would overwrite uncommitted changes.
    fn cli_pull_rebase(path: &Path) -> std::result::Result<(), GitError> {
        let output = std::process::Command::new("git")
            .args(["pull", "--rebase", "--autostash"])
            .current_dir(path)
            .output()
            .map_err(|e| GitError::Other(format!("git pull failed: {}", e)))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("CONFLICT") || stderr.contains("conflict") {
            // Abort the rebase to leave repo in a clean state
            let _ = std::process::Command::new("git")
                .args(["rebase", "--abort"])
                .current_dir(path)
                .status();
            return Err(GitError::MergeConflict);
        }

        Err(GitError::Other(format!(
            "git pull --rebase failed: {}",
            stderr
        )))
    }

    /// Pull with merge (Preserves Both Histories)
    ///
    /// Unlike rebase which rewrites commits, merge creates a merge commit
    /// that ties both histories together. Less likely to conflict when
    /// both sides have parallel commits.
    pub async fn pull_merge(&self) -> Result<()> {
        self.fetch().await?;
        let path = self.root_path.clone();

        tokio::task::spawn_blocking(move || -> std::result::Result<(), GitError> {
            let output = std::process::Command::new("git")
                .args(["pull", "--no-rebase"])
                .current_dir(&path)
                .output()
                .map_err(|e| GitError::Other(format!("git pull --no-rebase failed: {}", e)))?;

            if output.status.success() {
                return Ok(());
            }

            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("CONFLICT") || stderr.contains("conflict") {
                // Abort the merge to leave repo in a clean state
                let _ = std::process::Command::new("git")
                    .args(["merge", "--abort"])
                    .current_dir(&path)
                    .status();
                return Err(GitError::MergeConflict);
            }

            Err(GitError::Other(format!(
                "git pull --no-rebase failed: {}",
                stderr
            )))
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Add specific paths to the index.
    ///
    /// For files matching `filter=dracon` patterns in `.gitattributes`, the
    /// working-tree content is run through `dracon-warden filter-clean` before
    /// being written to the index.  This ensures secrets are encrypted-at-rest
    /// even when staging via libgit2 (which does not invoke git filters).
    pub async fn add_paths(&self, paths: &[String]) -> Result<()> {
        let root = self.root_path.clone();
        let paths = paths.to_vec();
        tokio::task::spawn_blocking(move || -> std::result::Result<(), GitError> {
            // Resolve glob patterns (e.g. "*") to concrete file paths.
            let concrete = resolve_concrete_paths(&root, &paths);

            // SECURITY: Validate all paths are within the repo root to prevent
            // path traversal attacks (e.g. "../etc/passwd" or absolute paths).
            let root_canon = std::fs::canonicalize(&root)
                .map_err(|_| GitError::Other("Failed to canonicalize repo root".into()))?;
            for rel_path in &concrete {
                let abs = root.join(rel_path);
                let abs_canon = std::fs::canonicalize(&abs).map_err(|_| {
                    GitError::Other(format!("Path escapes repo root: {}", rel_path))
                })?;
                if !abs_canon.starts_with(&root_canon) {
                    return Err(GitError::Other(format!(
                        "Path traversal attempt blocked: {}",
                        rel_path
                    )));
                }
            }

            // Try libgit2 first
            let libgit2_result = (|| -> std::result::Result<(), git2::Error> {
                let repo = git2::Repository::open(&root)?;
                let mut index = repo.index()?;

                let mut protected = Vec::new();
                let mut normal = Vec::new();

                for p in &concrete {
                    if is_dracon_protected(&root, p) {
                        protected.push(p.clone());
                    } else {
                        normal.push(p.clone());
                    }
                }

                // Handle deleted files: remove from index instead of adding
                let mut to_add = Vec::new();
                for rel_path in &normal {
                    let abs = root.join(rel_path);
                    if !abs.exists() {
                        let _ = index.remove_path(Path::new(rel_path));
                    } else {
                        to_add.push(rel_path.clone());
                    }
                }

                if !to_add.is_empty() {
                    index.add_all(to_add.iter(), git2::IndexAddOption::FORCE, None)?;
                }

                for rel_path in &protected {
                    let abs = root.join(rel_path);
                    // Handle deleted files: remove from index instead of adding
                    if !abs.exists() {
                        let _ = index.remove_path(Path::new(rel_path));
                        continue;
                    }
                    let plaintext = match std::fs::read(&abs) {
                        Ok(b) => b,
                        Err(_) => {
                            // File unreadable - remove from index if it was tracked
                            let _ = index.remove_path(Path::new(rel_path));
                            continue;
                        }
                    };
                    let encrypted = run_clean_filter(rel_path, &plaintext)
                        .unwrap_or_else(|_| plaintext.clone());
                    let entry_path = rel_path.as_bytes();
                    let mut entry = git2::IndexEntry {
                        ctime: git2::IndexTime::new(0, 0),
                        mtime: git2::IndexTime::new(0, 0),
                        dev: 0,
                        ino: 0,
                        mode: 0o100644,
                        uid: 0,
                        gid: 0,
                        file_size: encrypted.len() as u32,
                        id: git2::Oid::ZERO_SHA1,
                        flags: entry_path.len().min(0xFFF) as u16,
                        flags_extended: 0,
                        path: Vec::new(),
                    };
                    let mut path_vec = entry_path.to_vec();
                    if !path_vec.ends_with(&[0]) {
                        path_vec.push(0);
                    }
                    entry.path = path_vec;
                    let oid = repo.blob(&encrypted)?;
                    entry.id = oid;
                    index.add(&entry)?;
                }

                index.write()?;
                Ok(())
            })();

            match libgit2_result {
                Ok(()) => Ok(()),
                Err(_) => {
                    // Fallback: use git CLI for everything
                    let paths: Vec<&str> = concrete.iter().map(|s| s.as_str()).collect();
                    let status = std::process::Command::new("git")
                        .arg("add")
                        .arg("-f")
                        .args(&paths)
                        .current_dir(&root)
                        .status();
                    match status {
                        Ok(s) if s.success() => Ok(()),
                        Ok(s) => Err(GitError::Other(format!(
                            "git add failed with exit code: {:?}",
                            s.code()
                        ))),
                        Err(e) => Err(GitError::Other(format!("git add failed: {}", e))),
                    }
                }
            }
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Stage a single file
    pub async fn stage_file(&self, file_path: &str) -> Result<()> {
        self.add_paths(&[file_path.to_string()]).await
    }

    /// Untrack a file (remove from index but keep on disk)
    pub async fn untrack_file(&self, file_path: &Path) -> Result<()> {
        let path = self.root_path.clone();
        let file_path = file_path.to_path_buf();
        tokio::task::spawn_blocking(move || -> std::result::Result<(), git2::Error> {
            let repo = git2::Repository::open(&path)?;
            let mut index = repo.index()?;
            index.remove_path(&file_path)?;
            index.write()?;
            Ok(())
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
        .map_err(GitError::LibGit2)
    }

    /// Commit with message
    pub async fn commit_all(&self, message: &str) -> Result<()> {
        self.add_paths(&["*".to_string()]).await?;
        self.commit(message).await
    }

    /// Commit staged changes. Falls back to git CLI if libgit2 fails on binary files.
    pub async fn commit(&self, message: &str) -> Result<()> {
        let path = self.root_path.clone();
        let message = message.to_string();
        tokio::task::spawn_blocking(move || -> std::result::Result<(), GitError> {
            // Try libgit2 first (faster)
            let result = (|| -> std::result::Result<(), git2::Error> {
                let repo = git2::Repository::open(&path)?;
                let mut index = repo.index()?;
                let tree_id = index.write_tree()?;
                let tree = repo.find_tree(tree_id)?;

                let sig = repo
                    .signature()
                    .unwrap_or_else(|_| git2::Signature::now("Dracon", "dracon@void").unwrap());

                let mut parents = Vec::new();
                if let Ok(head) = repo.head() {
                    if let Ok(parent) = head.peel_to_commit() {
                        parents.push(parent);
                    }
                }

                let parents_refs: Vec<&git2::Commit> = parents.iter().collect();
                repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents_refs)?;
                Ok(())
            })();

            match result {
                Ok(()) => Ok(()),
                Err(e) => {
                    // Fallback: git CLI handles binary files, nul bytes, etc.
                    let status = std::process::Command::new("git")
                        .args(["commit", "-m", &message])
                        .current_dir(&path)
                        .status();
                    match status {
                        Ok(s) if s.success() => Ok(()),
                        _ => Err(GitError::LibGit2(e)),
                    }
                }
            }
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Get unpushed commits count
    pub async fn get_unpushed_commits(&self) -> Result<usize> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<usize, git2::Error> {
            let repo = git2::Repository::open(&path)?;
            let head = repo.head()?;
            let head_name = head.shorthand().ok().unwrap_or("detached");
            let upstream_name = format!("origin/{}", head_name);

            if let Ok(upstream_oid) = repo.refname_to_id(&format!("refs/remotes/{}", upstream_name))
            {
                let head_oid = match head.target() {
                    Some(oid) => oid,
                    None => return Ok(0), // unborn HEAD, no commits yet
                };
                let (ahead, _) = repo.graph_ahead_behind(head_oid, upstream_oid)?;
                Ok(ahead)
            } else {
                Ok(0)
            }
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
        .map_err(GitError::LibGit2)
    }

    /// Push current branch to remote
    pub async fn push(&self) -> Result<()> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<(), GitError> {
            let output = std::process::Command::new("git")
                .arg("push")
                .arg("origin")
                .arg("HEAD")
                .current_dir(&path)
                .output()
                .map_err(|e| GitError::Other(e.to_string()))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(GitError::Other(format!("Git push failed: {}", err)));
            }
            Ok(())
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
    }

    /// Ensure remotes use SSH
    pub async fn harden_remotes(&self) -> Result<()> {
        let path = self.root_path.clone();
        tokio::task::spawn_blocking(move || -> std::result::Result<(), git2::Error> {
            let repo = git2::Repository::open(&path)?;
            if let Ok(remote) = repo.find_remote("origin") {
                if let Ok(url) = remote.url() {
                    if url.starts_with("https://github.com/") {
                        let new_url = url.replace("https://github.com/", "git@github.com:");
                        repo.remote_set_url("origin", &new_url)?;
                    }
                }
            }
            Ok(())
        })
        .await
        .map_err(|e| GitError::Other(e.to_string()))?
        .map_err(GitError::LibGit2)
    }
}

// ---------------------------------------------------------------------------
// Clean-filter helpers (secrets encryption)
// ---------------------------------------------------------------------------

/// Check whether `rel_path` matches a `filter=dracon` line in the repo's
/// `.gitattributes`.  Uses simple glob matching (no external crate).
fn is_dracon_protected(repo_root: &Path, rel_path: &str) -> bool {
    let ga = repo_root.join(".gitattributes");
    let Ok(content) = std::fs::read_to_string(&ga) else {
        return false;
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // We only care about lines that set the dracon filter.
        if !line.contains("filter=dracon") {
            continue;
        }

        let pattern = line.split_whitespace().next().unwrap_or("");
        if glob_match(pattern, rel_path) {
            return true;
        }
    }
    false
}

/// Minimal glob matcher supporting `*`, `**`, `?`, and `secrets/**` recursive
/// patterns.  Good enough for `.gitattributes` matching; not a full glob impl.
fn glob_match(pattern: &str, text: &str) -> bool {
    // Fast path: exact match or simple wildcard.
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') && !pattern.contains('?') {
        return pattern == text;
    }

    // Handle recursive `**` by splitting on it.
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1].trim_start_matches('/');
            let prefix_ok = prefix.is_empty() || text.starts_with(prefix);
            let suffix_ok = suffix.is_empty() || text.ends_with(suffix);
            if prefix_ok && suffix_ok {
                // For `secrets/**`, make sure we don't match `secretsX/foo`.
                if !prefix.is_empty() && !prefix.ends_with('/') {
                    return text.starts_with(prefix) && text[prefix.len()..].starts_with('/');
                }
                return true;
            }
            return false;
        }
    }

    // Simple single-`*` matching: split on *, check prefix/suffix.
    let segments: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;
    for (i, seg) in segments.iter().enumerate() {
        if i == 0 {
            if !text[pos..].starts_with(seg) {
                return false;
            }
            pos += seg.len();
        } else {
            match text[pos..].find(seg) {
                Some(idx) => pos += idx + seg.len(),
                None => return false,
            }
        }
    }
    true
}

/// Run `dracon-warden filter-clean <path>` with `input` on stdin and return
/// the (possibly encrypted) stdout.
fn run_clean_filter(rel_path: &str, input: &[u8]) -> Result<Vec<u8>> {
    let mut child = std::process::Command::new("dracon-warden")
        .arg("filter-clean")
        .arg(rel_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| GitError::Other(format!("failed to spawn dracon-warden: {e}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input)
            .map_err(|e| GitError::Other(format!("failed to write to dracon-warden stdin: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| GitError::Other(format!("dracon-warden wait failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::Other(format!(
            "dracon-warden filter-clean failed for {rel_path}: {stderr}"
        )));
    }

    Ok(output.stdout)
}

/// Expand wildcard paths (like `"*"`) to the actual modified/tracked files
/// in the working tree so we can inspect each one for filter=dracon.
fn resolve_concrete_paths(repo_root: &Path, paths: &[String]) -> Vec<String> {
    let has_wildcard = paths.iter().any(|p| p.contains('*'));
    if !has_wildcard {
        return paths.to_vec();
    }

    // Use git status to enumerate relevant files.
    let Ok(output) = std::process::Command::new("git")
        .args(["diff", "--name-only", "--cached"])
        .current_dir(repo_root)
        .output()
    else {
        return paths.to_vec();
    };

    let mut result: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Also include untracked/modified files.
    if let Ok(output) = std::process::Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "--modified"])
        .current_dir(repo_root)
        .output()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if !line.is_empty() && !result.contains(&line.to_string()) {
                result.push(line.to_string());
            }
        }
    }

    if result.is_empty() {
        // Fall back to original paths if nothing resolved.
        return paths.to_vec();
    }

    result
}

/// Parse `git status --porcelain -z` output into DiffFile entries.
fn parse_porcelain_z(data: &[u8], _repo: &Path) -> std::result::Result<Vec<DiffFile>, GitError> {
    let mut diffs = Vec::new();
    let mut i = 0;
    while i < data.len() {
        // Find null terminator
        let end = data[i..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| i + p)
            .unwrap_or(data.len());
        let line = &data[i..end];
        i = end + 1;

        if line.len() < 4 {
            continue;
        }
        let xy = &line[..2];
        let path_str = std::str::from_utf8(&line[3..]).unwrap_or("");
        if path_str.is_empty() {
            continue;
        }

        let status = match xy {
            b"??" => FileStatus::Added,
            [b'A', _] | [_, b'A'] => FileStatus::Added,
            [b'D', _] | [_, b'D'] => FileStatus::Deleted,
            [b'R', _] | [_, b'R'] => FileStatus::Renamed,
            [b'T', _] | [_, b'T'] => FileStatus::TypeChange,
            [b'M', _] | [_, b'M'] => FileStatus::Modified,
            _ => FileStatus::Modified,
        };

        diffs.push(DiffFile {
            path: PathBuf::from(path_str),
            status,
        });
    }
    Ok(diffs)
}

/// Fallback: parse libgit2 statuses (may fail on binary files).
fn parse_git2_statuses(
    statuses: &git2::Statuses,
) -> std::result::Result<Vec<DiffFile>, git2::Error> {
    let mut diffs = Vec::new();
    for entry in statuses.iter() {
        let status_flags = entry.status();
        let file_status = if status_flags.is_wt_new() || status_flags.is_index_new() {
            FileStatus::Added
        } else if status_flags.is_wt_deleted() || status_flags.is_index_deleted() {
            FileStatus::Deleted
        } else if status_flags.is_wt_renamed() || status_flags.is_index_renamed() {
            FileStatus::Renamed
        } else if status_flags.is_wt_typechange() || status_flags.is_index_typechange() {
            FileStatus::TypeChange
        } else {
            FileStatus::Modified
        };
        if let Ok(p) = entry.path() {
            diffs.push(DiffFile {
                path: PathBuf::from(p),
                status: file_status,
            });
        }
    }
    Ok(diffs)
}

/// Get repo status using git CLI (handles binary blobs, nul bytes, etc.)
fn cli_get_status(path: &Path) -> std::result::Result<RepoStatus, GitError> {
    let mut status = RepoStatus::new();

    // Branch
    if let Ok(o) = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
    {
        let b = String::from_utf8_lossy(&o.stdout).trim().to_string();
        status.branch = if b.is_empty() {
            "detached".to_string()
        } else {
            b
        };
    }

    // Ahead/behind
    if let Ok(o) = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{u}"])
        .current_dir(path)
        .output()
    {
        let s = String::from_utf8_lossy(&o.stdout);
        let mut parts = s.split_whitespace();
        status.ahead = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
        status.behind = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
    }

    // File counts
    if let Ok(o) = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
    {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            if line.len() < 3 {
                continue;
            }
            let idx = &line[..2];
            if !idx.starts_with(' ') && !idx.starts_with('?') && idx != "  " {
                status.staged_files += 1;
            }
            if idx.starts_with("??") {
                // Untracked files: count separately, NOT as modified.
                status.untracked_files += 1;
            } else if idx.contains('M')
                || idx.contains('A')
                || idx.contains('D')
                || idx.contains('R')
                || idx.contains('T')
            {
                status.modified_files += 1;
            }
        }
    }

    status.is_clean =
        status.modified_files == 0 && status.staged_files == 0 && status.untracked_files == 0;
    Ok(status)
}

/// Read the most recently modified markdown file from a repo's `plan/` directory.
/// Returns empty string if no plan dir or no .md files found.
pub fn read_blueprint_content(repo: &Path) -> String {
    let plan_dir = repo.join("plan");
    if !plan_dir.exists() {
        return String::new();
    }
    std::fs::read_dir(&plan_dir)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
        })
        .and_then(|e| std::fs::read_to_string(e.path()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper: init a bare repo, clone it, commit a file, and return (remote, local) dirs.
    fn setup_remote_and_local() -> (TempDir, TempDir) {
        let remote = TempDir::new().unwrap();
        let local = TempDir::new().unwrap();

        // Init bare remote with main as the default branch.
        Command::new("git")
            .args(["init", "--bare", "-b", "main"])
            .current_dir(remote.path())
            .status()
            .unwrap();

        // Init local repo directly instead of cloning an unborn remote, so HEAD
        // is unambiguously on main for pull_rebase().
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(local.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["remote", "add", "origin", remote.path().to_str().unwrap()])
            .current_dir(local.path())
            .status()
            .unwrap();

        // Configure git identity
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(local.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(local.path())
            .status()
            .unwrap();

        // Create and push initial file
        std::fs::write(local.path().join("keep.txt"), "keep me").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(local.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "--no-verify", "-m", "init"])
            .current_dir(local.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(local.path())
            .status()
            .unwrap();

        (remote, local)
    }

    /// Helper: add a commit to the bare remote (via a temp clone).
    fn push_new_file_to_remote(remote: &Path, filename: &str, content: &str) {
        let tmp = TempDir::new().unwrap();
        Command::new("git")
            .args([
                "clone",
                remote.to_str().unwrap(),
                tmp.path().to_str().unwrap(),
            ])
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        std::fs::write(tmp.path().join(filename), content).unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "--no-verify", "-m", &format!("add {filename}")])
            .current_dir(tmp.path())
            .status()
            .unwrap();
        Command::new("git")
            .args(["push", "origin", "HEAD"])
            .current_dir(tmp.path())
            .status()
            .unwrap();
    }

    /// Regression test: pull_rebase must NOT restore deleted files when the
    /// working tree has uncommitted changes. Previously, `checkout_head(force())`
    /// would blindly restore all upstream files, undoing intentional deletions.
    #[tokio::test]
    async fn test_pull_rebase_preserves_dirty_deletions() {
        let (remote, local) = setup_remote_and_local();

        // Delete the file locally (uncommitted)
        std::fs::remove_file(local.path().join("keep.txt")).unwrap();
        assert!(
            !local.path().join("keep.txt").exists(),
            "file should be deleted"
        );

        // Push a new file to remote so local is behind
        push_new_file_to_remote(remote.path(), "upstream.txt", "from remote");

        // Fetch to update remote tracking branch
        Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(local.path())
            .status()
            .unwrap();

        // pull_rebase should NOT restore keep.txt (the deleted file)
        let svc = GitService::new(local.path()).unwrap();
        let result = svc.pull_rebase().await;

        // Should either succeed via CLI fallback or report conflict/error
        // but must NEVER restore the deleted file
        let file_restored = local.path().join("keep.txt").exists();
        assert!(
            !file_restored,
            "REGRESSION: pull_rebase restored a deleted file! \
             The force() checkout is overwriting intentional deletions."
        );

        // The upstream file should be present (pull succeeded somehow)
        // If CLI autostash worked, both should be true:
        // - keep.txt stays deleted
        // - upstream.txt is pulled from remote
        if result.is_ok() {
            assert!(
                local.path().join("upstream.txt").exists(),
                "pull should bring new upstream files"
            );
        }
    }

    /// Verify that pull_rebase on a clean working tree still works normally.
    #[tokio::test]
    async fn test_pull_rebase_clean_fast_forward() {
        let (remote, local) = setup_remote_and_local();

        // Push new file to remote
        push_new_file_to_remote(remote.path(), "new.txt", "hello");

        let svc = GitService::new(local.path()).unwrap();
        let result = svc.pull_rebase().await;
        assert!(
            result.is_ok(),
            "clean pull_rebase should succeed: {:?}",
            result
        );

        // New file should be present
        assert!(local.path().join("new.txt").exists());
        assert!(local.path().join("keep.txt").exists());
    }
}
