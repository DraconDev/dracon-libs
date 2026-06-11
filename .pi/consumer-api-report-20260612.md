# Dracon-libs External Rust Consumer/API Audit Report

Date: 2026-06-12
Scope: External Rust crate/API consumers of the `dracon-libs` workspace.
Boundary: Read-only analysis. No source, manifest, lockfile, CI, or README changes were made.

## Executive summary

`dracon-libs` exposes a broad set of public Rust APIs across Git, system, files, media, memory, and AI crates. Most core APIs are discoverable and reasonably typed, but downstream consumers will hit avoidable friction: several README examples do not match the real API, feature/default-feature documentation is inconsistent, public structs/enums are exposed without `#[non_exhaustive]`, and some contract crates diverge from runtime APIs.

The highest-priority consumer fixes are documentation/example correctness, feature-flag alignment, and a deliberate semver plan for public structs/enums before adding fields or variants.

## Verification commands and outcomes

- `cargo metadata --locked --format-version=1 --no-deps`: 14 workspace packages; package names and crate names inventoried.
- `rg -n '^pub (struct|enum|trait|type|fn|mod|use|const)' ...`: public API entry points inventoried across workspace crates.
- `nix-shell -p pkg-config alsa-lib sqlite --run 'cargo doc --workspace --no-deps --target-dir /tmp/dracon-libs-consumer-doc-default'`: passed, with 2 rustdoc warnings:
  - broken link to `WhisperStt` in `tools/media/dracon-stt-runtime/src/lib.rs:8`
  - broken link to `install_package` in `tools/system/dracon-system/src/lib.rs:199`
- `rg -n 'non_exhaustive|^pub (struct|enum)' tools services contracts`: no `#[non_exhaustive]` found on public structs/enums.
- `rg -n 'MIT|Apache|AGPL' README.md tools services contracts -g 'README.md'`: crate README license text conflicts with workspace/package AGPL metadata in multiple crates.
- `git status --short`: only `.pi/` goal/report bookkeeping changes (`M .pi/goals/active_goal_2026061200025808_mqa3sfmq-3x0msc.md` and `?? .pi/consumer-api-report-20260612.md`), confirming no implementation changes to source, manifests, lockfile, or CI.

## Public API inventory

