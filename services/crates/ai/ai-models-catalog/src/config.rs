//! JSONC config parsing for ai-models-catalog.
//!
//! The shape mirrors opencode/Kilo Code:
//!
//! ```jsonc
//! {
//!   "provider": {
//!     "openai": {
//!       "env": ["OPENAI_API_KEY"],
//!       "api": "https://api.openai.com/v1",
//!       "models": {
//!         "gpt-4o": {
//!           "name": "GPT-4o"
//!         }
//!       }
//!     }
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Top-level config file.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AiModelsConfig {
    #[serde(default)]
    pub provider: BTreeMap<String, ProviderConfig>,
}

/// Provider config block.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProviderConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub npm: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelOverride>,
    #[serde(default)]
    pub options: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
}

/// Per-model override block.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ModelOverride {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub attachment: Option<bool>,
    #[serde(default)]
    pub reasoning: Option<bool>,
    #[serde(default)]
    pub temperature: Option<bool>,
    #[serde(default)]
    pub tool_call: Option<bool>,
    #[serde(default)]
    pub cost: Option<serde_json::Value>,
    #[serde(default)]
    pub limit: Option<serde_json::Value>,
    #[serde(default)]
    pub modalities: Option<serde_json::Value>,
    #[serde(default)]
    pub options: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub variants: Option<BTreeMap<String, serde_json::Value>>,
}

/// Parse a JSONC string into an [`AiModelsConfig`].
///
/// Supports `//` and `/* ... */` comments, but not trailing commas (yet).
pub fn parse_config(input: &str) -> Result<AiModelsConfig, serde_json::Error> {
    let cleaned = strip_json_comments(input);
    serde_json::from_str(&cleaned)
}

/// Load a config file from disk.
pub fn load_config(path: impl AsRef<Path>) -> Result<AiModelsConfig, ConfigError> {
    let content = fs::read_to_string(path.as_ref())?;
    parse_config(&content).map_err(ConfigError::Parse)
}

/// Deep-merge two configs. `other` wins on conflicts.
pub fn merge_configs(base: &mut AiModelsConfig, other: AiModelsConfig) {
    for (provider_id, provider_config) in other.provider {
        let entry = base.provider.entry(provider_id).or_default();
        if let Some(name) = provider_config.name {
            entry.name = Some(name);
        }
        if !provider_config.env.is_empty() {
            entry.env = provider_config.env;
        }
        if let Some(api) = provider_config.api {
            entry.api = Some(api);
        }
        if let Some(doc) = provider_config.doc {
            entry.doc = Some(doc);
        }
        if let Some(npm) = provider_config.npm {
            entry.npm = Some(npm);
        }
        if let Some(protocol) = provider_config.protocol {
            entry.protocol = Some(protocol);
        }
        for (model_id, model_override) in provider_config.models {
            entry.models.insert(model_id, model_override);
        }
        if let Some(options) = provider_config.options {
            entry.options = Some(options);
        }
        if let Some(headers) = provider_config.headers {
            entry.headers = Some(headers);
        }
    }
}

/// Minimal JSONC comment stripper.
fn strip_json_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                out.push(ch);
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push(c);
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = '\0';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
            }
            _ => out.push(ch),
        }
    }

    out
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(serde_json::Error),
}
