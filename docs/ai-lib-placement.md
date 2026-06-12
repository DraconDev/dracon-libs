# ai-lib placement decision

Date: 2026-06-12

Decision: keep `dracon-ai-lib` as a standalone repository and do not move or copy it into this `dracon-libs` workspace.

## Rationale

1. **Different runtime boundary**
   - `dracon-ai-lib` owns the standalone BYOK provider client surface: direct provider calls where the caller supplies API keys, base URLs, and model IDs.
   - `dracon-libs` already owns local AI runtime/service crates: `dracon-ai-contracts`, `dracon-ai-runtime-contracts`, `ai-runtime-adapters`, `ai-routing-runtime`, `ai-runtime-config`, and `ai-service`.
   - Merging the two would blur the direct-BYOK client boundary with this workspace's runtime/service boundary.

2. **Existing ownership in `dracon-libs`**
   - `dracon-libs` already has AI contracts, routing, config, adapter, and service crates.
   - It does not currently have `ai-lib` or `ai-models-catalog`, but adding them would create a third AI surface in the same workspace instead of clarifying ownership.

3. **Version and publishing mismatch**
   - `dracon-ai-lib` is currently versioned at `0.2.0` and published from `DraconDev/dracon-ai-lib`.
   - `dracon-libs` uses workspace version `94.7.0` and `AGPL-3.0-only` workspace defaults.
   - Moving `ai-lib` or `ai-models-catalog` into `dracon-libs` would require a deliberate versioning/publishing decision rather than a mechanical move.

4. **Avoid duplicating deprecated legacy crates**
   - `dracon-ai-lib` still contains deprecated legacy crates (`dracon-ai-client`, `dracon-ai-contracts`, `dracon-ai-core`, `dracon-ai-providers`, `extract-keys`).
   - Moving the whole workspace would duplicate or conflict with `dracon-libs` crates, especially `dracon-ai-contracts`.
   - Moving only `ai-lib` and `ai-models-catalog` would split the workspace and require path/dependency cleanup.

5. **Consumers can depend on the standalone repo**
   - Direct BYOK consumers can use `ai-lib = { git = "https://github.com/DraconDev/dracon-ai-lib", tag = "v0.2.0" }`.
   - Local development can use a path dependency or `[patch]` without merging repositories.

## Recommended follow-up

Do not merge repositories unless there is a new requirement that `dracon-libs` must publish `ai-lib`/`ai-models-catalog` under the `dracon-libs` workspace version and release process.

If local development needs a local checkout, prefer one of these lower-risk options:

```toml
# In a consuming workspace:
ai-lib = { path = "/home/dracon/Dev/dracon-ai-lib/crates/ai-lib" }
ai-models-catalog = { path = "/home/dracon/Dev/dracon-ai-lib/crates/ai-models-catalog" }
```

or use Cargo `[patch]` for local overrides while keeping the published source repo separate.

## Evidence inspected

- `/home/dracon/Dev/dracon-ai-lib/Cargo.toml`
- `/home/dracon/Dev/dracon-ai-lib/README.md`
- `/home/dracon/Dev/dracon-ai-lib/docs/STRATEGY.md`
- `/home/dracon/Dev/dracon-libs/Cargo.toml`
- `/home/dracon/Dev/dracon-libs/README.md`
- Existing `dracon-libs` AI crates under `contracts/crates/ai/` and `services/crates/ai/`
