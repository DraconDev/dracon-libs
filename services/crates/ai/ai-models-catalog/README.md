# ai-models-catalog

models.dev-backed provider catalog and BYOK config types for AI clients.

## What this is

A pure Rust data crate for:

- parsing `models.dev/api.json`
- representing provider + model metadata
- loading JSONC config files with the opencode/Kilo-style `provider.<id>` shape
- looking up built-in env var names for common providers

## What this is not

- Not an HTTP client
- Not a provider adapter
- Not a key store
- Not an env-var reader
- Not a replacement for `ai-lib`

It is the shared schema layer that `ai-lib`, `dracon-code`, and other tools can use to agree on provider metadata and config shape.

## Example

```rust
use ai_models_catalog::{parse_config, Catalog};

let catalog_json = r#"{"openai":{"id":"openai","name":"OpenAI","env":["OPENAI_API_KEY"],"models":{"gpt-4o":{"id":"gpt-4o","name":"GPT-4o"}}}}"#;
let catalog: Catalog = serde_json::from_str(catalog_json).unwrap();

let config = parse_config(r#"{"provider":{"openai":{"env":["OPENAI_[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBvQ2I0aTJ1QXdGVFNTMVhEcURVM2lmS0FrL1ZtbmJEWGV0L1dOamdVd240ClRjbFVkQnJjTjhtVDBxeGpyUFg0QXo5VnZJZnMxNS9qanhncFhqSFVMRE0KLT4gWDI1NTE5IFgrSEpJS2N0ZWtjZTY5allhZmI3NWpxekhTbzZLNWZIUDU1elhWOWxGUWsKaWVSV3JCMGJ6eEU4ZkY1RzdITitIUmd0MFZJYWU2VDhJRXMvMng4dXBFQQotPiBYMjU1MTkgMWRVelBVbVp4dkJWTUhPSHJ2SEhIUFBrU0ZBVnVSazZqZEk5SFg1V1YydwoyNUFwem1QbG9CdHE2T29QKzdhYXU4cSs2SGpyajA5UFhpaWp3R0VZYWdRCi0+IFgyNTUxOSBqT1RlRlZBT2tZZkN2Tm4yS2xqNVg4VXFMOVBYMkhQTlhFTEFXOHUvWHhFCkhsSUh5ejcwaUxBenRVSFRvb21vVk00SXFBS3RLNWcxUjhNZU8zazk4V0EKLT4gWDI1NTE5IElTd0FVc0Z0dDI1Y1JybFc2aEFYTkliWldMQ3JSd0JLWWY2VGMxLzBCbjAKbFVPTXlNSldnc09MMHd5L3RURmU4S2d4VklUSjd2M0xVQjVmaCtpZlBiUQotPiBKLS1ncmVhc2UgRiQiIHxJOihZOksgMltJLgpYY2hEOWo1a1hCYkZrWk1UZVlobUpTUXFwSWJSNHFXaVhGZnF0ajhVSkdXZTI5Yi80a25lOEZ0K3hYSXBjRllsCi9qSElRUzhtUE1KM3pZOFZQYlNCdXV1SjZmdHN1Ry9VbGhRUCt6WmJwZ1UKLS0tIEVmY3czbjN3d25UZndmRVl4bWxnRWNpSHN2ejEzWUtlOGFJK3NyS1A3aFkKBbTtJ1p/m8dxA5m+kb+iuLMKeNH6hK6lxi4o9xfX34Xmg0cH7Kh54rbwV5scFkXsMAWpd+7QC3n/5qCmA2Id9u8ntG/sIX1EY9F8]}}}"#).unwrap();
```

## Built-in env vars

The crate ships a small lookup table for the common providers:

- `openai` → `OPENAI_API_KEY`
- `anthropic` → `ANTHROPIC_API_KEY`
- `google` → `GOOGLE_API_KEY`
- `mistral` → `MISTRAL_API_KEY`
- `deepseek` → `DEEPSEEK_API_KEY`
- `nvidia` → `NVIDIA_API_KEY`
- `openrouter` → `OPENROUTER_API_KEY`
- `minimax` → `MINIMAX_API_KEY`
- `apertis` → `APERTIS_API_KEY`

Use `env_var_names_for(provider_id)` to read them.

## Config shape

```jsonc
{
  "$schema": "https://ai-models-catalog.dracon.dev/schema.json",
  "provider": {
    "openai": {
      "env": ["OPENAI_API_KEY"],
      "api": "https://api.openai.com/v1",
      "models": {
        "gpt-4o": {
          "name": "GPT-4o"
        }
      }
    }
  }
}
```

## License

AGPL-3.0.
