# Dracon-libs Actionable Audit Backlog

Date: 2026-06-12
Sources:
- `.pi/audit-report-20260611.md` — comprehensive read-only workspace audit.
- `.pi/consumer-api-report-20260612.md` — external Rust consumer/API audit.

Purpose: prioritize concrete follow-up work from the audits without starting implementation. This backlog is intentionally read-only planning output.

## Priority scale

- **P0 / Security-critical** — should be handled before broader cleanup because it can cause unsafe command execution, secret exposure, or protected-file leakage.
- **P1 / High consumer or reliability impact** — blocks downstream users, causes broken docs/examples, or creates near-term breakage risk.
- **P2 / Medium maintainability impact** — improves reliability, ergonomics, or dependency health.
- **P3 / Low/deferred polish** — useful cleanup after higher-priority items.

Effort: **S**mall, **M**edium, **L**arge.

## Executive sequencing

1. **First quick wins (docs/examples/features/lints):** fix non-compiling README/rustdoc examples, feature docs, video README scope, clippy warnings, and license text mismatches.
2. **First security hardening:** gate/remove raw remote shell API, then fix `SystemAgent` approval/execution semantics and Git hook/filter behavior.
3. **Medium-term hardening:** AI secret/TLS cleanup, memory privacy controls, media unsafe/failure handling, and targeted security tests.
4. **Larger maintenance tracks:** dependency refresh batches, semver/public API policy, and ergonomic constructors/builders.

## Prioritization rationale

Items are ranked first by security impact, then by consumer/API breakage risk, then by maintainability value. Quick wins are separated from larger projects so a follow-up implementation goal can choose a small safe slice without losing visibility into the larger remediation roadmap.

## Quick wins backlog

| ID | Priority | Area / owner | Item | Impact | Effort | Evidence | Suggested remediation |
|---|---:|---|---|---|---:|---|---|
| QW-01 | P1 | Docs / consumer API | Fix non-compiling README and rustdoc examples in TTS, STT, system, and video crates. | Prevents downstream users from copying examples that do not compile. | S | TTS README uses `KittenTTS::new().await?` but `KittenTTS::new(model_path, voices_path)` requires args (`.pi/consumer-api-report-20260612.md`; `tools/media/dracon-tts-runtime/src/kitten.rs:176`). STT README uses async calls for sync `ParakeetStt::new`/`transcribe` (`tools/media/dracon-stt-runtime/src/parakeet.rs:17`; `tools/media/dracon-stt-runtime/src/stt_contracts.rs`). System README uses `SystemSnapshotProvider::new()?` but `new()` returns `Self` (`tools/system/dracon-system/src/lib.rs:78`). Video README imports `FfmpegVideoProcessor`/`WhisperTranscriptProcessor` but runtime impls are absent (`tools/media/dracon-video-runtime/src/lib.rs:30`). | Correct examples, add compile-checked examples where practical, and fix broken rustdoc links for `WhisperStt` and `install_package`. |
| QW-02 | P1 | Cargo features / docs | Align README feature/default-feature claims with `Cargo.toml`. | Removes ambiguity about dependencies and feature-gated APIs. | S | TTS README says `kitten` default but Cargo default is `["kitten", "kokoro"]` (`tools/media/dracon-tts-runtime/Cargo.toml:11-13`). Video README says `ffmpeg` default but Cargo default is empty (`tools/media/dracon-video-runtime/Cargo.toml:34-36`). System README mentions `notify` feature but no `[features]` exists (`tools/system/dracon-system/Cargo.toml`). | Update README feature tables, or intentionally change feature defaults only in a major release. |
| QW-03 | P1 | Video crate / docs | Clarify that `dracon-video-runtime` currently exposes protocol/exporter utilities, not runtime implementations. | Prevents consumers from choosing the crate for missing FFmpeg/Whisper processors. | S | Video README lists `FfmpegVideoProcessor`, `WhisperTranscriptProcessor`, and `AutoReframeProcessor`; `src/lib.rs` says runtime implementations are not present (`tools/media/dracon-video-runtime/README.md`; `tools/media/dracon-video-runtime/src/lib.rs:30`). | Rewrite README to say “protocol + exporters; runtime implementations not included yet” and add roadmap notes if implementations are planned. |
| QW-04 | P2 | Lint / CI readiness | Fix existing clippy warnings and then add a lint gate. | Improves maintainability and makes future cleanup safer. | S | Comprehensive audit: `cargo clippy --workspace --exclude dracon-tts-runtime --exclude dracon-stt-runtime --exclude dracon-video-runtime --all-targets -- -D warnings` failed on `tools/sync/dracon-git/src/intent.rs:562,568,573`; example clippy failed on `tools/media/dracon-tts-runtime/examples/kitten_voice_test.rs:12`. | Replace empty `writeln!` calls with `writeln!(file)` or `file.write_all(b"\n")`; replace example `vec!` with arrays; add CI lint gate after warnings are fixed. |
| QW-05 | P2 | Workspace docs / compliance | Standardize crate README license text to match AGPL-3.0-only package/workspace metadata. | Reduces legal/compliance confusion for downstream consumers. | S | Workspace license is AGPL-3.0-only (`Cargo.toml:26`); several tool READMEs advertise MIT or MIT OR Apache-2.0 (`.pi/audit-report-20260611.md`; `rg -n 'MIT|Apache|AGPL' README.md tools services contracts -g 'README.md'`). | Update README license sections to AGPL-3.0-only unless licensing is intentionally changed in a separate approved goal. |
| QW-06 | P2 | AI crates / docs | Add concise READMEs for `ai-service`, `ai-runtime-config`, `ai-runtime-adapters`, and `ai-routing-runtime`. | Improves discoverability for external Rust consumers. | S/M | Consumer audit found no README files for AI service/config/adapter/routing crates (`.pi/consumer-api-report-20260612.md`). | Add usage snippets, feature flags, configuration expectations, and warnings about fake API keys. |
| QW-07 | P2 | Security / remote execution | Gate or remove deprecated raw remote shell API. | Reduces arbitrary remote command execution surface. | S/M | `RemoteExecContract::run_command()` remains public and executes raw remote shell commands (`tools/system/dracon-system/src/contracts.rs:222-228`; `.pi/audit-report-20260611.md`). | Remove, gate behind explicit unsafe/opt-in feature, or add compile-time/runtime guardrails and tests. |

