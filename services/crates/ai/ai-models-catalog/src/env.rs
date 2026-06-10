//! Built-in env-var catalog for common providers.
//!
//! This is only a lookup table. It does **not** read env vars on its own.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinProvider {
    OpenAi,
    Anthropic,
    Google,
    Mistral,
    DeepSeek,
    Nvidia,
    OpenRouter,
    Minimax,
    Apertis,
}

impl BuiltinProvider {
    pub const fn id(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::Mistral => "mistral",
            Self::DeepSeek => "deepseek",
            Self::Nvidia => "nvidia",
            Self::OpenRouter => "openrouter",
            Self::Minimax => "minimax",
            Self::Apertis => "apertis",
        }
    }
}

/// Built-in env var names for common providers.
pub fn builtin_env_vars() -> &'static [(BuiltinProvider, &'static [&'static str])] {
    &[
        (BuiltinProvider::OpenAi, &["OPENAI_API_KEY"]),
        (BuiltinProvider::Anthropic, &["ANTHROPIC_API_KEY"]),
        (BuiltinProvider::Google, &["GOOGLE_API_KEY"]),
        (BuiltinProvider::Mistral, &["MISTRAL_API_KEY"]),
        (BuiltinProvider::DeepSeek, &["DEEPSEEK_API_KEY"]),
        (BuiltinProvider::Nvidia, &["NVIDIA_API_KEY"]),
        (BuiltinProvider::OpenRouter, &["OPENROUTER_API_KEY"]),
        (BuiltinProvider::Minimax, &["MINIMAX_API_KEY"]),
        (BuiltinProvider::Apertis, &["APERTIS_API_KEY"]),
    ]
}

/// Return the env var names for a provider id.
pub fn env_var_names_for(provider_id: &str) -> Option<Vec<&'static str>> {
    builtin_env_vars()
        .iter()
        .find(|(provider, _)| provider.id() == provider_id)
        .map(|(_, vars)| vars.to_vec())
}
