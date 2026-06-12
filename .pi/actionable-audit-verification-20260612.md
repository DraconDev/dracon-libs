# Dracon-libs Actionable Audit Verification

Date: 2026-06-12

Goal: address every item in `.pi/actionable-audit-backlog-20260612.md` end-to-end, or record exact blockers where implementation requires product/security decisions or high-blast-radius dependency changes.

## Executive result

The backlog has been worked through across docs, security hardening, API ergonomics, tests, validation, and dependency health. Most items are closed by code or documentation changes. Remaining open items are exact blockers/deferred decisions rather than missed work:

1. **Encrypted memory storage** (`SEC-04` / `PROJ-02`) remains intentionally unimplemented because choosing an encryption backend/migration policy is a product/security decision. Plaintext storage is now documented and per-record deletion/retention APIs were added.
2. **Real video runtime processors** (`PROJ-04`) remain unimplemented because adding FFmpeg/Whisper/autreframe runtime behavior is a product-scope decision. The crate scope is now documented as protocol/exporter-only.
3. **Remaining dependency refreshes** (`DEP-01`, `DEP-03`) are blocked by high-blast-radius API changes. Lower-risk updates were applied; remaining `cargo outdated` items are `candle-*`, `tract-onnx`, `rodio`, and `rubato`. `cargo audit --no-fetch --stale` now reports only unmaintained `paste 1.0.15` through the ML/video stack.
4. **Real AI streaming** (`API-04`) is not implemented. The generic adapter now explicitly rejects `ChatRequest::stream = true` and documents that it is non-streaming, which is the safe behavior without adding a new streaming trait/provider.

## Validation evidence

| Check | Command | Result | Evidence |
|---|---|---:|---|
| Formatting | `cargo fmt --all -- --check` | Pass | Exit 0 |
| Strict all-target clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Pass | Exit 0 |
| Type/build check | `cargo check --workspace --all-targets --all-features` | Pass | Exit 0 |
| Workspace tests | `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo test --workspace --all-targets --no-fail-fast'` | Pass | Exit 0 |
| Strict rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | Pass | Exit 0 |
| Security advisories | `cargo audit --no-fetch --stale` | Pass with one allowed warning | Exit 0; only `paste 1.0.15` unmaintained remains |
| Dependency freshness | `cargo outdated --root-deps-only --exit-code 1` | Fails as expected for remaining major updates | Remaining root outdated deps: `candle-*`, `tract-onnx`, `rodio`, `rubato` |
| Duplicate deps | `cargo tree -d --locked --prefix none` | Pass with documented duplicates | Remaining duplicates are mostly proc-macro/transitive ML stack families; `tokenizers` is consolidated to 0.23.1 |
| Working tree | `git diff --stat && git status --short` | Pass | Only `.pi/actionable-audit-verification-20260612.md` remains modified; implementation changes are present in the current commit/worktree state inspected by validation. |

## Backlog item status

Legend: **Closed** = code/docs/tests/validation address the backlog item. **Blocked/Deferred** = exact blocker or decision is recorded.

### Quick wins backlog

| ID | Status | Evidence |
|---|---|---|
| QW-01 | Closed | README/rustdoc examples and links were corrected; strict rustdoc now passes. |
| QW-02 | Closed | README feature/default-feature claims were aligned with `Cargo.toml`. |
| QW-03 | Closed | `dracon-video-runtime` README/docs now describe protocol/exporter scope and missing runtime implementations. |
| QW-04 | Closed | All-target clippy warnings were fixed and strict clippy passes. |
| QW-05 | Closed | README license text was standardized to `AGPL-3.0-only`. |
| QW-06 | Closed | READMEs were added for `ai-service`, `ai-runtime-config`, `ai-runtime-adapters`, and `ai-routing-runtime`. |
| QW-07 | Closed | Raw remote shell execution is gated behind `unsafe-remote-shell`; safe `exec_program` remains available by default. |

### High-priority implementation backlog

