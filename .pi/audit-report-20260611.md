# Dracon-libs Read-only Workspace Audit Report

Date: 2026-06-11
Scope: `dracon-libs` Rust workspace audit for security posture, code quality/maintainability, dependency/Cargo health, and documentation/public API completeness.
Boundary: read-only audit; no implementation fixes, dependency changes, CI changes, or destructive operations were made.

## Executive summary

The workspace is broadly buildable and testable on this NixOS host when required system libraries are available, but several security and maintainability risks should be prioritized. The highest-risk issues are concentrated in command execution and process control surfaces (`dracon-system`), Git fallback behavior (`dracon-git`), and media/ML code that silently falls back or ignores external process failures. Dependency health is acceptable from a known-vulnerability perspective (`cargo audit` found no vulnerability failures), but there are unmaintained transitive crates, many outdated dependencies, duplicate ML/tokenizer dependency versions, and inconsistent `reqwest` TLS features.

## Verification commands and outcomes

- `cargo metadata --locked --format-version=1 --no-deps`: 14 workspace members, lockfile-compatible metadata.
- `wc -l Cargo.lock && test -f Cargo.lock`: `Cargo.lock` exists and has 6604 lines.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --exclude dracon-tts-runtime --exclude dracon-stt-runtime --exclude dracon-video-runtime --all-targets --target-dir /tmp/dracon-libs-audit-check-nonmedia`: passed.
- `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo check --workspace --all-targets --target-dir /tmp/dracon-libs-audit-check-full'`: passed.
- `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo check --workspace --all-targets --all-features --target-dir /tmp/dracon-libs-audit-check-allfeatures'`: passed.
- `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo test --workspace --all-targets --target-dir /tmp/dracon-libs-audit-test --no-fail-fast'`: passed.
- `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo doc --workspace --no-deps --target-dir /tmp/dracon-libs-audit-doc-default'`: passed with 2 rustdoc warnings:
  - broken link to `install_package` in `tools/system/dracon-system/src/lib.rs`.
  - broken link to `WhisperStt` in `tools/media/dracon-stt-runtime/src/lib.rs`.
- `cargo clippy --workspace --exclude dracon-tts-runtime --exclude dracon-stt-runtime --exclude dracon-video-runtime --all-targets --target-dir /tmp/dracon-libs-audit-clippy-lib-nonmedia -- -D warnings`: failed on `tools/sync/dracon-git/src/intent.rs` `writeln!(file, "")`.
- `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo clippy --workspace --examples --target-dir /tmp/dracon-libs-audit-clippy-examples-nix -- -D warnings'`: failed on `tools/media/dracon-tts-runtime/examples/kitten_voice_test.rs` `useless_vec`.
- `cargo audit --no-fetch --stale`: no vulnerability failures; 2 unmaintained warnings:
  - `number_prefix 0.4.0` via `indicatif -> tokenizers -> dracon-memory-runtime`.
  - `paste 1.0.15` via ML/tokenizer/image stack.
- `cargo outdated --root-deps-only --exit-code 1`: many outdated dependencies, including `sysinfo`, `candle-*`, `parakeet-rs`, `tokenizers`, `ort`, `zip`, `rubato`, `rodio`, `reqwest`, and `tokio`.
- `cargo tree -d --locked --prefix none`: duplicate dependency families observed, including `tokenizers` 0.19.1/0.22.2, `reqwest` feature split, `ort`, and ML stack duplicates.
- `git status --short`: only `?? .pi/`, confirming audit commands did not modify workspace source files.

## Prioritized findings

### P1 — `SystemAgent` approval model can be bypassed or over-trusted

Severity: High
Affected paths:
- `tools/system/dracon-system/src/lib.rs:172`, `190-223`, `272`, `281`, `296`
- `tools/system/dracon-system/src/monitor.rs:8-17`, `232-251`

Impact:
- `approve_command()` records the exact program string and args, but `run_command_checked()` later calls `Command::new(command)`, which resolves the executable via `PATH` at execution time. An approved `"foo"` command could execute a different binary if `PATH` changes or is attacker-controlled.
- `apply_config()` runs `home-manager switch` and `install_package()` runs `nix profile install` without going through the approval gate, even though both are privileged local mutation operations.
- `run_command_checked()` returns stdout even if the command exits non-zero, so callers may treat failed privileged commands as successful.
- `ProcessController::kill_process()` only blocks when UID differs. If `get_process_uid()` returns `None` because `/proc/{pid}/status` is unreadable, missing, or unparseable, the kill proceeds. Signal values are not range-validated.

