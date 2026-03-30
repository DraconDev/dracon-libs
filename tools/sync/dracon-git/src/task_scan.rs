use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct TaskProgress {
    pub done: usize,
    pub in_progress: usize,
    pub pending: usize,
}

impl TaskProgress {
    pub fn total(&self) -> usize {
        self.done + self.in_progress + self.pending
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }

    pub fn summary(&self) -> String {
        let total = self.total();
        if total == 0 {
            return String::new();
        }
        if self.in_progress > 0 {
            format!(
                "{}/{} done ({} in progress)",
                self.done, total, self.in_progress
            )
        } else {
            format!("{}/{} done", self.done, total)
        }
    }
}

pub fn scan_blueprint_tasks(blueprint_path: &Path) -> TaskProgress {
    let mut progress = TaskProgress::default();
    if let Ok(content) = fs::read_to_string(blueprint_path) {
        count_tasks_in_content(&content, &mut progress);
    }
    progress
}

fn count_tasks_in_content(content: &str, progress: &mut TaskProgress) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_progress_summary() {
        let progress = TaskProgress {
            done: 3,
            in_progress: 1,
            pending: 2,
        };
        assert_eq!(progress.summary(), "3/6 done (1 in progress)");

        let progress2 = TaskProgress {
            done: 3,
            in_progress: 0,
            pending: 2,
        };
        assert_eq!(progress2.summary(), "3/5 done");
    }

    #[test]
    fn test_scan_blueprint_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let blueprint_path = temp_dir.path().join("blueprint-test.md");

        let content = r#"# Test Blueprint

### Track V1: Initial Setup
- [x] Setup project
- [x] Add dependencies
- [~] Configure environment

### Track V2: Core Features
- [ ] Implement feature A
- [ ] Implement feature B
"#;

        let mut file = std::fs::File::create(&blueprint_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let progress = scan_blueprint_tasks(&blueprint_path);
        assert_eq!(progress.done, 2);
        assert_eq!(progress.in_progress, 1);
        assert_eq!(progress.pending, 2);
        assert_eq!(progress.total(), 5);
    }

    #[test]
    fn test_scan_blueprint_tasks_empty() {
        let temp_dir = TempDir::new().unwrap();
        let blueprint_path = temp_dir.path().join("blueprint-empty.md");

        let content = r#"# Empty Blueprint

No tasks here.
"#;

        let mut file = std::fs::File::create(&blueprint_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let progress = scan_blueprint_tasks(&blueprint_path);
        assert_eq!(progress.total(), 0);
        assert!(progress.is_empty());
    }

    #[test]
    fn test_task_progress_default() {
        let progress = TaskProgress::default();
        assert_eq!(progress.done, 0);
        assert_eq!(progress.pending, 0);
        assert_eq!(progress.in_progress, 0);
        assert!(progress.is_empty());
    }

    #[test]
    fn test_count_tasks_case_insensitive() {
        let content = "- [x] Task 1\n- [X] Task 2\n- [ ] Task 3";
        let mut progress = TaskProgress::default();
        count_tasks_in_content(content, &mut progress);
        assert_eq!(progress.done, 2);
        assert_eq!(progress.pending, 1);
    }
}