## High-priority implementation backlog

| ID | Priority | Area / owner | Item | Impact | Effort | Evidence | Suggested remediation |
|---|---:|---|---|---|---:|---|---|
| SEC-01 | P0 | `dracon-system` / security | Fix `SystemAgent` approval/execution semantics and privileged operation gating. | Prevents approved command bypass via `PATH`, unapproved privileged mutations, and false success on non-zero exit. | M | `approve_command()` stores command string but execution uses `Command::new(command)` via `PATH`; `apply_config()` and `install_package()` mutate system state without approval; `run_command_checked()` returns stdout even on non-zero exit (`tools/system/dracon-system/src/lib.rs:172,190-223,272,281,296`). Also `ProcessController::kill_process()` proceeds when UID is `None` (`tools/system/dracon-system/src/monitor.rs:232-251`). | Resolve approved commands to absolute paths at approval time and re-check at execution time; route `apply_config()` and `install_package()` through approval/confirmation; check exit status and stderr; deny process control when UID cannot be verified. |
| SEC-02 | P0 | `dracon-git` / security | Fix Git hook suppression, path operand handling, and fail-closed protected-file filtering. | Prevents hook execution during protected workflows and prevents accidental plaintext staging. | M | `GIT_HOOKS_PATH` is not standard hook suppression; `git commit -m` fallback may execute hooks; `git add -f` fallback lacks `--`; `run_clean_filter()` silently falls back to plaintext on `dracon-warden` failure (`tools/sync/dracon-git/src/cli.rs:9-18`; `tools/sync/dracon-git/src/lib.rs:648`; `.pi/audit-report-20260611.md`). | Use `git -c core.hooksPath=/dev/null` plus `--no-verify`; validate/canonicalize repo root; use `--` before paths; parse `-z` output; fail closed if clean filter cannot run. |
| SEC-03 | P1 | AI crates / security | Align AI crates to rustls-only `reqwest` and add secret redaction/validation. | Reduces secret/TLS policy drift and accidental credential logging. | M | Workspace `reqwest` uses rustls-only features, but AI service/adapters declare direct `reqwest` without disabling default features; `GenericOpenAIAdapter` stores raw `String` API key with no redaction or validation (`Cargo.toml:45`; `services/crates/ai/ai-service/Cargo.toml:15-16`; `services/crates/ai/ai-runtime-adapters/Cargo.toml:11-15`; `.pi/audit-report-20260611.md`). | Use workspace `reqwest`; add URL/model/header validation; redact API keys in `Debug`; consider secret wrapper or zeroization policy. |
| SEC-04 | P1 | Memory / privacy | Add memory privacy controls and retention/deletion APIs. | Addresses plaintext SQLite storage and lack of per-record deletion. | M/L | SQLite stores conversations/facts in plaintext; no documented encryption-at-rest, permissions, backup, retention, or per-record deletion; `clear()` deletes all conversations and vector rows (`tools/memory/dracon-memory-runtime/src/db.rs:20-28,52-86,167`). | Document privacy assumptions; add OS permission guidance or encrypted storage option; add per-conversation/per-fact deletion; clarify fallback embedding stability. |
| SEC-05 | P1 | Media / reliability | Surface external media tool failures instead of silent fallback. | Prevents hidden environment misconfiguration and degraded output. | S/M | TTS `espeak-ng` spawn/write/status failures are ignored and raw text is used as phonemes (`tools/media/dracon-tts-runtime/src/kitten.rs:488-499`; `tools/media/dracon-tts-runtime/src/kokoro.rs:509-519`). | Return `Result` from phoneme/token conversion paths; surface missing binary errors; add fake `espeak-ng` integration tests. |
| SEC-06 | P2 | Media / unsafe code | Tighten unsafe/thread-safety invariants in STT/TTS. | Reduces risk from broad unsafe assumptions and malformed model/voice files. | M | Unsafe `Send/Sync` for `ParakeetStt`; Whisper mmap loading; Kokoro voice loading assumes `file_size % 4 == 0` without check (`tools/media/dracon-stt-runtime/src/parakeet.rs:181-182`; `tools/media/dracon-stt-runtime/src/whisper.rs:63`; `tools/media/dracon-tts-runtime/src/kokoro.rs:267-269`). | Narrow unsafe blocks; add explicit checks; validate model/voice files at construction; re-evaluate manual `Send/Sync`. |

