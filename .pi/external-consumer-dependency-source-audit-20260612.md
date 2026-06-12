# External/local consumer dependency source audit

Date: 2026-06-12

Goal: verify whether other consumers use `dracon-libs` from the repository or from local path overrides, and ensure documented consumers resolve it from the repository source.

## Consumers found

| Consumer | Before | After | Evidence |
|---|---|---|---|
| `/home/dracon/Dev/dracon-utilities` | Local sibling path deps for `dracon-git`, `dracon-system-lib`, `ai-routing-runtime`, `ai-runtime-adapters`, `ai-runtime-config`, `dracon-ai-runtime-contracts` | Git deps to `https://github.com/DraconDev/dracon-libs` | `Cargo.toml` changed; `cargo check --manifest-path ... --workspace` passed and `Cargo.lock` now records `git+https://github.com/DraconDev/dracon-libs#436049dd...`. |
| `/home/dracon/Dev/kiki-sassy-desktop-announcer` | `dracon-tts-runtime = "94.7"` | Unchanged crates.io dependency | `Cargo.toml` and `Cargo.lock` show `registry+https://github.com/rust-lang/crates.io-index`. |
| `/home/dracon/Dev/avid` | Only a commented-out local STT path | Unchanged; no active local `dracon-libs` dependency | `Cargo.toml` has the STT dep commented out; `Cargo.lock` has no active `dracon-stt-runtime`. |
| `/home/dracon/Dev/dracon-platform` | `ai-api` consumes `ai-lib` as a path dep inside the platform repo | Out of scope for `dracon-libs`; it is a different library boundary (`DraconDev/dracon-ai-lib` / `ai-api`) | `apis/services/ai-api/Cargo.toml` documents intentional path dep for in-repo `ai-lib`. |
| Other local repos under `/home/dracon/Dev` | No active `dracon-libs` path/git deps found | Unchanged | Full `Cargo.toml` scan across `/home/dracon/Dev`. |

## Change made

Updated `/home/dracon/Dev/dracon-utilities/Cargo.toml` workspace dependencies:

- `ai-routing-runtime = { git = "https://github.com/DraconDev/dracon-libs" }`
- `ai-runtime-adapters = { git = "https://github.com/DraconDev/dracon-libs" }`
- `ai-runtime-config = { git = "https://github.com/DraconDev/dracon-libs" }`
- `dracon-ai-runtime-contracts = { git = "https://github.com/DraconDev/dracon-libs" }`
- `dracon-git = { git = "https://github.com/DraconDev/dracon-libs" }`
- `dracon-system-lib = { git = "https://github.com/DraconDev/dracon-libs" }`

`cargo check --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace` regenerated `Cargo.lock` and resolved:

- `dracon-git v94.7.0 (https://github.com/DraconDev/dracon-libs#436049dd32b681cccdef37054260fd324e7e32c0)`
- `dracon-system-lib v94.7.0 (https://github.com/DraconDev/dracon-libs#436049dd32b681cccdef37054260fd324e7e32c0)`

## Validation

| Check | Command | Result |
|---|---|---:|
| Consumer manifest scan | `python3` scan of `/home/dracon/Dev/**/Cargo.toml` for `dracon-libs`, `DraconDev/dracon-libs`, and Dracon crate names | Found only the documented consumers. |
| `dracon-utilities` build/type check | `cargo check --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace` | Pass |
| `dracon-utilities` full tests | `cargo test --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace --no-fail-fast` | Failed for unrelated environment/pre-existing issues: missing git author identity in git tests, SSH/mock SSH failures, and security tests reporting no master identities. |
| Current workspace validation | `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`, `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo test --workspace --all-targets --no-fail-fast'` | Pass |

## Takeaway

The only active local sibling consumer of `dracon-libs` was `dracon-utilities`, and it now consumes `dracon-libs` from the Git repository rather than local `../dracon-libs` paths. Other local consumers either use crates.io, use a different AI library boundary, or have only commented/local-development dependencies.

## Remaining notes

Full `dracon-utilities` tests did not pass in this environment, but the failures are unrelated to the dependency-source change: git tests require configured author identity and SSH/mock transport behavior, and security tests report missing master identities. The relevant build/type-check validation for the dependency-source change passed.