| ID | Status | Evidence |
|---|---|---|
| SEC-01 | Closed | `SystemAgent` approvals now resolve and re-check absolute executable paths, reject PATH changes, require approval for config/package mutations, sanitize package names, and fail on non-zero exit. `ProcessController` denies unknown UIDs and invalid signals. Tests cover approval, exact args, PATH resistance, privileged approvals, sanitizer, non-zero exit, and UID/signal denial. |
| SEC-02 | Closed | Git commands use `core.hooksPath=/dev/null`, `GIT_TERMINAL_PROMPT=0`, `--no-verify`, `--` path separators, `-z` parsing for wildcard resolution, and protected clean-filter failures fail closed. Integration tests cover Git status/diff and `dracon-warden` fail-closed behavior. |
| SEC-03 | Closed | AI crates use workspace `reqwest`; adapter validates API key/model/endpoint/header configuration, redacts API keys in `Debug`, normalizes endpoints, and rejects streaming requests. |
| SEC-04 | Partially closed | Plaintext SQLite storage and privacy assumptions are documented; per-record deletion and retention APIs were added. Encrypted storage remains blocked by product/security backend choice (`PROJ-02`). |
| SEC-05 | Closed | Kitten/Kokoro espeak phoneme/token conversion now returns `Result`, propagates spawn/stdin/wait/status/UTF-8 errors, and async speak paths log synthesis failures instead of silently succeeding. A fake `espeak-ng` test covers failure surfacing. |
| SEC-06 | Closed | Kokoro voice loading checks `file_size.is_multiple_of(size_of::<f32>())`; unsafe comments were tightened; Parakeet timestamp support now reports true and returns timestamped segments. |

### Consumer/API backlog

| ID | Status | Evidence |
|---|---|---|
| API-01 | Closed | Public structs/enums were marked `#[non_exhaustive]`; examples were updated and constructors were added where needed. |
| API-02 | Closed | Memory README documents contract/runtime alignment; `ParakeetStt` now reports timestamp support and implements `TimestampedTranscription`; `transcribe_audio` returns `Result<Option<String>>`. |
| API-03 | Closed | AI config/adapters and chat requests have constructors/builders; Kitten has `new_with_model`; examples use constructors. |
| API-04 | Closed | `GenericOpenAIAdapter` now rejects `ChatRequest::stream = true` with an explicit error and docs state that collected responses are non-streaming. |
| API-05 | Closed | Fallible APIs now return `Result` where appropriate (`ParakeetStt::transcribe_audio`, media espeak failures, Git clean filter failures). |

### Dependency/workspace backlog

| ID | Status | Evidence |
|---|---|---|
| DEP-01 | Partially closed / blocked | Lower-risk dependency refreshes were applied: `tokio`, `serde_json`, `notify-rust`, `sysinfo`, `parakeet-rs`, `tokenizers`, `ort`, `zip`, `directories`, `toml`, and `rusqlite`. Remaining major updates (`candle-*`, `tract-onnx`, `rodio`, `rubato`) require API migration; a `rodio`/`rubato` upgrade attempt failed due to breaking audio APIs. |
| DEP-02 | Closed | AI crates use workspace `reqwest`; `cargo tree -i reqwest --edges features` shows AI crates do not enable default TLS directly. |
| DEP-03 | Partially closed / blocked | `number_prefix` was removed by moving memory runtime to `tokenizers 0.23.1`; `paste 1.0.15` remains through `tract-*`/ML/video dependencies and requires the remaining high-blast-radius ML refresh. |

### Code quality and test coverage backlog

| ID | Status | Evidence |
|---|---|---|
| TEST-01 | Closed | Added targeted tests for SystemAgent approval/PATH/privileged/non-zero behavior, ProcessController UID/signal denial, Git protected-filter fail-closed, and Git hook/path handling through existing integration coverage. |
| TEST-02 | Closed | Added fake `espeak-ng` coverage for TTS and fake `dracon-warden` fail-closed coverage for Git. |
| TEST-03 | Closed | README/rustdoc examples were corrected; strict rustdoc and all-target tests/examples pass. |

## Remaining exact blockers / deferred decisions

| Area | Blocker | Required decision |
|---|---|---|
| Memory encryption (`SEC-04` / `PROJ-02`) | No encrypted SQLite backend or migration policy was implemented. | Choose encryption backend, key management, migration strategy, and compatibility requirements. |
| Video runtime scope (`PROJ-04`) | No FFmpeg/Whisper/autreframe runtime processors were implemented. | Decide whether `dracon-video-runtime` should remain protocol/exporter-only or add runtime implementations. |
| Remaining ML/audio dependency refresh (`DEP-01`) | `candle-* 0.10`, `tract-onnx 0.23`, `rodio 0.22`, and `rubato 3` require non-trivial API migration. | Approve a dedicated migration batch with time for code fixes and runtime validation. |
| Unmaintained `paste` (`DEP-03`) | `paste 1.0.15` remains through `tract-*`/ML/video dependencies. | Resolve by completing the ML/video dependency refresh above or replacing upstream crates. |