## Consumer/API backlog

| ID | Priority | Area / owner | Item | Impact | Effort | Evidence | Suggested remediation |
|---|---:|---|---|---|---:|---|---|
| API-01 | P1 | Public API policy / semver | Define and enforce a semver/public API policy for public structs/enums/traits. | Prevents accidental breaking changes in minor releases. | M | No `#[non_exhaustive]` found on public structs/enums (`rg -n "non_exhaustive|^pub (struct|enum)" tools services contracts`). Adding fields to structs or variants to enums is breaking for downstream struct literals/matches. | For next major release, add `#[non_exhaustive]` where appropriate; until then avoid adding fields/variants in minor releases; prefer constructors/builders. |
| API-02 | P1 | Contract/runtime alignment | Clarify and align contract crates with runtime APIs. | Prevents consumers from depending on contract APIs that runtime crates do not mirror. | M | Memory contract `MemoryStore::store_conversation(&Conversation)` differs from runtime `MemorySystem::store_conversation(role, content)`; `TextEmbedder::embed(&self)` differs from `OnnxEmbedder::embed(&mut self)`; `ParakeetStt` implements timestamp trait while reporting no timestamp support (`contracts/crates/dracon-memory-contracts/src/lib.rs`; `tools/memory/dracon-memory-runtime/src/lib.rs`; `tools/memory/dracon-memory-runtime/src/embedder.rs:75`; `.pi/consumer-api-report-20260612.md`). | Add contract-vs-runtime docs; align APIs or explicitly document divergence; rename/gate misleading timestamp trait implementation. |
| API-03 | P1 | Feature/API ergonomics | Add ergonomic constructors/builders for common runtime/config surfaces. | Makes downstream setup easier and reduces per-crate convention learning. | M | TTS/STT constructors are inconsistent; AI config exposes raw public fields; `GenericOpenAIAdapter::new_with_auth()` accepts raw strings without validation (`tools/media/dracon-tts-runtime/src/kitten.rs`; `tools/media/dracon-stt-runtime/src/parakeet.rs`; `services/crates/ai/ai-runtime-config/src/lib.rs`; `services/crates/ai/ai-runtime-adapters/src/lib.rs:28-40`). | Add builders/validated constructors for TTS engines, AI config, AI adapter, and common system/memory entry points. |
| API-04 | P2 | AI provider contract | Clarify streaming behavior in AI contracts/adapters. | Prevents consumers from expecting real streaming from a non-streaming adapter. | S/M | `ChatRequest::stream` exists, but generic adapter returns collected string/finish reason, not a stream (`contracts/crates/ai/dracon-ai-runtime-contracts/src/models.rs:14-28`; `services/crates/ai/ai-runtime-adapters/src/lib.rs:64-104`). | Rename/document streaming behavior; add separate streaming trait/method if real streaming is intended. |
| API-05 | P2 | Error/result ergonomics | Standardize fallible APIs that currently return `Option` or silent fallbacks. | Improves diagnosability for downstream consumers. | M | `ParakeetStt::transcribe_audio` returns `Option<String>`; media tool failures silently fall back (`tools/media/dracon-stt-runtime/src/parakeet.rs`; `.pi/audit-report-20260611.md`). | Prefer `Result` where error context matters; document intentional `Option` APIs; add error-context tests. |

