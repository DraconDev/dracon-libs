use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::task_scan::TaskProgress;

/// Extracted intent information from a repository's plan files or changed files.
#[derive(Debug, Clone, Default)]
pub struct IntentInfo {
    /// Detected intent label.
    pub intent: String,
    /// Active track/branch name.
    pub track: Option<String>,
    /// Path to the active blueprint file.
    pub blueprint: Option<PathBuf>,
    /// Task progress counts from the blueprint.
    pub task_progress: Option<TaskProgress>,
}

/// Extract intent info from a repository by analyzing plan files and changed files.
pub fn extract_intent(repo: &Path, changed_files: &[PathBuf], branch: Option<&str>) -> IntentInfo {
    let plan_dir = find_plan_dir(repo);

    if let Some(ref plan_dir) = plan_dir {
        if let Some(active_info) = extract_from_active_board(plan_dir, changed_files) {
            return IntentInfo {
                intent: active_info.intent,
                track: active_info.track,
                blueprint: Some(active_info.blueprint),
                task_progress: active_info.task_progress,
            };
        }
    }

    if let Some(intent) = extract_from_branch(branch.unwrap_or("")) {
        return IntentInfo {
            intent,
            track: extract_track_from_branch(branch.unwrap_or("")),
            blueprint: None,
            task_progress: None,
        };
    }

    if let Some(intent) = extract_from_file_patterns(changed_files) {
        return IntentInfo {
            intent,
            track: None,
            blueprint: plan_dir.and_then(|p| find_active_blueprint(&p)),
            task_progress: None,
        };
    }

    IntentInfo {
        intent: "unknown".to_string(),
        track: None,
        blueprint: None,
        task_progress: None,
    }
}

fn find_plan_dir(repo: &Path) -> Option<PathBuf> {
    let candidates = [
        repo.join("plan"),
        repo.join("plans"),
        repo.join("docs/plan"),
    ];
    candidates.into_iter().find(|p| p.exists() && p.is_dir())
}

fn find_active_blueprint(plan_dir: &Path) -> Option<PathBuf> {
    for entry in std::fs::read_dir(plan_dir).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("blueprint-") && name.ends_with(".md") {
            return Some(entry.path());
        }
    }
    None
}

#[derive(Debug)]
struct ActiveBoardInfo {
    intent: String,
    blueprint: PathBuf,
    track: Option<String>,
    task_progress: Option<TaskProgress>,
}

fn extract_from_active_board(
    plan_dir: &Path,
    changed_files: &[PathBuf],
) -> Option<ActiveBoardInfo> {
    let active_board_path = plan_dir.join("ACTIVE_BOARD.md");
    if !active_board_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&active_board_path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Path:") {
            let path_str = line.strip_prefix("Path:").unwrap_or("").trim();
            let blueprint_name = path_str.trim_start_matches("plan/");
            let intent = extract_intent_from_blueprint_name(blueprint_name);

            let blueprint_path = if path_str.starts_with("plan/") {
                plan_dir.parent().unwrap_or(plan_dir).join(path_str)
            } else {
                plan_dir.join(path_str)
            };

            // Read blueprint once and extract both track and tasks
            let (track, task_progress) = if let Ok(blueprint_content) =
                std::fs::read_to_string(&blueprint_path)
            {
                let track = extract_track_from_blueprint_content(&blueprint_content, changed_files);
                let progress = count_tasks_from_content(&blueprint_content);
                let task_progress = if progress.total() > 0 {
                    Some(progress)
                } else {
                    None
                };
                (track, task_progress)
            } else {
                (None, None)
            };

            return Some(ActiveBoardInfo {
                intent,
                blueprint: blueprint_path,
                track,
                task_progress,
            });
        }
    }

    None
}

fn extract_intent_from_blueprint_name(name: &str) -> String {
    let name = name.trim_end_matches(".md");

    let name = name.strip_prefix("blueprint-").unwrap_or(name);

    let name = trim_date_version_suffix(name);

    // Trim any trailing version numbers like -100, -v2
    let name = name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-' || c == '_');

    if name.is_empty() {
        "unknown".to_string()
    } else {
        name.to_string()
    }
}