Recommendations:
- Resolve approved commands to an absolute path at approval time and re-check the same executable path at execution time.
- Route `apply_config()` and `install_package()` through an explicit approval/confirmation API or mark them unsafe with the same approval contract.
- Check command exit status and include stderr in errors.
- Treat inability to verify UID as deny-by-default for process control; validate signal ranges before invoking `kill`.

### P2 — Git hook suppression and protected-file filtering are not reliable

Severity: High
Affected paths:
- `tools/sync/dracon-git/src/cli.rs:9-18`
- `tools/sync/dracon-git/src/lib.rs:73`, `184`, `234`, `390`, `403`, `426`, `439`, `461`, `470-477`, `531`, `567`, `648`, `693`

Impact:
- `GIT_HOOKS_PATH` is not a standard Git mechanism for disabling hooks. Git operations that mutate state, especially `git commit -m` fallback at `src/lib.rs:648`, may still execute hooks.
- `add_paths()` validates concrete paths after resolving globs, but `GitService::new()` does not canonicalize/validate the repo root. CLI fallback uses `git add -f` without `--`, so unusual filenames beginning with `-` can be interpreted as options.
- `resolve_concrete_paths()` parses non-`-z` Git output, so filenames containing newlines can be mishandled.
- `run_clean_filter()` silently falls back to plaintext when `dracon-warden` is missing or fails (`unwrap_or_else(|_| plaintext.clone())`), undermining encryption-at-rest guarantees for protected files.

Recommendations:
- Disable hooks with supported mechanisms such as `git -c core.hooksPath=/dev/null` and `git commit --no-verify`, or explicitly document hook behavior.
- Canonicalize and validate the repo root in `GitService::new()` or before each mutating operation.
- Use `--` before path operands in CLI fallbacks and `-z`/NUL-safe parsing for file lists.
- Fail closed for protected files if the clean filter cannot run; never stage plaintext as a silent fallback.

### P3 — Remote execution exposes a deprecated raw shell API

Severity: High
Affected paths:
- `tools/system/dracon-system/src/contracts.rs:222-228`
- `tools/system/dracon-system/src/remote.rs:139`, `178`

Impact:
- `RemoteExecContract::run_command()` remains public and executes arbitrary remote shell commands. It is deprecated, but deprecation does not prevent use.
- `exec_program()` shell-escapes program/args before SSH `exec`, which is safer, but still invokes a remote shell.

Recommendations:
- Remove or gate the raw shell API behind an explicit unsafe/opt-in feature.
- Prefer structured remote execution and document that `exec_program()` still uses remote shell quoting rather than a native argument vector.
- Add tests or compile-time lints ensuring new code paths use `exec_program()`.

### P4 — Memory runtime stores sensitive conversation/fact data without documented privacy controls

Severity: Medium-High
Affected paths:
- `tools/memory/dracon-memory-runtime/src/db.rs:20-28`, `52-86`, `167`
- `tools/memory/dracon-memory-runtime/src/embedder.rs:75`, `222`

Impact:
- SQLite stores conversations and user facts in plaintext. The crate currently has no documented encryption-at-rest, file permission, backup, retention, or per-record deletion guidance.
- `clear()` deletes all conversations and vector rows, but there is no targeted delete API for individual conversations or facts.
- The deterministic fallback embedder uses `DefaultHasher`; Rust does not guarantee long-term hash stability across versions/processes. Persisted fallback embeddings may not be stable enough for durable semantic recall.

Recommendations:
- Document privacy assumptions and provide OS file permission guidance or encrypted storage options.
- Add retention and per-record deletion APIs.
- Make fallback embedding stability explicit; if fallback vectors are persisted, use a stable hash or mark them non-durable.

### P5 — AI runtime stores API keys in plaintext and uses inconsistent TLS features

Severity: Medium-High
Affected paths:
- `Cargo.toml:45`
- `services/crates/ai/ai-service/Cargo.toml:15-16`
- `services/crates/ai/ai-runtime-adapters/Cargo.toml:11-15`
- `services/crates/ai/ai-runtime-adapters/src/lib.rs:28-40`, `91`