## Dependency/workspace backlog

| ID | Priority | Area / owner | Item | Impact | Effort | Evidence | Suggested remediation |
|---|---:|---|---|---|---:|---|---|
| DEP-01 | P2 | Workspace dependencies / ML stack | Create a dependency refresh plan by crate, starting with high-impact ML/tokenizer crates. | Reduces maintenance burden and future compatibility risk. | L | `cargo outdated` reported many outdated dependencies: `sysinfo`, `candle-*`, `parakeet-rs`, `tokenizers`, `ort`, `zip`, `rubato`, `rodio`, `reqwest`, `tokio`; `cargo tree -d` showed duplicate `tokenizers`, `reqwest`, `ort`, and ML stack families (`.pi/audit-report-20260611.md`). | Refresh in small batches; run `cargo audit`, `cargo outdated`, `cargo tree -d`, and workspace checks after each batch. |
| DEP-02 | P2 | Workspace dependencies / TLS | Consolidate `reqwest` feature policy across workspace crates. | Ensures consistent rustls-only TLS posture. | S/M | Workspace `reqwest` uses rustls-only features, but AI service/adapters use direct dependencies without disabling default features (`Cargo.toml:45`; `services/crates/ai/ai-service/Cargo.toml:15-16`; `services/crates/ai/ai-runtime-adapters/Cargo.toml:11-15`). | Move AI crates to workspace dependency or explicitly set rustls-only features; verify with `cargo tree`. |
| DEP-03 | P3 | Workspace dependencies / unmaintained crates | Address unmaintained `number_prefix` and `paste`. | Reduces long-tail dependency risk. | M | `cargo audit --no-fetch --stale` reported unmaintained `number_prefix 0.4.0` via `indicatif -> tokenizers -> dracon-memory-runtime` and `paste 1.0.15` via ML/tokenizer/image stack. | Refresh transitive dependency chains where feasible; verify no regressions after each update. |

## Code quality and test coverage backlog

| ID | Priority | Area / owner | Item | Impact | Effort | Evidence | Suggested remediation |
|---|---:|---|---|---|---:|---|---|---|
| TEST-01 | P1 | Security tests | Add targeted deny/fail-closed tests for `SystemAgent`, `ProcessController`, Git hooks, and protected-file filtering. | Ensures security-sensitive fixes remain effective. | M | Comprehensive audit found limited negative tests for command approval/PATH, process UID denial, remote shell API usage, Git hook behavior, and protected-file filter failure. | Add unit/integration tests for PATH-stable approved command execution, UID `None` denial, hook suppression, and `dracon-warden` failure fail-closed behavior. |
| TEST-02 | P2 | Media integration tests | Add fake external binary tests for `espeak-ng` and `dracon-warden`. | Catches environment misconfiguration and silent fallback regressions. | M | `espeak-ng` failures are ignored; `dracon-warden` failure silently falls back to plaintext. | Use fake binaries on `PATH` to assert errors/fail-closed behavior instead of silent fallback. |
| TEST-03 | P2 | Docs/API examples | Add doctest or compile-checked example coverage for public README examples. | Prevents docs regressions. | S/M | Several README/rustdoc examples did not match real APIs. | Add compile-checked examples where practical; mark external-service examples as `ignore` intentionally. |

## File evidence index

