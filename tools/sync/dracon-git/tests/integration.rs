use std::process::Command;
use std::path::Path;
use tempfile::TempDir;

fn setup_repo(path: &Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .status()
        .unwrap();
}

#[test]
fn test_git_service_new_valid_repo() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    let result = dracon_git::GitService::new(tmp.path());
    assert!(result.is_ok());
}

#[test]
fn test_git_service_new_invalid_path() {
    let result = dracon_git::GitService::new(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_git_service_is_repo() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let is_repo = svc.is_git_repo().await.unwrap();
    assert!(is_repo);
}

#[tokio::test]
async fn test_git_service_status_clean() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let status = svc.get_status().await.unwrap();
    assert!(status.is_clean);
}

#[tokio::test]
async fn test_git_service_status_dirty() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();

    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let status = svc.get_status().await.unwrap();
    assert!(!status.is_clean);
}

#[tokio::test]
async fn test_git_service_get_recent_commits() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    std::fs::write(tmp.path().join("a.txt"), "a").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "first"])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    std::fs::write(tmp.path().join("b.txt"), "b").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "second"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let diff_entries = svc.get_diff_entries().await.unwrap();
    assert_eq!(diff_entries.len(), 1);
    assert_eq!(diff_entries[0].path.to_str().unwrap(), "b.txt");
}

#[test]
fn test_commit_message_generation() {
    let ctx = dracon_git::CommitContext::new(
        "add login feature".to_string(),
        vec![],
        false,
    );
    let msg = dracon_git::build_commit_message(&ctx);
    assert!(!msg.is_empty());
}

#[test]
fn test_intent_extraction_from_branch() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    std::fs::write(tmp.path().join("test.txt"), "test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    let intent = dracon_git::extract_intent(tmp.path(), &[], Some("feature/add-login"));
    assert!(!intent.intent.is_empty());
}
