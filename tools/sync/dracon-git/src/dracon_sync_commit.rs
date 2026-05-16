use crate::task_scan::TaskProgress;
use crate::types::{DiffFile, FileStatus};

/// Lightweight semantic summary of changed files.
/// Any tool can construct this — no tree-sitter dependency required.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticSummary {
    pub symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct CommitContext {
    pub intent: String,
    pub track: Option<String>,
    pub is_checkpoint: bool,
    pub files: Vec<DiffFile>,
    pub task_progress: Option<TaskProgress>,
    pub refs: Option<String>,
    pub idle_seconds: u64,
    /// Override auto-detected category (fix, feat, refactor, security, perf, chore, docs)
    pub category: Option<String>,
    /// Override auto-detected scope (warden, sync, system, ai, shared-libs)
    pub scope: Option<String>,
    /// Severity for security/fix commits (critical, high, medium, low)
    pub severity: Option<String>,
    /// Human-readable description of why this change was made
    pub description: Option<String>,
    /// Semantic symbol summary of changed files (from tree-sitter analysis)
    pub semantic_summary: Option<SemanticSummary>,
}

impl CommitContext {
    pub fn new(intent: String, files: Vec<DiffFile>, is_checkpoint: bool) -> Self {
        Self {
            intent,
            track: None,
            is_checkpoint,
            files,
            task_progress: None,
            refs: None,
            idle_seconds: 0,
            category: None,
            scope: None,
            severity: None,
            description: None,
            semantic_summary: None,
        }
    }
}

/// Auto-detect scope from file paths
fn detect_scope(files: &[DiffFile]) -> &'static str {
    let mut scopes = std::collections::BTreeSet::new();

    for f in files {
        let path_str = f.path.to_string_lossy().to_lowercase();
        let parts: Vec<&str> = path_str.split('/').collect();

        // Check for known subsystem paths
        if parts
            .iter()
            .any(|p| *p == "dracon-warden" || *p == "warden")
        {
            scopes.insert("warden");
        } else if parts
            .iter()
            .any(|p| *p == "dracon-sync" || *p == "sync" || *p == "dracon-git")
        {
            scopes.insert("sync");
        } else if parts
            .iter()
            .any(|p| *p == "dracon-system" || *p == "system")
        {
            scopes.insert("system");
        } else if parts.iter().any(|p| *p == "dracon-ai" || *p == "ai") {
            scopes.insert("ai");
        } else if parts
            .iter()
            .any(|p| *p == "dracon-security" || *p == "dracon-config" || *p == "dracon-libs")
        {
            scopes.insert("shared-libs");
        } else if path_str.contains("install") {
            scopes.insert("build");
        } else if parts.iter().any(|p| p.ends_with(".md")) {
            scopes.insert("docs");
        }
    }

    if scopes.len() == 1 {
        scopes.into_iter().next().unwrap()
    } else if scopes.is_empty() {
        "misc"
    } else {
        "multi"
    }
}

/// Auto-detect category from file patterns and context
fn detect_category(files: &[DiffFile], is_checkpoint: bool) -> &'static str {
    if files.is_empty() {
        return "chore";
    }

    // Check for security-sensitive files
    for f in files {
        let name = f
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let path = f.path.to_string_lossy().to_lowercase();

        if name.contains("filter")
            || name.contains("scanner")
            || name.contains("crypto")
            || name.contains("identity")
            || name.contains("security")
            || path.contains("secrets/")
            || path.contains(".age")
            || path.contains(".pub")
        {
            return "security";
        }
    }

    // Check for chore files
    for f in files {
        let name = f
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name == "cargo.lock" || name == "cargo.toml" {
            return "chore";
        }
    }

    // Check for docs
    for f in files {
        let ext = f
            .path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if ext == "md" {
            return "docs";
        }
    }

    // Check for test files
    let all_tests = files.iter().all(|f| {
        let p = f.path.to_string_lossy().to_lowercase();
        p.contains("/test") || p.contains("_test.") || p.contains("tests/")
    });
    if all_tests {
        return "test";
    }

    // WIP checkpoints
    if is_checkpoint {
        return "chore";
    }

    // Default: if few files changed, likely a fix; more files, likely a feature
    if files.len() <= 2 {
        "fix"
    } else {
        "feat"
    }
}