| Crate | Main external entry points | Consumer notes |
|---|---|---|
| `dracon-files` / crate `dracon_files` | `FsCatalog`, `FileInspectContract`, `FileSearchContract`, `FileCopyContract`, `FileSuitabilityContract`, `FileCategory`, `EntryMetadata`, `FileSearchResult` | Simple file classification/search/copy API. README example is minimal and omits how `path` is supplied. |
| `dracon-git` / crate `dracon_git` | `GitService`, `CliGitSnapshotProvider`, `RepoStatus`, `FileStatus`, `DiffFile`, `build_commit_message`, `extract_intent`, `scan_blueprint_tasks`, `TaskProgress`, `IntentInfo` | Useful Git operations and commit-intent helpers. `GitService::new()` accepts any path; consumers should know repo-root validation is not enforced in the constructor. |
| `dracon-system-lib` / crate `dracon_system_lib` | `SystemAgent`, `SystemSnapshotProvider`, `ProcessController`, `SshRemoteConnector`, `SshRemoteExecProvider`, `SshRemoteFsProvider`, `WorkspaceStorageReport`, `analyze_workspace_storage`, `AppNotification` | Broad privileged/system surface. README example is incorrect (`SystemSnapshotProvider::new()?` even though `new()` returns `Self`). |
| `dracon-stt-runtime` / crate `dracon_stt_runtime` | `ParakeetStt`, feature-gated `WhisperStt`, `SttEngine`, `VadStateMachine`, `SpeechToText`, `TimestampedTranscription`, `TranscriptionResult`, `TimestampedSegment`, `EngineCapabilities` | README treats sync APIs as async. Default feature is `parakeet`; `WhisperStt` only exists with `whisper`. |
| `dracon-tts-runtime` / crate `dracon_tts_runtime` | `KittenTTS`, `KokoroTts`, `TtsEngine`, `TextToSpeech`, `VoiceProvider`, `DynTtsEngine`, `VoiceInfo`, `TtsConfig`, `SynthesisRequest`, `SynthesisResult` | README is wrong for `KittenTTS::new()` and `speak()` usage. Crate docs also contain example typos. Default features include both `kitten` and `kokoro`, while README says kitten default. |
| `dracon-video-runtime` / crate `dracon_video_runtime` | `VideoProcessor`, `AudioProcessor`, `TranscriptProcessor`, `SilenceSegment`, `TrimSegment`, `TranscriptSegment`, exporter functions `export_srt`, `export_youtube_chapters`, `export_ass` | README promises runtime implementations such as `FfmpegVideoProcessor` and `WhisperTranscriptProcessor`, but `src/lib.rs` says runtime implementations are not present. Currently protocol/exporter-oriented. |
| `dracon-memory-contracts` / crate `dracon_memory_contracts` | `MemoryStore`, `TextEmbedder`, `Role`, `Conversation`, `UserFact` | Lightweight contract crate for consumers that should not depend on runtime deps. Runtime API does not exactly mirror this trait shape. |
| `dracon-memory-runtime` / crate `dracon_memory_runtime` | `MemorySystem`, `MemoryDb`, `OnnxEmbedder`, re-exported contract types | `MemorySystem::new()` is sync `Result`; async methods return `anyhow::Result`. README is broadly aligned, but direct `OnnxEmbedder::embed(&mut self)` differs from `TextEmbedder::embed(&self)`. |
| `dracon-ai-contracts` / crate `dracon_ai_contracts` | `RoutingTask`, `SelectionConstraints` | Small routing contract crate. `SelectionConstraints` is not `Serialize`/`Deserialize` while many sibling config types are. |
| `dracon-ai-runtime-contracts` / crate `dracon_ai_runtime_contracts` | `AiProvider`, `ChatMessage`, `ChatRequest`, `ChatResponse` | Core AI provider contract. `ChatRequest::stream` exists but generic adapter does not actually stream. |
| `ai-routing-runtime` / crate `ai_routing_runtime` | `SmartRouter<T>`, `ProviderRegistry<T>`, `RoutingMessage`, `RoutingTrace`, `AiModelStore`, `ServiceLevel` | Generic routing API. `route_with_trace()` ignores `_project`, `_preferred`, and `_messages`; consumers may expect those to influence routing. |
| `ai-service` / crate `ai_service` | `AiService`, `ProviderRegistry`, `LaneModelPolicy`, `DEFAULT_PROVIDER`, re-exported AI contract types | Simple high-level service. No crate README found. `ProviderRegistry` and `AiService` are concrete, not trait-based, but accept `Arc<dyn AiProvider>`. |
| `ai-runtime-adapters` / crate `ai_runtime_adapters` | `GenericOpenAIAdapter` | Minimal OpenAI-compatible adapter. No crate README found. Example includes a fake API key and no redaction guidance. |
| `ai-runtime-config` / crate `ai_runtime_config` | `OpenAIProviderSpec`, `AiRuntimeConfig`, `resolve_ai_runtime_config` | Docs claim config resolves from environment or file, but implementation only returns an empty default config. No crate README found. |

## Prioritized consumer-facing findings

### 1. README and rustdoc examples are not trustworthy for first-time consumers

Severity: High
Evidence:
- TTS README shows `KittenTTS::new().await?`, but `KittenTTS::new(model_path, voices_path)` requires two arguments (`tools/media/dracon-tts-runtime/src/kitten.rs:176`).
- TTS README shows `kitten.speak("Hello world")?`, but crate-level docs call `speak(...).await`; the trait method is synchronous (`tools/media/dracon-tts-runtime/src/contracts.rs:181`).
- TTS crate docs have `. await?` spacing and call `speak(...).await` (`tools/media/dracon-tts-runtime/src/lib.rs:14-15`).
- STT README shows `ParakeetStt::new(model_path).await?` and `transcribe(...).await?`, but `ParakeetStt::new` and `SpeechToText::transcribe` are synchronous (`tools/media/dracon-stt-runtime/src/parakeet.rs:17`; `tools/media/dracon-stt-runtime/src/stt_contracts.rs`).
- System README shows `SystemSnapshotProvider::new()?`, but `new()` returns `Self` (`tools/system/dracon-system/src/lib.rs:78`).
- Video README imports `FfmpegVideoProcessor` and `WhisperTranscriptProcessor`, but runtime implementations are not present (`tools/media/dracon-video-runtime/src/lib.rs:30`).
- `cargo doc --workspace --no-deps` emits broken links for `WhisperStt` and `install_package`.

Impact:
- A downstream Rust user will copy examples that do not compile.
- Docs undermine confidence in the public API surface even where the code itself is usable.

Recommendations:
1. Fix README examples first; add at least one compile-checked example per public crate.
2. Prefer `ignore` only when examples intentionally require external services or files.
3. Add rustdoc tests for low-risk examples.
4. Fix broken intra-doc links.

### 2. Feature flags and defaults are misleading