fn trim_date_version_suffix(s: &str) -> String {
    // Pattern: -YYYY-MM-DD at end (date)
    // e.g., verticality-apex-2026-02-26 -> verticality-apex
    let parts: Vec<&str> = s.split('-').collect();
    let n = parts.len();

    if n >= 4 {
        // Date pattern: last 3 parts are MM-DD preceded by YYYY
        let year_idx = n - 3;
        let month_idx = n - 2;
        let day_idx = n - 1;

        // Check if looks like YYYY-MM-DD pattern
        if parts[year_idx].len() == 4
            && parts[year_idx].chars().all(|c| c.is_ascii_digit())
            && parts[month_idx].len() == 2
            && parts[month_idx].chars().all(|c| c.is_ascii_digit())
            && parts[day_idx].len() == 2
            && parts[day_idx].chars().all(|c| c.is_ascii_digit())
        {
            // Looks like date, remove the YYYY-MM-DD suffix
            return parts[..year_idx].join("-");
        }
    }

    // Pattern: -vNN or -VNN at end (version)
    // e.g., auth-v2 -> auth
    if let Some(pos) = s.rfind(['-', '_']) {
        let after = &s[pos + 1..];
        if after.starts_with('v') || after.starts_with('V') {
            let num = &after[1..];
            if num.chars().all(|c| c.is_ascii_digit()) && !num.is_empty() {
                return s[..pos].to_string();
            }
        }
        // Just -NN at end
        if after.chars().all(|c| c.is_ascii_digit()) && !after.is_empty() {
            return s[..pos].to_string();
        }
    }

    s.to_string()
}

fn extract_track_from_blueprint_content(
    content: &str,
    changed_files: &[PathBuf],
) -> Option<String> {
    let tracks = parse_blueprint_tracks(content);

    if tracks.is_empty() {
        return None;
    }

    for (track_id, track_content) in &tracks {
        for file in changed_files {
            let file_str = file.to_string_lossy();
            for line in track_content.lines() {
                if line.contains(&*file_str) || file_str.contains(line.trim()) {
                    let file_base = file.file_name()?.to_string_lossy();
                    if line.contains(&*file_base) {
                        return Some(track_id.clone());
                    }
                }
            }
        }
    }

    None
}

fn count_tasks_from_content(content: &str) -> TaskProgress {
    let mut progress = TaskProgress::default();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("- [x]") || line.starts_with("- [X]") {
            progress.done += 1;
        } else if line.starts_with("- [~]") {
            progress.in_progress += 1;
        } else if line.starts_with("- [ ]") {
            progress.pending += 1;
        }
    }
    progress
}

fn parse_blueprint_tracks(content: &str) -> Vec<(String, String)> {
    let mut tracks = Vec::new();
    let mut current_track: Option<(String, String)> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(track_id) = parse_track_header(trimmed) {
            if let Some((id, content)) = current_track.take() {
                tracks.push((id, content));
            }
            current_track = Some((track_id, String::new()));
        } else if let Some((_, ref mut content)) = current_track {
            content.push_str(line);
            content.push('\n');
        }
    }

    if let Some((id, content)) = current_track {
        tracks.push((id, content));
    }

    tracks
}

fn parse_track_header(line: &str) -> Option<String> {
    // Only recognize these patterns as track headers
    let patterns: &[(&str, Option<&str>)] = &[
        ("### Track ", Some("V")),
        ("## Track ", Some("V")),
        ("## V", None),
        ("- [ ] V", None),
    ];

    for (prefix, variant_prefix) in patterns {
        if line.starts_with(prefix) {
            let rest = line.strip_prefix(prefix)?;

            for (i, c) in rest.char_indices() {
                if c.is_ascii_digit() {
                    let num_str: String = rest[i..]
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !num_str.is_empty() {
                        match variant_prefix {
                            Some(vp) => return Some(format!("{}{}", vp, num_str)),
                            None => return Some(format!("V{}", num_str)),
                        }
                    }
                }
            }

            if let Some(_vp) = variant_prefix {
                let first_word: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                if !first_word.is_empty() {
                    return Some(first_word);
                }
            }
        }
    }

    None
}

fn extract_from_branch(branch: &str) -> Option<String> {
    let branch = branch.to_ascii_lowercase();

    let common_branches = [
        "master",
        "main",
        "develop",
        "dev",
        "staging",
        "production",
        "prod",
        "release",
        "beta",
        "alpha",
        "trunk",
        "stable",
    ];

    if common_branches.contains(&branch.as_str()) {
        return None;
    }

    let prefixes = [
        "feature/",
        "feat/",
        "fix/",
        "bugfix/",
        "chore/",
        "refactor/",
    ];
    let mut branch_name = branch.as_str();
    let mut matched_prefix = false;
    for prefix in prefixes {
        if branch_name.starts_with(prefix) {
            branch_name = branch_name.strip_prefix(prefix)?;
            matched_prefix = true;
            break;
        }
    }

    if !matched_prefix {
        return None;
    }

    let parts: Vec<&str> = branch_name
        .split(['-', '_', '/'])
        .filter(|s| !s.is_empty())
        .filter(|s| !matches!(*s, "v" | "wip" | "draft" | "temp" | "test"))
        .collect();

    let parts: Vec<&str> = parts
        .into_iter()
        .filter(|s| {
            !(s.starts_with('v') && s.len() > 1 && s[1..].chars().all(|c| c.is_ascii_digit()))
        })
        .take(3)
        .collect();

    let intent = parts.join("-");

    if intent.is_empty() || intent.len() < 2 {
        None
    } else {
        Some(intent)
    }
}