/// Extract a one-line summary from the scribe's "Current Focus" section in project-state.md
/// Handles both full markdown content and just the focus section.
fn extract_focus_summary(description: Option<&str>) -> Option<String> {
    let desc = description?;

    // Find "Current Focus" section (handles full markdown or just focus section)
    let mut in_focus = false;
    for line in desc.lines() {
        let trimmed = line.trim();

        // Enter focus section (handle "## Current Focus", "##Current Focus", "## CurrentFocus", etc.)
        let lower = trimmed.to_lowercase();
        let is_header = (trimmed.starts_with("## ") || trimmed.starts_with("##"))
            && (lower.contains("current focus") || lower.contains("currentfocus"));
        if is_header {
            in_focus = true;
            // Find where "current focus" or "currentfocus" ends
            let has_space_format = lower.contains("current focus");
            let focus_phrase_len = if has_space_format { 13 } else { 11 }; // "current focus"=13, "currentfocus"=11
            let search_start = lower
                .find("current focus")
                .or_else(|| lower.find("currentfocus"))
                .unwrap_or(0);
            let content_start = search_start + focus_phrase_len;

            // Only extract from same line if we have proper space-separated format
            // and actual content follows the header
            if has_space_format && content_start < trimmed.len() {
                let after_header = trimmed[content_start..].trim();
                if !after_header.is_empty() && !after_header.starts_with('#') {
                    let summary = after_header.to_string();
                    return Some(if summary.len() > 72 {
                        summary.chars().take(69).collect::<String>() + "..."
                    } else {
                        summary
                    });
                }
            }
            // Otherwise, content is on the next line - continue to next iteration
            continue;
        }

        // Exit on next section
        if in_focus && trimmed.starts_with("## ") {
            break;
        }

        // Collect first non-empty line in focus section
        if in_focus && !trimmed.is_empty() {
            let mut summary = trimmed.to_string();
            if let Some(stripped) = summary
                .strip_prefix("ONE LINE:")
                .map(|s| s.trim_start().to_string())
            {
                summary = stripped;
            } else if let Some(stripped) = summary
                .strip_prefix("one line:")
                .map(|s| s.trim_start().to_string())
            {
                summary = stripped;
            }
            // Truncate to reasonable commit subject length
            if summary.len() > 72 {
                return Some(summary.chars().take(69).collect::<String>() + "...");
            }
            return Some(summary);
        }
    }

    // Fallback: use first non-empty line
    let first_line = desc.lines().find(|l| !l.trim().is_empty())?;
    let summary = first_line.trim().to_string();
    if summary.is_empty() || summary.starts_with('#') {
        return None;
    }
    if summary.len() > 100 {
        Some(summary.chars().take(97).collect::<String>() + "...")
    } else {
        Some(summary)
    }
}