Severity: High
Evidence:
- TTS README says `kitten` is default/no GPU, but Cargo default is `["kitten", "kokoro"]` (`tools/media/dracon-tts-runtime/Cargo.toml:11-13`).
- STT README says `parakeet` default and `whisper` optional; Cargo matches, but `WhisperStt` link is broken under default docs.
- Video README says `ffmpeg` is default, but Cargo default is empty (`tools/media/dracon-video-runtime/Cargo.toml:34-36`).
- System README mentions a `notify` feature, but `tools/system/dracon-system/Cargo.toml` has no `[features]` section.

Impact:
- Consumers cannot infer which dependencies/features they will pull in.
- Feature changes may become accidental breaking changes.

Recommendations:
1. Align README feature tables with Cargo.toml.
2. Decide whether TTS should keep both default features or make heavy backends optional.
3. Either implement the `notify` feature or remove it from docs.
4. Document exact external binaries/services required per feature.

### 3. Public structs/enums are not future-proof for semver evolution

Severity: High
Evidence:
- `rg -n "non_exhaustive|^pub (struct|enum)" tools services contracts` found many public structs/enums and no `#[non_exhaustive]`.
- Examples include `SystemSnapshot`, `RemoteBookmark`, `RemoteConnection`, `OpenAIProviderSpec`, `AiRuntimeConfig`, `LaneModelPolicy`, `TtsConfig`, `VoiceInfo`, `GitRepoSnapshot`, `TranscriptSegment`, and many contract data types.

Impact:
- Adding fields to public structs is breaking for consumers using struct literals.
- Adding enum variants is breaking for exhaustive matches.
- Public traits with required methods also constrain downstream implementors.

Recommendations:
1. For the next major release, add `#[non_exhaustive]` to public structs/enums intended to grow.
2. Before that, document that adding fields/variants is breaking and avoid adding them in minor releases.
3. Add constructors/builders for common config structs (`OpenAIProviderSpec`, `AiRuntimeConfig`, `TtsConfig`, `SynthesisRequest`).
4. Prefer sealed traits where downstream implementation is not expected.

### 4. Contract/runtime APIs diverge in ways consumers may not expect

Severity: Medium-High
Evidence:
- Memory contract `MemoryStore::store_conversation(&Conversation)` differs from runtime `MemorySystem::store_conversation(role, content)` (`contracts/crates/dracon-memory-contracts/src/lib.rs`; `tools/memory/dracon-memory-runtime/src/lib.rs`).
- Memory contract `TextEmbedder::embed(&self)` differs from direct `OnnxEmbedder::embed(&mut self)` (`contracts/crates/dracon-memory-contracts/src/lib.rs`; `tools/memory/dracon-memory-runtime/src/embedder.rs:75`).
- `ParakeetStt` implements `TimestampedTranscription`, but capabilities report `supports_timestamps: false` (`tools/media/dracon-stt-runtime/src/parakeet.rs:181-182`; `tools/media/dracon-stt-runtime/src/stt_contracts.rs`).
- `ai-runtime-config` docs claim environment/file resolution, but `resolve_ai_runtime_config()` only returns an empty default config (`services/crates/ai/ai-runtime-config/src/lib.rs:50`).

Impact:
- Consumers depending on contract crates may be surprised by runtime APIs.
- Trait naming suggests capabilities that are not actually provided.

Recommendations:
1. Add README sections explaining contract crates vs runtime crates.
2. Either align runtime APIs with contract traits or explicitly document the divergence.
3. Rename or gate `TimestampedTranscription` implementation if timestamp support is absent.
4. Implement or rename `resolve_ai_runtime_config()` to avoid promising file/env loading.

### 5. Async/sync and Result ergonomics are inconsistent

Severity: Medium
Evidence:
- TTS trait `TextToSpeech::speak` returns `TtsResult<()>`, while inherent `KittenTTS::speak` and `KokoroTts::speak` are async and return `()`.
- STT `ParakeetStt::new` is sync, but README uses `.await`.
- TTS `KittenTTS::new` is async with required paths; no zero-arg default constructor exists.
- Some APIs return `Option` for failure (`ParakeetStt::transcribe_audio`), while sibling APIs return `Result`.

Impact:
- Downstream code must learn per-crate conventions rather than relying on workspace-wide API patterns.
- Error details are lost where `Option` or silent fallback is used.

Recommendations:
1. Standardize constructor patterns: sync constructors for pure config/path binding, async constructors only when IO/model loading occurs.
2. Standardize fallible APIs as `Result` where error context matters.
3. Add convenience constructors or builders for common TTS/STT setups.
4. Document async/sync behavior in README and rustdoc.

