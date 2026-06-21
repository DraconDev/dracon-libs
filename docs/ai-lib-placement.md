# ai-lib placement decision

Date: 2026-06-12
Last reviewed: 2026-06-21

## Decision

Keep `dracon-ai-lib` out of this `dracon-libs` workspace. Do not add it as a local path dependency, and do not copy it into this repository.

Current state: this workspace is not using `dracon-ai-lib` locally. Searches found no Cargo dependency, lockfile entry, or source reference for `dracon-ai-lib`, `ai-lib`, or `ai-models-catalog`; the only references were this decision document. The local checkout path previously shown as an example, `/home/dracon/Dev/dracon-ai-lib`, is also absent on this machine.

If a consumer needs the replacement AI client/engine functionality, use the archived `dracon-ai-lib` successor packages from `DraconDev/dracon-platform` instead of a local `dracon-ai-lib` checkout.

## Current status

- `dracon-ai-lib` is archived as of 2026-06-20.
- The AI engine source moved to `DraconDev/dracon-platform` at `apis/services/ai-api/ai-engine/` as package `ai-engine v0.2.0+`.
- The HTTP client SDK moved to `DraconDev/dracon-platform` at `apis/services/ai-api/dracon-ai-sdk/` as package `dracon-ai-sdk v0.3.0+`.
- The models catalog moved to `DraconDev/dracon-platform` at `apis/libs/ai-models-catalog/` as package `ai-models-catalog v0.2.0+`.
- The checked remote for `DraconDev/dracon-platform` currently exposes `dracon-ai-sdk-v0.3.0`; no dedicated `ai-engine` or `ai-models-catalog` tag was found during this review.

## Rationale

1. **No local usage in `dracon-libs`**
   - `grep` found no `ai-lib`, `dracon-ai-lib`, or `ai-models-catalog` references outside this document.
   - `Cargo.toml` and `Cargo.lock` contain no dependency on those packages.
   - The example local path `/home/dracon/Dev/dracon-ai-lib` does not exist locally.

2. **Different runtime boundary**
   - The old `dracon-ai-lib` owned the standalone BYOK provider client/engine surface: direct provider calls where the caller supplies API keys, base URLs, and model IDs.
   - `dracon-libs` already owns local AI runtime/service crates: `dracon-ai-contracts`, `dracon-ai-runtime-contracts`, `ai-runtime-adapters`, `ai-routing-runtime`, `ai-runtime-config`, and `ai-service`.
   - Merging the old standalone repo into this workspace would blur the direct-BYOK client boundary with this workspace's runtime/service boundary.

3. **Avoid duplicating deprecated legacy crates**
   - The archived `dracon-ai-lib` still contains deprecated legacy crates such as `dracon-ai-client`, `dracon-ai-contracts`, `dracon-ai-core`, `dracon-ai-providers`, and `extract-keys`.
   - Copying the old repo would duplicate or conflict with `dracon-libs` crates, especially `dracon-ai-contracts`.

4. **Use the platform repo for replacement packages**
   - The platform repository is the current source for the replacement engine and SDK.
   - If a consumer needs a local development override, prefer a `[patch]` or path dependency to the platform checkout paths, not the archived `dracon-ai-lib` checkout.

## Recommended dependency guidance

For consumers that need the SDK replacement:

```toml
dracon-ai-sdk = { git = "https://github.com/DraconDev/dracon-platform.git", tag = "dracon-ai-sdk-v0.3.0" }
```

For consumers that need the engine or models catalog, pin an explicit platform revision or future tag after confirming the required package is available:

```toml
ai-engine = { git = "https://github.com/DraconDev/dracon-platform.git", rev = "<pinned-platform-rev>" }
ai-models-catalog = { git = "https://github.com/DraconDev/dracon-platform.git", rev = "<pinned-platform-rev>" }
```

For local development overrides, use the platform checkout paths if present:

```toml
ai-engine = { path = "/home/dracon/Dev/dracon-platform/apis/services/ai-api/ai-engine" }
ai-models-catalog = { path = "/home/dracon/Dev/dracon-platform/apis/libs/ai-models-catalog" }
```

Do not use the archived `/home/dracon/Dev/dracon-ai-lib` path for new work.

## Recommended follow-up

Do not merge `dracon-ai-lib` into `dracon-libs` unless there is a new requirement that `dracon-libs` must publish the AI engine/catalog under the `dracon-libs` workspace version and release process.

## Evidence inspected

- `git status --short --branch` in `dracon-libs`: clean workspace.
- `grep -RIn "/home/dracon/Dev/dracon-ai-lib\\|dracon-ai-lib\\|ai-models-catalog\\|ai-lib ="` in `dracon-libs`: references only in this document.
- `grep -n "ai-lib\\|models-catalog\\|dracon-ai-lib" Cargo.lock Cargo.toml services contracts tools docs`: no dependency entries outside this document.
- `/home/dracon/Dev/dracon-ai-lib`: absent.
- `git ls-remote https://github.com/DraconDev/dracon-ai-lib.git refs/tags/v0.2.0 refs/heads/main`: tag and main exist.
- `git clone --depth 1 --branch main https://github.com/DraconDev/dracon-ai-lib.git`: README says archived and points to `DraconDev/dracon-platform`.
- `git ls-remote --tags https://github.com/DraconDev/dracon-platform.git`: only `dracon-ai-sdk-v0.3.0` was found for the AI package tags checked.
- Local platform checkout:
  - `/home/dracon/Dev/dracon-platform/apis/services/ai-api/ai-engine/Cargo.toml`: `ai-engine v0.2.0`.
  - `/home/dracon/Dev/dracon-platform/apis/services/ai-api/dracon-ai-sdk/Cargo.toml`: `dracon-ai-sdk v0.3.0`.
  - `/home/dracon/Dev/dracon-platform/apis/libs/ai-models-catalog/Cargo.toml`: `ai-models-catalog v0.2.0`.
