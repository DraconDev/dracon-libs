# Dracon-libs Actionable Audit Verification

Date: 2026-06-12
Goal: audit the project and verify `.pi/actionable-audit-backlog-20260612.md` against fresh repository evidence.
Boundary: read-only audit and validation. No source, manifest, lockfile, CI, or README changes were made.

## Executive result

The actionable backlog is still a valid prioritization artifact. Its items remain mostly confirmed by current code and fresh validation commands. No backlog item is fully closed. Two evidence references are stale enough to update before implementation work starts:

1. `QW-04` clippy evidence should be refreshed: current all-targets clippy failures are in `dracon-system-lib` tests and media test module placement, not `dracon-git/src/intent.rs`.
2. `SEC-02` should distinguish `run_clean_filter()` from `add_paths()`: `run_clean_filter()` now returns an error, but `add_paths()` still silently falls back to plaintext when the clean filter fails.

Implementation was not performed because the user task requested audit/checking and the backlog explicitly says it is read-only planning output. Security and API-hardening fixes require separate implementation goals, tests, and maintainer/product decisions.

## Fresh validation evidence

| Check | Command | Result | Evidence |
|---|---|---:|---|
| Workspace inventory | `cargo metadata --locked --format-version=1 --no-deps` | Pass | 14 workspace packages |
| Formatting | `cargo fmt --all -- --check` | Pass | Exit 0 |
| CI clippy command | `cargo clippy --workspace -- -D warnings` | Pass | Exit 0 |
| Stricter all-target clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Fail | Exit 101; 4 `useless_vec` in `tools/system/dracon-system/src/lib.rs`; 2 `items_after_test_module` in `kitten.rs`/`kokoro.rs` |
| Type/build check | `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo check --workspace --all-targets --all-features'` | Pass | Exit 0 |
| Workspace tests | `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo test --workspace --all-targets --no-fail-fast'` | Pass | Exit 0 |
| Docs default | `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo doc --workspace --no-deps'` | Pass with warnings | 2 rustdoc broken links: `install_package`, `WhisperStt` |
| Docs strict | `nix-shell -p pkg-config alsa-lib sqlite --run 'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps'` | Fail | Exit 101 for the two broken links |
| Security advisories | `cargo audit --no-fetch --stale` | Pass with warnings | `number_prefix 0.4.0` and `paste 1.0.15` unmaintained |
| Dependency freshness | `cargo outdated --root-deps-only --exit-code 1` | Fail as expected | Many outdated root deps: `sysinfo`, `candle-*`, `parakeet-rs`, `tokenizers`, `ort`, `zip`, `rubato`, `rodio`, `reqwest`, `tokio` |
| Duplicate deps | `cargo tree -d --locked --prefix none` | Pass with duplicates | `tokenizers` 0.19.1/0.22.2, `reqwest` feature split, `ort`, ML/tokenizer families |
| Working tree | `git status --porcelain=v1 -uall` | Clean | No source/report changes from this audit |

## Backlog item status

Legend: **Confirmed** = still present in current repo; **Partially confirmed** = issue still present but evidence/details need updating; **Not implemented** = no code fix was made.

### Quick wins backlog