pub fn build_commit_message(ctx: &CommitContext) -> String {
    let display_files: Vec<&DiffFile> = ctx
        .files
        .iter()
        .filter(|f| {
            f.path
                .file_name()
                .map(|n| n != "Cargo.lock")
                .unwrap_or(true)
        })
        .collect();

    // Auto-detect category and scope
    let category = ctx
        .category
        .as_deref()
        .unwrap_or_else(|| detect_category(&ctx.files, ctx.is_checkpoint));
    let scope = ctx
        .scope
        .as_deref()
        .unwrap_or_else(|| detect_scope(&ctx.files));

    // Count file statuses
    let mut added = 0usize;
    let mut modified = 0usize;
    let mut deleted = 0usize;
    for f in &display_files {
        match f.status {
            FileStatus::Added => added += 1,
            FileStatus::Modified => modified += 1,
            FileStatus::Deleted => deleted += 1,
            FileStatus::Renamed => modified += 1,
            FileStatus::TypeChange => modified += 1,
            FileStatus::Unknown => modified += 1,
        }
    }

    // Subject line: category(scope): summary
    // For non-checkpoint commits: if a description exists (AI or local fallback),
    // use it directly as the summary — parse_conventional_commit already extracted
    // the semantic meaning from the subject, so ctx.description IS the summary.
    // Only fall through to build_summary_line when description is genuinely absent.
    let summary = if ctx.is_checkpoint {
        extract_focus_summary(ctx.description.as_deref())
            .unwrap_or_else(|| "wip checkpoint".to_string())
    } else if ctx.description.is_some() {
        ctx.description.clone().unwrap()
    } else {
        build_summary_line(added, modified, deleted, &display_files)
    };
    let subject = format!("{}({}): {}", category, scope, summary);

    // Body: explicit description > semantic symbols > nothing
    // AI can `git diff` for raw details, but symbol names give it context without a tool call
    let body = ctx
        .description
        .clone()
        .or_else(|| format_semantic_body(&ctx.semantic_summary));

    // Machine-readable footer — only fields AI can't trivially get itself
    let mut footer_lines = Vec::new();
    footer_lines.push(format!("category: {}", category));
    footer_lines.push(format!("scope: {}", scope));

    if let Some(ref severity) = ctx.severity {
        footer_lines.push(format!("severity: {}", severity));
    }

    if let Some(ref refs) = ctx.refs {
        footer_lines.push(format!("refs: {}", refs));
    }

    // Assemble
    let mut msg = subject;
    if let Some(ref body_text) = body {
        msg.push('\n');
        msg.push('\n');
        msg.push_str(body_text);
    }
    msg.push('\n');
    msg.push_str("---\n");
    msg.push_str(&footer_lines.join("\n"));
    msg.push_str("\n---");

    msg
}

fn build_summary_line(
    added: usize,
    modified: usize,
    deleted: usize,
    _files: &[&DiffFile],
) -> String {
    let total = added + modified + deleted;
    if total == 0 {
        return "no changes".to_string();
    }

    let mut parts = Vec::new();
    if added > 0 {
        parts.push(format!("{} added", added));
    }
    if modified > 0 {
        parts.push(format!("{} modified", modified));
    }
    if deleted > 0 {
        parts.push(format!("{} deleted", deleted));
    }

    parts.join(", ")
}

