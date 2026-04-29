# Project State

## Current Focus
Add new encrypted secret files and update tracking rules for secret handling in the terminal engine.

## Completed
- [x] Add `owner_nixos.pub` key file to the dracon data directory for encrypted secret management.
- [x] Extend `.gitattributes` to mark additional secret‑related patterns (bash/zsh/sh history, netrc, terraform lock/vars, known_hosts, credentials, vault.yml, etc.) for dracon filtering, diffing, and merging.
- [x] Update `.gitignore` to ensure these encrypted secret files are not ignored, preserving them in version control for team collaboration.