| ID | Status | Fresh finding |
|---|---|---|
| QW-01 | Confirmed | README/rustdoc examples still mismatch APIs: TTS README calls `KittenTTS::new().await?`; STT README awaits sync APIs; system README uses `SystemSnapshotProvider::new()?`; video README imports missing runtime processors. Strict rustdoc links fail for `WhisperStt` and `install_package`. |
| QW-02 | Confirmed | TTS default is `["kitten", "kokoro"]` but README says only `kitten`; video default is empty but README says `ffmpeg` default; system README mentions `notify` but `Cargo.toml` has no `[features]`. |
| QW-03 | Confirmed | `dracon-video-runtime/README.md` and crate docs still advertise `FfmpegVideoProcessor`, `WhisperTranscriptProcessor`, and `AutoReframeProcessor`, while `src/lib.rs` says runtime implementations are not present. |
| QW-04 | Partially confirmed | Clippy is still not clean under all-target `-D warnings`, but current failures are `tools/system/dracon-system/src/lib.rs:323,332,336,342` and `tools/media/dracon-tts-runtime/src/kitten.rs:1066`, `kokoro.rs:893`. CI clippy without `--all-targets` passes. |
| QW-05 | Confirmed | READMEs still advertise MIT/Apache licenses while workspace/package metadata is `AGPL-3.0-only`. |
| QW-06 | Confirmed | AI crates still have no README files: `ai-service`, `ai-routing-runtime`, `ai-runtime-config`, `ai-runtime-adapters`. |
| QW-07 | Confirmed | `RemoteExecContract::run_command()` remains public and deprecated, and `SshRemoteExecProvider::run_command()` still executes raw remote shell commands. |

### High-priority implementation backlog

| ID | Status | Fresh finding |
|---|---|---|
| SEC-01 | Confirmed | `SystemAgent::approve_command()` stores strings but `run_command_checked()` resolves via `Command::new(command)` at execution time; `apply_config()` and `install_package()` mutate system state without approval; non-zero command status is ignored; `ProcessController::kill_process()` proceeds when UID cannot be verified (`None`). |
| SEC-02 | Partially confirmed | Hook suppression still relies on non-standard `GIT_HOOKS_PATH`; `git commit -m` fallback can execute hooks; `git add -f` fallback lacks `--`; `resolve_concrete_paths()` parses line-based output; protected-file clean-filter failure is still fail-open because `add_paths()` uses `unwrap_or_else(|_| plaintext.clone())`. Update: `run_clean_filter()` itself returns `Result`, so the stale evidence should point to the fallback in `add_paths()`. |
| SEC-03 | Confirmed | `ai-service` and `ai-runtime-adapters` still declare direct `reqwest` dependencies with default features, while workspace `reqwest` is rustls-only; `GenericOpenAIAdapter` stores raw `api_key: String` and has no redaction/validation. |
| SEC-04 | Confirmed | SQLite stores conversations/facts in plaintext; no per-record deletion/retention API; `clear()` deletes all conversations and vector rows; fallback embedding uses `DefaultHasher` without documented stability guarantees. |
| SEC-05 | Confirmed | Kitten and Kokoro spawn `espeak-ng`, ignore spawn/write/status/read failures, and silently use raw text as phonemes. |
| SEC-06 | Confirmed | Unsafe/thread-safety invariants remain broad: manual `Send/Sync` for `ParakeetStt`, Whisper mmap safety relies on caller guarantees, Kokoro voice loading does not check `file_size % 4 == 0`. |

### Consumer/API backlog

| ID | Status | Fresh finding |
|---|---|---|
| API-01 | Confirmed | `rg -n "non_exhaustive|^pub (struct|enum)" tools services contracts` found public structs/enums and no `#[non_exhaustive]`. |
| API-02 | Confirmed | Memory contract `MemoryStore::store_conversation(&Conversation)` differs from runtime `MemorySystem::store_conversation(role, content)`; contract `TextEmbedder::embed(&self)` differs from runtime `OnnxEmbedder::embed(&mut self)`; `ParakeetStt` implements `TimestampedTranscription` while `supports_timestamps: false`. |
| API-03 | Confirmed | TTS/STT constructors remain inconsistent; AI config exposes public fields; `GenericOpenAIAdapter::new_with_auth()` accepts raw strings without validation. |
| API-04 | Confirmed | `ChatRequest::stream` exists, but `GenericOpenAIAdapter` returns collected content; `SmartRouter::route_with_trace()` ignores project, preferred model, messages, and constraints. |
| API-05 | Confirmed | `ParakeetStt::transcribe_audio()` returns `Option<String>`; media tool failures silently fall back; Git protected-file filter failure falls back to plaintext. |

### Dependency/workspace backlog

