# dracon-git

Git operations with libgit2 and automatic CLI fallback for robustness.

## Usage

```rust
use dracon_git::GitService;

let git = GitService::new("/path/to/repo")?;

// Status
let status = git.get_status().await?;
println!("Clean: {}", status.is_clean);

// Pull with rebase
git.pull_rebase().await?;

// Commit with semantic message
git.commit_all("feat: add login").await?;
```

## Key Types

- [`GitService`] — async git operations (status, pull, push, commit, add)
- [`extract_intent`] — parses branch names and task boards for commit intent
- [`build_commit_message`] — generates conventional-commit messages

## Feature Flags

None required — libgit2 with automatic CLI fallback for binary file handling.

## License

MIT OR Apache-2.0