Impact:
- Workspace `reqwest` defaults to `default-features = false` with `rustls-tls`, but AI service/adapters declare direct `reqwest` dependencies without disabling default features. This can pull native-tls/OpenSSL and diverge from workspace TLS policy.
- `GenericOpenAIAdapter` stores `api_key: String` and sends it as an auth header. There is no redaction, secret lifecycle, or zeroization policy.
- `auth_header_prefix` can be empty, and endpoint/model/auth header names are not validated.

Recommendations:
- Move AI crates to workspace `reqwest = { workspace = true }` with rustls-only features.
- Document that API keys are held in memory and logs/debug output must redact them.
- Validate endpoint URL scheme/host, non-empty model/header names, and non-empty prefix when required.
- Consider a secret wrapper type with `Debug` redaction.

### P6 — Unsafe/thread-safety assumptions need narrower invariants

Severity: Medium
Affected paths:
- `tools/media/dracon-stt-runtime/src/parakeet.rs:181-182`
- `tools/media/dracon-stt-runtime/src/whisper.rs:63`
- `tools/media/dracon-tts-runtime/src/kokoro.rs:267-269`

Impact:
- `unsafe impl Send/Sync for ParakeetStt` relies on external `Parakeet` thread-safety beyond the visible `Arc<Mutex<_>>`.
- Whisper mmap loading and Kokoro voice loading use unsafe operations with broad safety comments.
- Kokoro `load_voice()` computes `num_floats = file_size / 4` and then claims `file_size` is exactly `num_floats * 4`; if the file has a trailing non-multiple-of-4 byte tail, it is silently ignored.

Recommendations:
- Narrow unsafe blocks and require explicit checks, e.g. `file_size % 4 == 0`.
- Add tests or construction-time validation for model/voice files.
- Re-evaluate manual `Send/Sync` by wrapping only thread-safe handles or documenting exact upstream guarantees.

### P7 — External media tools are spawned but failures are ignored

Severity: Medium
Affected paths:
- `tools/media/dracon-tts-runtime/src/kitten.rs:488-499`
- `tools/media/dracon-tts-runtime/src/kokoro.rs:509-519`

Impact:
- `espeak-ng` spawn/write/status failures are ignored. If the binary is missing, stdin write fails, or phoneme extraction fails, code silently falls back to using raw text as phonemes.
- This can cause degraded output and hides environment misconfiguration.

Recommendations:
- Return `Result` from phoneme/token conversion paths where possible.
- Surface missing external binary errors at construction or first use.
- Add integration tests with a fake `espeak-ng` on `PATH`.

### P8 — Dependency health has no known vulnerability failures but needs maintenance

Severity: Medium
Affected paths:
- `Cargo.lock`
- `tools/memory/dracon-memory-runtime/Cargo.toml`
- `tools/media/dracon-tts-runtime/Cargo.toml`
- `tools/media/dracon-stt-runtime/Cargo.toml`
- `tools/media/dracon-video-runtime/Cargo.toml`

Impact:
- `cargo audit` reports unmaintained `number_prefix` and `paste`.
- `cargo outdated` reports many outdated dependencies, including `sysinfo 0.32.1 -> 0.39.3`, `candle-* 0.9.2 -> 0.10.2`, `parakeet-rs 0.3.3 -> 0.3.6`, `tokenizers 0.22.2/0.19.1 -> 0.23.1`, `ort 2.0.0-rc.11 -> 2.0.0-rc.12`, `zip 2.4.2 -> 8.6.0`, `rubato 0.14.1 -> 3.0.0`, and `reqwest 0.12.28 -> 0.13.4`.
- `cargo tree -d` shows duplicate dependency families and feature splits, especially around tokenizers, ML crates, and `reqwest`.

Recommendations:
- Create a dependency refresh plan by crate, starting with high-impact ML/tokenizer crates.
- Consolidate workspace dependencies where possible.
- Re-run `cargo audit`, `cargo outdated`, and `cargo tree -d` after each batch update.
- Prefer rustls-only `reqwest` consistently across workspace crates.

### P9 — Clippy is not clean under `-D warnings`