| ID | Status | Fresh finding |
|---|---|---|
| DEP-01 | Confirmed | `cargo outdated --root-deps-only --exit-code 1` reports many outdated root deps across system, media, memory, and AI crates. |
| DEP-02 | Confirmed | `cargo tree -d` shows `reqwest` feature split; AI crates still use direct `reqwest` with default features. |
| DEP-03 | Confirmed | `cargo audit --no-fetch --stale` reports unmaintained `number_prefix 0.4.0` and `paste 1.0.15`. |

### Code quality and test coverage backlog

| ID | Status | Fresh finding |
|---|---|---|
| TEST-01 | Confirmed | Existing tests pass under nix, but negative coverage for PATH-stable approved command execution, UID `None` denial, hook suppression, and protected-file filter failure is still limited. |
| TEST-02 | Confirmed | No fake binary tests were found for `espeak-ng` or `dracon-warden`; `rg` only found production spawn sites. |
| TEST-03 | Confirmed | Strict rustdoc fails; README examples remain non-compiling/misleading. |

### Larger projects and deferred work

| ID | Status | Fresh finding |
|---|---|---|
| PROJ-01 | Confirmed | Constructor/builder ergonomics still need API design decisions before implementation; current TTS/STT/AI config/system/memory surfaces remain inconsistent. |
| PROJ-02 | Confirmed | Memory privacy/encrypted storage remains a product/security decision; current runtime still uses plaintext SQLite and lacks retention/per-record deletion. |
| PROJ-03 | Confirmed | Dependency refresh remains high-blast-radius; `cargo outdated` and `cargo tree -d` still show many outdated/duplicate ML/tokenizer families. |
| PROJ-04 | Confirmed | Video runtime scope still needs product roadmap confirmation before adding `FfmpegVideoProcessor`, `WhisperTranscriptProcessor`, or `AutoReframeProcessor`. |
| PROJ-05 | Confirmed | Crate/package naming inconsistency remains mostly discoverability work; changing `dracon-system-lib` / `dracon_system_lib` would be breaking. |

### Recommended first implementation goal candidates

| Candidate | Status | Fresh finding |
|---|---|---|
| Docs/examples/features/license/AI READMEs | Confirmed | Still the lowest-risk first implementation slice. |
| Command execution + Git protected workflows | Confirmed | Still the highest security-value slice, but behavior-changing and test-heavy. |
| AI secret/TLS cleanup | Confirmed | Still small-to-medium if limited to redaction, validation, and feature alignment. |
| Clippy cleanup + lint gate | Confirmed | Current all-target clippy blockers are concrete and low-risk. |
| Memory privacy/retention | Confirmed | Still requires explicit privacy requirements before implementation. |

## Recommended next implementation slices

1. **Fix consumer-facing docs/examples and feature documentation**: QW-01, QW-02, QW-03, QW-05, QW-06. This is the lowest-risk high-impact slice.
2. **Fix clippy all-target warnings and add strict lint gate**: QW-04, TEST-03. Current blockers are system test `vec!` literals and media test module placement.
3. **Harden command execution and Git protected workflows**: SEC-01, SEC-02, TEST-01. Requires careful tests because behavior changes are security-sensitive.
4. **Clean AI secret/TLS handling**: SEC-03, DEP-02, API-04.
5. **Address memory privacy, media failure handling, and unsafe invariants**: SEC-04, SEC-05, SEC-06, TEST-02.
6. **Plan dependency refresh in small verified batches**: DEP-01, DEP-03.

## Completion audit for this goal

- Read and audited `.pi/actionable-audit-backlog-20260612.md` completely.
- Cross-checked every backlog item against current files and fresh command output.
- Verified workspace metadata, formatting, clippy, check, tests, docs, dependency health, and working tree state.
- Produced this findings/action-plan report.
- Did not modify source, manifests, lockfiles, CI, or READMEs.
- No backlog item is marked complete without fresh evidence; stale evidence is explicitly called out for QW-04 and SEC-02.