Every backlog item includes a `Evidence` column with concrete file paths, function/type names, line references where available, or completed-report references. The most important evidence anchors are:

- `tools/system/dracon-system/src/lib.rs` and `tools/system/dracon-system/src/monitor.rs` for `SystemAgent` approval, privileged operations, and process-control risks.
- `tools/sync/dracon-git/src/cli.rs` and `tools/sync/dracon-git/src/lib.rs` for Git hook suppression, path operand handling, and protected-file filtering.
- `tools/media/dracon-tts-runtime/src/kitten.rs`, `tools/media/dracon-tts-runtime/src/kokoro.rs`, `tools/media/dracon-stt-runtime/src/parakeet.rs`, and `tools/media/dracon-stt-runtime/src/stt_contracts.rs` for media API/docs and unsafe/failure-handling findings.
- `tools/media/dracon-video-runtime/README.md` and `tools/media/dracon-video-runtime/src/lib.rs` for the video runtime scope mismatch.
- `Cargo.toml`, `tools/media/dracon-tts-runtime/Cargo.toml`, `tools/media/dracon-video-runtime/Cargo.toml`, and `tools/system/dracon-system/Cargo.toml` for feature/default-feature evidence.
- `.pi/audit-report-20260611.md` and `.pi/consumer-api-report-20260612.md` as the source audit reports.

## Larger projects and deferred work

| ID | Priority | Area / owner | Item | Why deferred | Evidence | Suggested next step |
|---|---:|---|---|---|---|---|
| PROJ-01 | P1 | Public API design | Add constructors/builders and ergonomic APIs for TTS/STT/AI config/system/memory surfaces. | Requires API design decisions and may be breaking if constructors/signatures change. | Consumer audit findings on TTS/STT async/sync confusion, AI config public fields, and `GenericOpenAIAdapter` raw-string constructor. | Start with non-breaking convenience constructors/builders; save signature changes for major release. |
| PROJ-02 | P1 | Memory privacy | Provide encrypted storage or documented secure-storage integration. | Larger product/security decision; may require dependencies and migration design. | Memory runtime stores plaintext SQLite and lacks retention/per-record deletion. | Define privacy requirements first, then implement encrypted backend or OS permission guidance. |
| PROJ-03 | P2 | Dependency health | Refresh ML/tokenizer dependency families and reduce duplicates. | High blast radius; should be batched and verified incrementally. | `cargo outdated` and `cargo tree -d` show many outdated and duplicate dependencies. | Create crate-by-crate refresh plan and run checks after each batch. |
| PROJ-04 | P2 | Video runtime scope | Decide whether `dracon-video-runtime` should add runtime implementations or remain protocol/exporter-only. | Product scope decision, not just code cleanup. | README promises runtime implementations but crate only exports protocol/exporter utilities. | Confirm roadmap before implementing `FfmpegVideoProcessor`, `WhisperTranscriptProcessor`, or `AutoReframeProcessor`. |
| PROJ-05 | P3 | API naming | Consider crate/package naming consistency. | Mostly discoverability; changing crate names is breaking. | Package `dracon-system-lib` maps to crate `dracon_system_lib`, unlike more natural crate names elsewhere. | Document current mapping or plan major-release renames only if justified. |

## Recommended first implementation goal candidates

If the next goal should implement something, these are the best candidates in order:

1. **Fix consumer-facing docs/examples and feature documentation**
   - Includes `QW-01`, `QW-02`, `QW-03`, `QW-05`, and `QW-06`.
   - Good first because it is high impact, readably scoped, and low implementation risk.

2. **Harden command execution and Git protected workflows**
   - Includes `SEC-01`, `SEC-02`, and `TEST-01`.
   - Highest security value, but needs careful tests and approval from maintainers because it changes behavior.

3. **Clean AI secret/TLS handling**
   - Includes `SEC-03`, `DEP-02`, and `API-04`.
   - Important for external AI consumers; likely small-to-medium scope if limited to redaction, validation, and feature alignment.

4. **Fix clippy warnings and add lint gate**
   - Includes `QW-04`.
   - Fast maintainability win before larger refactors.

5. **Memory privacy and retention**
   - Includes `SEC-04` and `PROJ-02`.
   - Important, but should follow explicit privacy requirements.

## Read-only boundary note

This backlog intentionally does not modify source, manifests, lockfiles, CI, or READMEs. It is a planning artifact derived from the completed audits.
