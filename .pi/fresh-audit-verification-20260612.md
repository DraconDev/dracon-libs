# Fresh end-to-end audit verification

Date: 2026-06-12

Goal: double-check the updated `dracon-libs` workspace and documented consumer/dependency-source changes end-to-end.

## Current repository state

- `dracon-libs` HEAD: `b387807`
- Working tree: clean (`git status --porcelain=v1 -uall` returned no paths after validation and report commit).
- Recent audited changes are committed in the current history:
  - `b387807` — fresh end-to-end audit verification report.
  - `957c6af` — updated external consumer dependency-source audit report.
  - `4c4ebdc` — updated README external/local consumer takeaway.
  - `436049d` — updated local AI consumer takeaway.
  - `035c02d` — AI consumer/API changes and verification report.
  - `4398b92` — actionable audit verification report.

## Manifest and dependency-source audit

### `dracon-libs` workspace

Evidence:

- `cargo metadata --no-deps --format-version 1` found 14 workspace packages, all version `94.7.0` and all source `None` (workspace-local).
- `cargo metadata --format-version 1` found only expected internal path dependencies inside the workspace:
  - `ai-routing-runtime -> dracon-ai-contracts`
  - `ai-routing-runtime -> dracon-ai-runtime-contracts`
  - `ai-runtime-adapters -> dracon-ai-runtime-contracts`
  - `ai-service -> ai-routing-runtime`
  - `ai-service -> ai-runtime-adapters`
  - `ai-service -> dracon-ai-contracts`
  - `ai-service -> dracon-ai-runtime-contracts`
  - `dracon-ai-runtime-contracts -> dracon-ai-contracts`
- No external path dependencies were found from `dracon-libs` packages.

### Local AI consumers

Evidence: `cargo metadata --no-deps --format-version 1` plus reverse `cargo tree -i` checks.

| Local consumer | Consumes | Evidence |
|---|---|---|
| `dracon-ai-runtime-contracts` | `dracon-ai-contracts` | Internal workspace path dependency and reverse tree. |
| `ai-routing-runtime` | `dracon-ai-contracts`, `dracon-ai-runtime-contracts` | Internal workspace path dependencies and reverse tree. |
| `ai-runtime-adapters` | `dracon-ai-runtime-contracts` | Internal workspace path dependency and reverse tree. |
| `ai-service` | `ai-routing-runtime`, `ai-runtime-adapters`, `dracon-ai-contracts`, `dracon-ai-runtime-contracts` | Internal workspace path dependencies and reverse tree. |
| `ai-runtime-config` | none | Reverse `cargo tree -i ai-runtime-config --workspace --prefix none` showed no dependents. |

No non-AI workspace package depends on the AI crates.

### External/local consumer scan

Evidence: Python scan of `/home/dracon/Dev/**/Cargo.toml` for `dracon-libs`, `DraconDev/dracon-libs`, and Dracon crate names.

| Consumer | Current source | Result |
|---|---|---|
| `/home/dracon/Dev/dracon-utilities` | Git deps to `https://github.com/DraconDev/dracon-libs` | No active `../dracon-libs` path deps remain. |
| `/home/dracon/Dev/kiki-sassy-desktop-announcer` | `dracon-tts-runtime = "94.7"` | `cargo metadata` reports `registry+https://github.com/rust-lang/crates.io-index`. |
| `/home/dracon/Dev/avid` | STT dep is commented out | `cargo metadata` found no active `dracon-stt-runtime`. |
| `/home/dracon/Dev/dracon-platform/apis/services/ai-api` | `ai-lib = { path = "../../../../dracon-ai-lib/crates/ai-lib" }` | Different AI library boundary (`DraconDev/dracon-ai-lib`), not `dracon-libs`. |
| `/home/dracon/Dev/dracon-code` and `/home/dracon/Dev/dracon-ai-lib` | Local workspace/internal deps | Separate `dracon-code` / `dracon-ai-lib` boundary; no active `dracon-libs` path deps found. |
| Other `/home/dracon/Dev` manifests | No active `dracon-libs` path/git deps | Scan found no additional `dracon-libs` consumers. |

### `dracon-utilities` verification

Evidence:

- `rg -n "dracon-libs|DraconDev/dracon-libs|path\\s*=\\s*\"\\.\\./dracon-libs" /home/dracon/Dev/dracon-utilities -g Cargo.toml` found only six Git deps and no local path deps.
- Current `Cargo.toml` declares:
  - `ai-routing-runtime = { git = "https://github.com/DraconDev/dracon-libs" }`
  - `ai-runtime-adapters = { git = "https://github.com/DraconDev/dracon-libs" }`
  - `ai-runtime-config = { git = "https://github.com/DraconDev/dracon-libs" }`
  - `dracon-ai-runtime-contracts = { git = "https://github.com/DraconDev/dracon-libs" }`
  - `dracon-git = { git = "https://github.com/DraconDev/dracon-libs" }`
  - `dracon-system-lib = { git = "https://github.com/DraconDev/dracon-libs" }`
- `cargo metadata --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --format-version 1` reports used deps:
  - `dracon-sync -> dracon-git source=git+https://github.com/DraconDev/dracon-libs path=None`
  - `dracon-system -> dracon-system-lib source=git+https://github.com/DraconDev/dracon-libs path=None`
- `cargo tree --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace -i dracon-git --prefix none` reports `dracon-git v94.7.0 (https://github.com/DraconDev/dracon-libs#436049dd)`.
- `cargo tree --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace -i dracon-system-lib --prefix none` reports `dracon-system-lib v94.7.0 (https://github.com/DraconDev/dracon-libs#436049dd)`.
- `cargo check --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace` passed.
- `cargo check --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace --locked` passed.