### 6. AI service/config ergonomics need clearer boundaries

Severity: Medium
Evidence:
- `GenericOpenAIAdapter::new_with_auth()` accepts raw strings for endpoint/model/auth header/prefix with no validation (`services/crates/ai/ai-runtime-adapters/src/lib.rs:28-40`).
- `ChatRequest::stream` exists, but the generic adapter returns a collected string and finish reason, not a stream (`contracts/crates/ai/dracon-ai-runtime-contracts/src/models.rs:14-28`; `services/crates/ai/ai-runtime-adapters/src/lib.rs:64-104`).
- `AiRuntimeConfig` and `OpenAIProviderSpec` expose public fields but no builders/validation.
- Examples include fake API keys (`sk-test`, `sk-test-key`).

Impact:
- Consumers may misuse endpoints, auth headers, or streaming expectations.
- API keys in examples teach unsafe logging/debug habits.

Recommendations:
1. Add validated builder constructors for `OpenAIProviderSpec`, `AiRuntimeConfig`, and `GenericOpenAIAdapter`.
2. Rename or document streaming behavior; add a separate streaming trait/method if real streaming is intended.
3. Redact API keys in `Debug` output and examples.
4. Add READMEs for AI service/config/adapter crates.

### 7. Video crate currently sells runtime capabilities it does not expose

Severity: Medium
Evidence:
- README architecture diagram lists runtime implementations: `FfmpegVideoProcessor`, `WhisperTranscriptProcessor`, `AutoReframeProcessor`.
- `src/lib.rs` states runtime implementations are not present.
- Public API currently exposes protocol traits and exporters.

Impact:
- Consumers may choose this crate expecting ready-to-use FFmpeg/Whisper processors.

Recommendations:
1. Update README to say “protocol + exporter utilities; runtime implementations not included yet.”
2. Add a roadmap section for planned implementations.
3. If implementations are planned, add feature-gated modules with stubs or examples.

### 8. Crate READMEs are missing or inconsistent

Severity: Medium
Evidence:
- AI service/config/adapter/routing crates have no README files.
- Existing READMEs advertise MIT/Apache/MIT while workspace/package metadata says AGPL-3.0-only (`Cargo.toml:26`; crate READMEs under `tools/.../README.md`).
- Some READMEs claim feature flags not present in Cargo.toml.

Impact:
- Downstream users cannot quickly assess licensing, features, or examples for AI crates.
- License mismatch creates legal/compliance confusion.

Recommendations:
1. Add concise READMEs for AI crates with usage examples and feature flags.
2. Standardize license text to match package/workspace metadata.
3. Add a workspace-level “consumer quick start” table mapping use case to crate.

## Prioritized remediation plan

1. **Fix non-compiling README/rustdoc examples**
   - TTS, STT, system, video, and rustdoc links should be corrected first.
   - Add compile-checked examples where possible.

2. **Align feature flags and defaults with docs**
   - TTS default features, video default features, system `notify`, and STT `whisper` docs need one source of truth.

3. **Publish a semver/public API policy**
   - Decide where `#[non_exhaustive]` should be added in the next breaking release.
   - Avoid adding fields/variants to existing public structs/enums in minor releases.

4. **Clarify contract vs runtime APIs**
   - Especially memory contracts vs runtime methods, STT timestamp capability, and AI config resolution.

5. **Add ergonomic constructors/builders**
   - TTS engines, AI config/adapter, and common system/memory entry points would be easier for downstream consumers.

6. **Add missing AI crate READMEs**
   - `ai-service`, `ai-runtime-config`, `ai-runtime-adapters`, and `ai-routing-runtime` should document usage, feature flags, and examples.

7. **Standardize error/result patterns**
   - Prefer `Result` where failures should be diagnosable.
   - Document APIs that intentionally return `Option`.

8. **Clean license and feature documentation**
   - Match READMEs to `Cargo.toml` package metadata and actual feature gates.

## What I would not change yet

- Do not rename public crates or APIs without a major-version plan.
- Do not upgrade dependencies as part of this consumer-API cleanup; dependency health is a separate track.
- Do not implement missing video runtime processors unless product scope confirms the crate should provide them.
- Do not add `#[non_exhaustive]` casually in a minor release; it can itself be breaking for consumers who construct public structs or match enums exhaustively.

## Bottom line

The workspace is usable, but external Rust consumers will get a better experience if the team first fixes docs/examples and feature documentation, then makes an explicit semver plan for public structs/enums. After that, add builders/constructors and missing READMEs for the AI crates.