fn format_semantic_body(summary: &Option<SemanticSummary>) -> Option<String> {
    let summary = summary.as_ref()?;
    if summary.symbols.is_empty() {
        return None;
    }

    let lines: Vec<String> = summary
        .symbols
        .iter()
        .map(|s| format!("{} {} ({})", s.kind, s.name, s.language))
        .collect();

    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ctx_minimal(intent: &str, files: Vec<DiffFile>, is_checkpoint: bool) -> CommitContext {
        CommitContext {
            intent: intent.to_string(),
            track: None,
            is_checkpoint,
            files,
            task_progress: None,
            refs: None,
            idle_seconds: 0,
            category: None,
            scope: None,
            severity: None,
            description: None,
            semantic_summary: None,
        }
    }

    #[test]
    fn test_structured_checkpoint() {
        let mut ctx = ctx_minimal(
            "my-feature",
            vec![DiffFile {
                path: PathBuf::from("dracon-warden/src/main.rs"),
                status: FileStatus::Modified,
            }],
            true,
        );
        ctx.track = Some("V3".to_string());
        ctx.idle_seconds = 300;

        let msg = build_commit_message(&ctx);
        let lines: Vec<&str> = msg.lines().collect();

        assert!(lines[0].starts_with("chore(warden):"));
        assert!(msg.contains("---\n"));
        assert!(msg.contains("category: chore"));
        assert!(msg.contains("scope: warden"));
    }

    #[test]
    fn test_structured_fix() {
        let mut ctx = ctx_minimal(
            "fix-auth",
            vec![
                DiffFile {
                    path: PathBuf::from("dracon-sync/src/main.rs"),
                    status: FileStatus::Modified,
                },
                DiffFile {
                    path: PathBuf::from("dracon-libs/tools/sync/dracon-git/src/lib.rs"),
                    status: FileStatus::Modified,
                },
            ],
            false,
        );
        ctx.task_progress = Some(TaskProgress {
            done: 3,
            in_progress: 1,
            pending: 1,
        });
        ctx.refs = Some("plan/blueprint.md".to_string());

        let msg = build_commit_message(&ctx);
        assert!(msg.contains("fix(sync):"));
        assert!(msg.contains("category: fix"));
        assert!(msg.contains("scope: sync"));
        assert!(msg.contains("refs: plan/blueprint.md"));
    }

    #[test]
    fn test_security_category_detection() {
        let mut ctx = ctx_minimal(
            "security-fix",
            vec![DiffFile {
                path: PathBuf::from("dracon-libs/tools/security/dracon-security/src/filter.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx.severity = Some("high".to_string());
        ctx.description = Some("Binary files bypassed encryption silently".to_string());

        let msg = build_commit_message(&ctx);
        assert!(msg.contains("security("));
        assert!(msg.contains("category: security"));
        assert!(msg.contains("severity: high"));
        assert!(msg.contains("Binary files bypassed encryption silently"));
    }

    #[test]
    fn test_chore_detection() {
        let ctx = ctx_minimal(
            "deps",
            vec![
                DiffFile {
                    path: PathBuf::from("Cargo.lock"),
                    status: FileStatus::Modified,
                },
                DiffFile {
                    path: PathBuf::from("Cargo.toml"),
                    status: FileStatus::Modified,
                },
            ],
            false,
        );

        let msg = build_commit_message(&ctx);
        assert!(msg.contains("chore("));
        assert!(msg.contains("category: chore"));
    }

    #[test]
    fn test_explicit_category_override() {
        let mut ctx = ctx_minimal(
            "custom",
            vec![DiffFile {
                path: PathBuf::from("src/main.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx.category = Some("perf".to_string());
        ctx.scope = Some("core".to_string());
        ctx.description = Some("Optimized hot path in request handler".to_string());

        let msg = build_commit_message(&ctx);
        assert!(msg.starts_with("perf(core):"));
        assert!(msg.contains("category: perf"));
        assert!(msg.contains("scope: core"));
        assert!(msg.contains("Optimized hot path"));
    }

    #[test]
    fn test_multi_scope_detection() {
        let ctx = ctx_minimal(
            "cross-cut",
            vec![
                DiffFile {
                    path: PathBuf::from("dracon-warden/src/main.rs"),
                    status: FileStatus::Modified,
                },
                DiffFile {
                    path: PathBuf::from("dracon-sync/src/main.rs"),
                    status: FileStatus::Modified,
                },
            ],
            false,
        );

        let msg = build_commit_message(&ctx);
        assert!(msg.contains("(multi):"));
        assert!(msg.contains("scope: multi"));
    }

    #[test]
    fn test_show_example_formats() {
        // Security fix with description
        let mut ctx1 = ctx_minimal(
            "keygen-fix",
            vec![DiffFile {
                path: PathBuf::from("dracon-warden/src/main.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx1.category = Some("security".to_string());
        ctx1.severity = Some("critical".to_string());
        ctx1.description =
            Some("Use create_new(true) to atomically fail if key file already exists".to_string());
        let msg1 = build_commit_message(&ctx1);
        println!("\n--- SECURITY FIX ---\n{}\n", msg1);

        // WIP checkpoint with refs
        let mut ctx2 = ctx_minimal(
            "my-feature",
            vec![
                DiffFile {
                    path: PathBuf::from("dracon-sync/src/main.rs"),
                    status: FileStatus::Modified,
                },
                DiffFile {
                    path: PathBuf::from("dracon-warden/src/main.rs"),
                    status: FileStatus::Modified,
                },
            ],
            true,
        );
        ctx2.track = Some("V3".to_string());
        ctx2.task_progress = Some(TaskProgress {
            done: 3,
            in_progress: 1,
            pending: 5,
        });
        ctx2.refs = Some("plan/blueprint.md".to_string());
        let msg2 = build_commit_message(&ctx2);
        println!("--- WIP CHECKPOINT ---\n{}\n", msg2);

        // Daemon auto-commit
        let ctx3 = ctx_minimal(
            "daemon",
            vec![
                DiffFile {
                    path: PathBuf::from("dracon-system/src/main.rs"),
                    status: FileStatus::Modified,
                },
                DiffFile {
                    path: PathBuf::from("dracon-system/src/lib.rs"),
                    status: FileStatus::Added,
                },
            ],
            false,
        );
        let msg3 = build_commit_message(&ctx3);
        println!("--- DAEMON AUTO ---\n{}\n", msg3);
    }

    #[test]
    fn test_semantic_body() {
        let mut ctx = ctx_minimal(
            "fix",
            vec![DiffFile {
                path: PathBuf::from("src/main.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx.scope = Some("core".to_string());

        // With semantic summary
        ctx.semantic_summary = Some(SemanticSummary {
            symbols: vec![
                SymbolInfo {
                    name: "run".to_string(),
                    kind: "function".to_string(),
                    language: "rust".to_string(),
                },
                SymbolInfo {
                    name: "Config".to_string(),
                    kind: "struct".to_string(),
                    language: "rust".to_string(),
                },
            ],
        });

        let msg = build_commit_message(&ctx);
        assert!(msg.contains("function run (rust)"));
        assert!(msg.contains("struct Config (rust)"));
    }

    #[test]
    fn test_ai_description_used_in_non_checkpoint_subject() {
        let mut ctx = ctx_minimal(
            "daemon",
            vec![DiffFile {
                path: PathBuf::from("dracon-sync/src/git.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx.category = Some("fix".to_string());
        ctx.scope = Some("sync".to_string());
        ctx.description = Some("add SSH hardening to push stderr capture".to_string());

        let msg = build_commit_message(&ctx);
        let subject = msg.lines().next().unwrap();
        assert!(
            subject.contains("add SSH hardening to push stderr capture"),
            "AI description should appear in subject, got: {}",
            subject
        );
        assert!(subject.starts_with("fix(sync):"));
    }

    #[test]
    fn test_ai_description_not_truncated_arbitrarily() {
        let mut ctx = ctx_minimal(
            "daemon",
            vec![DiffFile {
                path: PathBuf::from("dracon-sync/src/main.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx.category = Some("feat".to_string());
        ctx.scope = Some("sync".to_string());
        ctx.description =
            Some("add SSH hardening to push stderr capture for better debugging".to_string());

        let msg = build_commit_message(&ctx);
        let subject = msg.lines().next().unwrap();
        assert!(
            subject.starts_with("feat(sync):"),
            "subject should start with correct category/scope"
        );
        // AI is prompted to keep subjects <=72 chars, so description is used as-is
        assert!(
            subject.contains("add SSH hardening to push stderr capture for better debugging"),
            "full AI description should appear in subject, got: {}",
            subject
        );
    }

    #[test]
    fn test_local_fallback_description_used_directly() {
        let mut ctx = ctx_minimal(
            "daemon",
            vec![DiffFile {
                path: PathBuf::from("dracon-sync/src/git.rs"),
                status: FileStatus::Modified,
            }],
            false,
        );
        ctx.description = Some("update auth, jwt and 2 files".to_string());

        let msg = build_commit_message(&ctx);
        let subject = msg.lines().next().unwrap();
        assert!(
            subject.contains("update auth, jwt and 2 files"),
            "local fallback description should appear in subject, got: {}",
            subject
        );
    }

    #[test]
    fn test_checkpoint_uses_extract_focus_summary_still() {
        let mut ctx = ctx_minimal(
            "my-feature",
            vec![DiffFile {
                path: PathBuf::from("dracon-sync/src/main.rs"),
                status: FileStatus::Modified,
            }],
            true,
        );
        ctx.description =
            Some("## Current Focus\nwip: implementing auth flow\nmore notes".to_string());

        let msg = build_commit_message(&ctx);
        let subject = msg.lines().next().unwrap();
        assert!(
            subject.contains("wip: implementing auth flow"),
            "checkpoint should use extract_focus_summary, got: {}",
            subject
        );
    }

    #[test]
    fn test_no_description_falls_back_to_summary_line() {
        let ctx = ctx_minimal(
            "daemon",
            vec![
                DiffFile {
                    path: PathBuf::from("dracon-sync/src/git.rs"),
                    status: FileStatus::Modified,
                },
                DiffFile {
                    path: PathBuf::from("dracon-sync/src/main.rs"),
                    status: FileStatus::Added,
                },
            ],
            false,
        );
        // description is None

        let msg = build_commit_message(&ctx);
        let subject = msg.lines().next().unwrap();
        assert!(
            subject.contains("modified") && subject.contains("added"),
            "no description should fall back to summary line with counts, got: {}",
            subject
        );
    }
}
