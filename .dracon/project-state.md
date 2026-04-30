# Project State

## Current Focus
Add CI/CD pipelines, documentation, release automation, bump widget count, fix test assertions, and update README.

## Completed
- [x] Added Infrastructure section to CHANGELOG with CI/CD workflows, release steps, issue/PR templates, CODE_OF_CONDUCT, CONTRIBUTING, and GitHub topics
- [x] Updated Cargo.toml to reflect new version metadata and binary size changes
- [x] Updated README badge license link from `LICENSE` to `LICENSE-MIT` and corrected widget count from 28 to 29
- [x] Adjusted example command descriptions to show 29 framework widgets instead of 28
- [x] Fixed scroll test assertion to verify `content_height == 0` exactly
- [x] Corrected gutter indexing calculations in `text_editor_adapter_test.rs`
- [x] Regenerated Cargo.lock after adding new dependencies for CI/CD and clippy configuration
- [x] Added Clippy and rustfmt configuration files
- [x] Added GitHub Actions workflows for CI (stable/beta/nightly, macOS/Windows) and release pipelines
- [x] Added issue templates (bug report, feature request) and PR template
- [x] Added CODE_OF_CONDUCT.md (Contributor Covenant v2.0) and CONTRIBUTING.md documentation
- [x] Updated project topics, description, and homepage on GitHub
- [x] Updated README quick‑start example to underscore unused tick parameter
- [x] Updated README widget count references from “23+” to “29”
- [x] Updated README example description wording to reflect current widget count
- [x] Applied Clippy fixes for zero‑multiplication warnings and always‑true assertions across widget files
