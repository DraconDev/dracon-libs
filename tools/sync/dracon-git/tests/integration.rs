use std::path::Path;
use std::process::Command;
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
fn test_git_service_new_accepts_any_path() {
    let tmp = TempDir::new().unwrap();
    let result = dracon_git::GitService::new(tmp.path());
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_git_service_is_repo_invalid() {
    let tmp = TempDir::new().unwrap();
    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let is_repo = svc.is_git_repo().await.unwrap();
    assert!(!is_repo);
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
        .args(["commit", "--no-verify", "-m", "initial"])
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
async fn test_git_service_protected_filter_failure_is_fail_closed() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    std::fs::write(tmp.path().join(".gitattributes"), "*.txt filter=dracon\n").unwrap();
    std::fs::write(tmp.path().join("secret.txt"), "plaintext").unwrap();

    let bin_dir = tmp.path().join("bin");
    std::fs::create_dir(&bin_dir).unwrap();
    let warden = bin_dir.join("dracon-warden");
    std::fs::write(&warden, "#!/bin/sh\necho fake warden failure >&2\nexit 1\n").unwrap();
    let mut perms = std::fs::metadata(&warden).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&warden, perms).unwrap();

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let combined_path = format!("{}:{}", bin_dir.display(), old_path.to_string_lossy());
    std::env::set_var("PATH", combined_path);
    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let err = svc.add_paths(&["secret.txt".to_string()]).await;
    match old_path.is_empty() {
        true => std::env::remove_var("PATH"),
        false => std::env::set_var("PATH", old_path),
    }

    assert!(err.is_err());
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&status.stdout);
    assert!(stdout.contains("secret.txt"));
}

#[tokio::test]
async fn test_git_service_get_diff_entries() {
    let tmp = TempDir::new().unwrap();
    setup_repo(tmp.path());
    std::fs::write(tmp.path().join("a.txt"), "a").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .status()
        .unwrap();
    std::fs::write(tmp.path().join("b.txt"), "b").unwrap();

    let svc = dracon_git::GitService::new(tmp.path()).unwrap();
    let diff_entries = svc.get_diff_entries().await.unwrap();
    assert!(!diff_entries.is_empty());
    assert!(diff_entries
        .iter()
        .any(|e| e.path.to_str() == Some("b.txt")));
}

#[test]
fn test_commit_message_generation() {
    let ctx = dracon_git::CommitContext::new("add login feature".to_string(), vec![], false);
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
