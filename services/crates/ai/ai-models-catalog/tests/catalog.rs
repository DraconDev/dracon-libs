use ai_models_catalog::{
    builtin_env_vars, env_var_names_for, load_config, merge_configs, parse_config,
    AiModelsConfig, Catalog, ProviderConfig,
};

#[test]
fn parses_models_dev_api_json() {
    let json = r##"
    {
      "openai": {
        "id": "openai",
        "name": "OpenAI",
        "npm": "@ai-sdk/openai",
        "env": ["OPENAI_API_KEY"],
        "doc": "https://platform.openai.com/docs/models",
        "api": "https://api.openai.com/v1",
        "models": {
          "gpt-4o": {
            "id": "gpt-4o",
            "name": "GPT-4o",
            "family": "gpt",
            "release_date": "2024-05-13",
            "attachment": true,
            "reasoning": false,
            "temperature": true,
            "tool_call": true,
            "structured_output": true,
            "open_weights": false,
            "cost": {
              "input": 2.5,
              "output": 10,
              "cache_read": 1.25,
              "cache_write": 5
            },
            "limit": {
              "context": 128000,
              "output": 16384
            },
            "modalities": {
              "input": ["text", "image"],
              "output": ["text"]
            },
            "status": null
          }
        }
      }
    }
    "##;

    let catalog: Catalog = serde_json::from_str(json).expect("parse");
    assert_eq!(catalog.get_provider("openai").expect("provider").name, "OpenAI");
    assert_eq!(
        catalog.get_model("openai", "gpt-4o").expect("model").name,
        "GPT-4o"
    );
    assert_eq!(
        catalog
            .get_model("openai", "gpt-4o")
            .expect("model")
            .cost
            .expect("cost")
            .input,
        Some(2.5)
    );
}

#[test]
fn parses_jsonc_config() {
    let jsonc = r#"
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
    "#;

    let config = parse_config(jsonc).expect("parse");
    let provider = config.provider.get("openai").expect("provider");
    assert_eq!(provider.api.as_deref(), Some("https://api.openai.com/v1"));
    assert_eq!(provider.env, vec!["OPENAI_API_KEY"]);
    assert_eq!(
        provider.models.get("gpt-4o").expect("model").name.as_deref(),
        Some("GPT-4o")
    );
}

#[test]
fn builtin_env_lookup_works_for_three_providers() {
    assert_eq!(
        env_var_names_for("openai").expect("openai"),
        vec!["OPENAI_API_KEY"]
    );
    assert_eq!(
        env_var_names_for("anthropic").expect("anthropic"),
        vec!["ANTHROPIC_API_KEY"]
    );
    assert_eq!(
        env_var_names_for("openrouter").expect("openrouter"),
        vec!["OPENROUTER_API_KEY"]
    );
}

#[test]
fn deep_merge_config_layers() {
    let mut base = AiModelsConfig::default();
    base.provider.insert(
        "openai".into(),
        ProviderConfig {
            name: Some("OpenAI".into()),
            env: vec!["OPENAI_API_KEY".into()],
            api: Some("https://api.openai.com/v1".into()),
            ..Default::default()
        },
    );

    let mut override_config = AiModelsConfig::default();
    override_config.provider.insert(
        "openai".into(),
        ProviderConfig {
            api: Some("https://proxy.example.com/v1".into()),
            ..Default::default()
        },
    );

    merge_configs(&mut base, override_config);

    let provider = base.provider.get("openai").expect("provider");
    assert_eq!(provider.name.as_deref(), Some("OpenAI"));
    assert_eq!(provider.env, vec!["OPENAI_API_KEY"]);
    assert_eq!(provider.api.as_deref(), Some("https://proxy.example.com/v1"));
}

#[test]
fn load_config_reads_file_from_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("ai-models.jsonc");
    std::fs::write(
        &path,
        r#"
        {
          "provider": {
            "anthropic": {
              "env": ["ANTHROPIC_API_KEY"]
            }
          }
        }
        "#,
    )
    .expect("write");

    let config = load_config(&path).expect("load");
    assert_eq!(
        config
            .provider
            .get("anthropic")
            .expect("provider")
            .env,
        vec!["ANTHROPIC_API_KEY"]
    );
}

#[test]
fn builtin_env_vars_contains_nine_common_providers() {
    let names: Vec<_> = builtin_env_vars().iter().map(|(provider, _)| provider.id()).collect();
    assert_eq!(
        names,
        vec![
            "openai",
            "anthropic",
            "google",
            "mistral",
            "deepseek",
            "nvidia",
            "openrouter",
            "minimax",
            "apertis",
        ]
    );
}