Remote sync evidence for `dracon-utilities`:

- Local HEAD: `b66ac345040ce6d4609504ed9f0753c5f22151f6`.
- Codeberg `main`: `b66ac345040ce6d4609504ed9f0753c5f22151f6`.
- GitLab `main`: `b66ac345040ce6d4609504ed9f0753c5f22151f6`.
- GitHub `main`: `9ca1a5804ee7091d3d7fd1f4a00569cb7c0323e6`, divergent from local. After `git fetch github main`, local is 8 commits ahead and GitHub is 7 commits ahead of their merge base. Pushing this manifest change to GitHub would require integrating remote changes or an explicit force-with-lease decision.

## Documentation and report audit

Inspected:

- `README.md`
- `docs/ai-lib-placement.md`
- `.pi/actionable-audit-verification-20260612.md`
- `.pi/ai-consumer-impact-verification-20260612.md`
- `.pi/local-ai-consumer-takeaway-20260612.md`
- `.pi/external-consumer-dependency-source-audit-20260612.md`

Findings:

- README documents the local AI consumer takeaway and the `dracon-utilities` migration from local sibling paths to Git repo dependencies.
- `docs/ai-lib-placement.md` documents that `dracon-ai-lib` remains standalone and direct BYOK consumers should use `ai-lib = { git = "https://github.com/DraconDev/dracon-ai-lib", tag = "v0.2.0" }` or local path overrides only for local development.
- `.pi` reports document exact blockers/deferred decisions rather than hiding them:
  - encrypted memory storage backend/key policy,
  - video runtime implementation scope,
  - high-blast-radius ML/audio dependency migrations,
  - unmaintained `paste` through the ML/video dependency chain,
  - GitHub mirror sync for `dracon-utilities` if that mirror must receive the manifest change.

## Hygiene checks

- `rg -n "TODO|FIXME|unwrap\\(|expect\\(|panic!"` over the changed Rust/docs/report files found no matches.
- `git status --porcelain=v1 -uall` returned clean after validation.
- `git diff --stat` returned no output.

## Validation evidence

| Check | Command | Result |
|---|---|---:|
| Formatting | `nix-shell -p pkg-config alsa-lib sqlite bash --run 'cargo fmt --all -- --check'` | Pass, exit 0 |
| Type/build check | `nix-shell -p pkg-config alsa-lib sqlite bash --run 'cargo check --workspace --all-targets --all-features'` | Pass, exit 0 |
| Strict clippy | `nix-shell -p pkg-config alsa-lib sqlite bash --run 'cargo clippy --workspace --all-targets -- -D warnings'` | Pass, exit 0 |
| Strict rustdoc | `nix-shell -p pkg-config alsa-lib sqlite bash --run 'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps'` | Pass, exit 0 |
| Workspace tests | `nix-shell -p pkg-config alsa-lib sqlite bash --run 'cargo test --workspace --all-targets --no-fail-fast'` | Pass, exit 0 |
| Consumer build/type check | `cargo check --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace` | Pass, exit 0 |
| Consumer locked build/type check | `cargo check --manifest-path /home/dracon/Dev/dracon-utilities/Cargo.toml --workspace --locked` | Pass, exit 0 |
| Security advisories | `cargo audit --no-fetch --stale` | Pass with one allowed warning: unmaintained `paste 1.0.15` |
| Dependency freshness | `cargo outdated --root-deps-only --exit-code 1` | Expected failure for remaining major updates: `rodio`, `rubato`, `candle-core`, `candle-nn`, `candle-transformers`, `tract-onnx` |
| Duplicate deps | `cargo tree -d --locked --prefix none` | Pass, exit 0; known transitive duplicates remain in ML/video stack |

## Completion audit mapping

| Goal requirement | Evidence |
|---|---|
| Updated `dracon-libs` workspace is audited | Fresh status, metadata, manifest scan, docs/reports inspected. |
| Every local/path/git dependency involving `dracon-libs` or documented consumers is re-scanned | Python manifest scan, `cargo metadata`, `cargo tree`, `rg` checks. |
| `dracon-utilities` consumes `dracon-libs` from repo, not local path overrides | `Cargo.toml` has Git deps; metadata/tree show Git sources; no `../dracon-libs` paths; locked check passes. |
| Local AI consumers and external/local consumers re-mapped | `cargo metadata`, reverse `cargo tree`, README/report evidence. |
| No unapproved shortcuts/TODOs/dead code/hidden assumptions remain in changed files | `rg` for TODO/FIXME/unwrap/expect/panic found no matches; docs record known blockers. |
| Required validation run | All required commands passed except advisory/freshness checks with documented expected warnings/failures. |
| Exact blockers reported | Memory encryption, video runtime scope, high-blast-radius ML/audio refresh, unmaintained `paste`, and GitHub mirror sync decision are documented. |

## Final takeaway

The fresh audit confirms the dependency-source fix is present and verified: the active local sibling consumer (`dracon-utilities`) no longer uses local `../dracon-libs` paths for `dracon-libs` crates and resolves the used `dracon-git`/`dracon-system-lib` dependencies from `https://github.com/DraconDev/dracon-libs`. Other local consumers are either crates.io consumers, separate AI library boundaries, or commented/local-development-only cases. The only remaining decision is whether the divergent GitHub mirror of `dracon-utilities` must also receive the manifest change via integration or an explicit force-with-lease push.