Severity: Medium
Affected paths:
- `tools/sync/dracon-git/src/intent.rs:562`, `568`, `573`
- `tools/media/dracon-tts-runtime/examples/kitten_voice_test.rs:12`

Impact:
- Non-media clippy fails on empty `writeln!(file, "")` in tests.
- Example clippy fails on `vec!` where an array would suffice.

Recommendations:
- Replace empty `writeln!` calls with `writeln!(file)` or `file.write_all(b"\n")`.
- Replace example `vec!` literals with arrays when no `Vec` is required.
- Add a CI lint gate once these existing warnings are fixed.

### P10 — Documentation and README examples are inconsistent with public APIs

Severity: Medium
Affected paths:
- `tools/media/dracon-tts-runtime/README.md:12`, `38`
- `tools/media/dracon-tts-runtime/src/lib.rs:14-15`
- `tools/media/dracon-stt-runtime/README.md:15`
- `tools/media/dracon-stt-runtime/src/parakeet.rs:17`
- `tools/system/dracon-system/README.md:11`
- `tools/system/dracon-system/src/lib.rs:78`
- `tools/media/dracon-video-runtime/README.md`
- `tools/media/dracon-video-runtime/src/lib.rs:30`
- `tools/files/dracon-files/README.md:22`
- `tools/media/dracon-stt-runtime/README.md:32`
- `tools/media/dracon-tts-runtime/README.md:38`
- `tools/memory/dracon-memory-runtime/README.md:36`
- `tools/sync/dracon-git/README.md:35`
- `tools/system/dracon-system/README.md:35`
- `Cargo.toml:26`

Impact:
- TTS README uses `KittenTTS::new().await?`, but actual `KittenTTS::new(model_path, voices_path)` requires two arguments. The crate-level example also has `. await?` spacing and calls `speak(...).await` even though trait `speak` is synchronous.
- STT README calls `ParakeetStt::new(model_path).await?` and `transcribe(...).await?`, but `ParakeetStt::new` and `transcribe` are synchronous.
- System README calls `SystemSnapshotProvider::new()?`, but `new()` returns `Self`.
- Video README claims runtime implementations such as `FfmpegVideoProcessor` and `WhisperTranscriptProcessor`, while `src/lib.rs` says runtime implementations are not present.
- Most crate READMEs advertise MIT/Apache/MIT while the workspace/package license is AGPL-3.0-only.
- `cargo doc` emits broken links for `install_package` and `WhisperStt`.

Recommendations:
- Standardize crate READMEs to `AGPL-3.0-only` unless licensing is intentionally changed.
- Fix README examples and add doctest coverage where practical.
- Update video README to describe the current protocol/exporter-only state.
- Fix rustdoc links: use `Self::install_package` or backticks without link syntax, and gate or link `WhisperStt` correctly under the `whisper` feature.

### P11 — Test coverage is uneven

Severity: Low-Medium
Evidence:
- `cargo test --workspace --all-targets` passed, but several AI crates and contracts had zero tests.
- `dracon-git` has meaningful tests, and `dracon-memory-runtime` has broad unit coverage; media tests mostly cover helpers rather than external tool/runtime behavior.

Impact:
- Security-sensitive paths have limited negative tests: command approval/PATH, process UID denial, remote shell API usage, Git hook behavior, and protected-file filter failure are not fully covered.

Recommendations:
- Add targeted unit/integration tests for deny-by-default behavior and error propagation.
- Add fake external binary tests for `espeak-ng` and `dracon-warden`.
- Add API example/doctest coverage for public README examples.

## Recommended remediation order

1. Fix `SystemAgent` command approval/execution semantics and make privileged config/package operations approval-gated.
2. Fix Git hook behavior, path operand handling, and fail-closed protected-file filtering.
3. Remove or gate deprecated raw remote shell execution.
4. Add memory privacy/retention documentation and APIs.
5. Align AI `reqwest` features and add secret redaction/validation.
6. Tighten unsafe blocks and external tool failure handling.
7. Refresh high-impact dependencies and reduce duplicate dependency families.
8. Clean clippy/doc warnings and fix README/API examples.
9. Add targeted tests for security-sensitive deny/fail-closed behavior.

## No implementation changes made

No source, manifest, lockfile, or CI files were edited. `git status --short` after the audit shows only `?? .pi/`, which is the audit/goal bookkeeping directory.