fn extract_track_from_branch(branch: &str) -> Option<String> {
    let branch = branch.to_ascii_lowercase();

    for (i, c) in branch.char_indices() {
        if c == 'v' || c == 'V' {
            let rest = &branch[i + 1..];
            let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !num.is_empty() {
                return Some(format!("V{}", num));
            }
        }
    }

    None
}

fn extract_from_file_patterns(changed_files: &[PathBuf]) -> Option<String> {
    if changed_files.is_empty() {
        return None;
    }

    let mut dir_counts: HashMap<String, usize> = HashMap::new();

    for file in changed_files {
        if let Some(first) = file.components().next() {
            if let Some(dir) = first.as_os_str().to_str() {
                if !dir.starts_with('.') && dir != "Cargo.lock" && dir != "Cargo.toml" {
                    *dir_counts.entry(dir.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    dir_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .filter(|(_, count)| *count >= 1)
        .map(|(dir, _)| dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_intent_from_blueprint_name() {
        assert_eq!(
            extract_intent_from_blueprint_name("blueprint-verticality-apex-2026-02-26.md"),
            "verticality-apex"
        );
        assert_eq!(
            extract_intent_from_blueprint_name("blueprint-operational-100-2026-02-25.md"),
            "operational"
        );
        assert_eq!(
            extract_intent_from_blueprint_name("blueprint-auth-v2.md"),
            "auth"
        );
        assert_eq!(
            extract_intent_from_blueprint_name("blueprint-2026-02-26.md"),
            "unknown"
        );
    }

    #[test]
    fn test_extract_from_branch() {
        assert_eq!(
            extract_from_branch("feature/verticality-apex-v3"),
            Some("verticality-apex".to_string())
        );
        assert_eq!(
            extract_from_branch("fix/auth-bug"),
            Some("auth-bug".to_string())
        );
        assert_eq!(
            extract_from_branch("chore/cleanup-deps"),
            Some("cleanup-deps".to_string())
        );
        assert_eq!(
            extract_from_branch("refactor/parsers-v2"),
            Some("parsers".to_string())
        );
    }

    #[test]
    fn test_extract_from_branch_common_names() {
        assert_eq!(extract_from_branch("master"), None);
        assert_eq!(extract_from_branch("main"), None);
        assert_eq!(extract_from_branch("develop"), None);
        assert_eq!(extract_from_branch("dev"), None);
        assert_eq!(extract_from_branch("staging"), None);
        assert_eq!(extract_from_branch("production"), None);
        assert_eq!(extract_from_branch("release"), None);
    }

    #[test]
    fn test_extract_track_from_branch() {
        assert_eq!(
            extract_track_from_branch("feature/verticality-apex-v3"),
            Some("V3".to_string())
        );
        assert_eq!(
            extract_track_from_branch("fix/auth-v10"),
            Some("V10".to_string())
        );
        assert_eq!(extract_track_from_branch("feature/no-version"), None);
    }

    #[test]
    fn test_parse_track_header() {
        assert_eq!(
            parse_track_header("### Track V1: Some Title"),
            Some("V1".to_string())
        );
        assert_eq!(parse_track_header("## V2: Another"), Some("V2".to_string()));
        assert_eq!(
            parse_track_header("### Track V10: Multi-digit"),
            Some("V10".to_string())
        );
        assert_eq!(
            parse_track_header("- [ ] V3: Task track"),
            Some("V3".to_string())
        );
        // Non-track headers should return None
        assert_eq!(parse_track_header("### Some Other Header"), None);
        assert_eq!(parse_track_header("- [ ] Write tests"), None);
        assert_eq!(parse_track_header("- [x] Done task"), None);
    }

    #[test]
    fn test_extract_from_file_patterns() {
        let files = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            PathBuf::from("tests/test.rs"),
        ];
        assert_eq!(extract_from_file_patterns(&files), Some("src".to_string()));

        let files2 = vec![PathBuf::from("Cargo.toml"), PathBuf::from("Cargo.lock")];
        assert_eq!(extract_from_file_patterns(&files2), None);

        let files3: Vec<PathBuf> = vec![];
        assert_eq!(extract_from_file_patterns(&files3), None);

        let files4 = vec![
            PathBuf::from("host/src/main.rs"),
            PathBuf::from("host/lib.rs"),
        ];
        assert_eq!(
            extract_from_file_patterns(&files4),
            Some("host".to_string())
        );
    }

    #[test]
    fn test_extract_intent_fallback_chain() {
        let result = extract_intent(
            Path::new("/nonexistent/repo"),
            &[PathBuf::from("src/main.rs")],
            Some("master"),
        );
        assert_eq!(result.intent, "src");
        assert_eq!(result.track, None);

        let result2 = extract_intent(
            Path::new("/nonexistent/repo"),
            &[PathBuf::from("src/main.rs")],
            Some("feature/my-feature-v2"),
        );
        assert_eq!(result2.intent, "my-feature");
        assert_eq!(result2.track, Some("V2".to_string()));
    }

    #[test]
    fn test_extract_intent_with_active_board() {
        let temp_dir = TempDir::new().unwrap();
        let plan_dir = temp_dir.path().join("plan");
        fs::create_dir(&plan_dir).unwrap();

        let active_board = plan_dir.join("ACTIVE_BOARD.md");
        let mut file = fs::File::create(&active_board).unwrap();
        writeln!(file, "# Active Board").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "Path: plan/blueprint-test-feature-2026-02-27.md").unwrap();

        let blueprint = plan_dir.join("blueprint-test-feature-2026-02-27.md");
        let mut file = fs::File::create(&blueprint).unwrap();
        writeln!(file, "# Test Feature").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "### Track V1: Implementation").unwrap();
        writeln!(file, "- [x] Setup project").unwrap();
        writeln!(file, "- [~] Add feature").unwrap();
        writeln!(file, "- [ ] Write tests").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "Files: src/feature.rs, src/lib.rs").unwrap();

        let changed_files = vec![PathBuf::from("src/feature.rs")];
        let result = extract_intent(temp_dir.path(), &changed_files, Some("main"));

        assert_eq!(result.intent, "test-feature");
        assert_eq!(result.track, Some("V1".to_string()));
        assert!(result.blueprint.is_some());
        assert!(result.task_progress.is_some());

        let progress = result.task_progress.unwrap();
        assert_eq!(progress.done, 1);
        assert_eq!(progress.in_progress, 1);
        assert_eq!(progress.pending, 1);
    }

    #[test]
    fn test_extract_intent_without_plan() {
        let temp_dir = TempDir::new().unwrap();
        let changed_files = vec![PathBuf::from("src/main.rs")];

        let result = extract_intent(temp_dir.path(), &changed_files, Some("main"));

        assert_eq!(result.intent, "src");
        assert!(result.track.is_none());
        assert!(result.blueprint.is_none());
        assert!(result.task_progress.is_none());
    }

    #[test]
    fn test_extract_intent_with_branch_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let changed_files = vec![PathBuf::from("src/main.rs")];

        let result = extract_intent(
            temp_dir.path(),
            &changed_files,
            Some("feature/auth-system-v5"),
        );

        assert_eq!(result.intent, "auth-system");
        assert_eq!(result.track, Some("V5".to_string()));
    }

    #[test]
    fn test_extract_intent_blueprint_path_variants() {
        let temp_dir = TempDir::new().unwrap();
        let plan_dir = temp_dir.path().join("plan");
        fs::create_dir(&plan_dir).unwrap();

        // Test with "plan/" prefix in Path
        let active_board = plan_dir.join("ACTIVE_BOARD.md");
        let mut file = fs::File::create(&active_board).unwrap();
        writeln!(file, "Path: plan/blueprint-with-prefix.md").unwrap();

        let blueprint = plan_dir.join("blueprint-with-prefix.md");
        fs::File::create(&blueprint).unwrap();

        let result = extract_intent(temp_dir.path(), &[], Some("main"));
        assert_eq!(result.intent, "with-prefix");

        // Test without "plan/" prefix in Path
        let mut file = fs::File::create(&active_board).unwrap();
        writeln!(file, "Path: blueprint-without-prefix.md").unwrap();

        let blueprint2 = plan_dir.join("blueprint-without-prefix.md");
        fs::File::create(&blueprint2).unwrap();

        let result2 = extract_intent(temp_dir.path(), &[], Some("main"));
        assert_eq!(result2.intent, "without-prefix");
    }
}
