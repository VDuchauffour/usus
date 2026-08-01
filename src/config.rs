use std::{collections::BTreeMap, env, fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::providers::ProviderId;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub default_provider: String,
    #[serde(default)]
    pub sub_day: u32,
    #[serde(default)]
    pub providers: BTreeMap<String, Value>,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(1..=31).contains(&self.sub_day) {
            bail!("sub_day must be between 1 and 31 (got {})", self.sub_day);
        }
        if !self.default_provider.is_empty() && !self.providers.contains_key(&self.default_provider)
        {
            bail!(
                "default = '{}' but no such entry in providers map (have: {})",
                self.default_provider,
                self.providers
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        for (id, blob) in &self.providers {
            let provider_id: ProviderId = id
                .parse()
                .with_context(|| format!("Invalid config for provider '{id}'"))?;
            provider_id
                .validate_blob(blob)
                .with_context(|| format!("Invalid config for provider '{id}'"))?;
        }
        Ok(())
    }

    pub fn pick_provider_id(&self, explicit: Option<ProviderId>) -> Result<ProviderId> {
        if let Some(id) = explicit {
            if self.providers.contains_key(id.as_str()) {
                return Ok(id);
            }
            let prog = env::args().next().unwrap_or_else(|| "usus".into());
            bail!("Provider '{id}' is not configured. Run '{prog} {id} login' first.");
        }
        if !self.default_provider.is_empty()
            && let Ok(id) = self.default_provider.parse::<ProviderId>()
            && self.providers.contains_key(id.as_str())
        {
            return Ok(id);
        }
        if self.providers.len() == 1 {
            let key = self.providers.keys().next().unwrap();
            let id = key
                .parse::<ProviderId>()
                .with_context(|| format!("Provider '{key}' is not a known provider id"))?;
            return Ok(id);
        }
        let configured: Vec<String> = self.providers.keys().cloned().collect();
        let prog = env::args().next().unwrap_or_else(|| "usus".into());
        bail!(
            "No default provider configured. Run '{prog} <provider>' or set 'default_provider' in the config. Configured: {}",
            if configured.is_empty() {
                "none".to_string()
            } else {
                configured.join(", ")
            }
        );
    }
}

const CONFIG_SCHEMA: &str = include_str!("../schema/config.schema.json");

fn validate_against_schema(value: &Value) -> Result<()> {
    let schema: Value =
        serde_json::from_str(CONFIG_SCHEMA).expect("embedded config schema is valid JSON");
    let validator = jsonschema::draft7::options()
        .build(&schema)
        .expect("embedded config schema is a valid JSON Schema");

    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|error| format!("  - {} (at `{}`)", error, error.instance_path()))
        .collect();

    if !errors.is_empty() {
        bail!("config file does not match schema:\n{}", errors.join("\n"));
    }

    Ok(())
}

pub fn config_dir() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".config/usus"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        let prog = env::args().next().unwrap_or_else(|| "usus".into());
        bail!("Configuration not found. Run '{prog} <provider> login' first.");
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Reading config at {}", path.display()))?;
    let value: Value = toml::from_str(&raw).context("Parsing config TOML")?;
    validate_against_schema(&value)?;
    let cfg: Config = serde_json::from_value(value).context("Parsing config TOML")?;
    cfg.validate()?;
    Ok(cfg)
}

pub fn load_or_default() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Reading config at {}", path.display()))?;
    toml::from_str(&raw).context("Parsing config TOML")
}

pub fn save(cfg: &Config) -> Result<()> {
    cfg.validate().context("Refusing to save invalid config")?;
    fs::create_dir_all(config_dir()?)?;
    let path = config_path()?;
    let text = toml::to_string_pretty(cfg).context("Serializing config to TOML")?;
    fs::write(&path, text).with_context(|| format!("Writing config to {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schema_is_valid_json_schema() {
        let schema: Value = serde_json::from_str(CONFIG_SCHEMA).unwrap();
        assert!(jsonschema::meta::is_valid(&schema));
    }

    #[test]
    fn schema_accepts_valid_config() {
        let raw = r#"
default_provider = "anthropic"
sub_day = 5

[providers.anthropic]
admin_key = "sk-ant-admin-x"
allowance = 200.0

[providers."opencode"]
auth_cookie = "Fe26.2**x"
workspace_id = "w"
server_id = "s"
function_id = 31
"#;
        let value: Value = toml::from_str(raw).unwrap();
        validate_against_schema(&value).unwrap();
    }

    #[test]
    fn schema_rejects_unknown_top_level_key() {
        let value: Value = toml::from_str("sub_day = 5\nmystery_field = 42").unwrap();
        assert!(validate_against_schema(&value).is_err());
    }

    #[test]
    fn schema_rejects_unknown_provider_id() {
        let raw = r#"
sub_day = 5

[providers."unknown-provider"]
foo = "bar"
"#;
        let value: Value = toml::from_str(raw).unwrap();
        assert!(validate_against_schema(&value).is_err());
    }

    #[test]
    fn schema_rejects_sub_day_out_of_range() {
        let value: Value = toml::from_str("sub_day = 0").unwrap();
        let err = validate_against_schema(&value).unwrap_err().to_string();
        assert!(err.contains("schema"), "unexpected error: {err}");
        assert!(err.contains("sub_day"), "unexpected error: {err}");
    }

    #[test]
    fn schema_rejects_missing_required_provider_field() {
        let raw = r#"
[providers."opencode"]
auth_cookie = "Fe26.2**x"
"#;
        let value: Value = toml::from_str(raw).unwrap();
        assert!(validate_against_schema(&value).is_err());
    }

    #[test]
    fn schema_rejects_unknown_provider_field() {
        let raw = r#"
[providers.anthropic]
admin_key = "sk-ant-admin-x"
allowance = 200.0
typo_field = "oops"
"#;
        let value: Value = toml::from_str(raw).unwrap();
        assert!(validate_against_schema(&value).is_err());
    }

    #[test]
    fn schema_rejects_wrong_type_for_function_id() {
        let raw = r#"
[providers."opencode"]
auth_cookie = "Fe26.2**x"
workspace_id = "w"
server_id = "s"
function_id = "not-an-integer"
"#;
        let value: Value = toml::from_str(raw).unwrap();
        assert!(validate_against_schema(&value).is_err());
    }

    #[test]
    fn schema_accepts_empty_config() {
        let value: Value = toml::from_str("").unwrap();
        validate_against_schema(&value).unwrap();
    }

    #[test]
    fn reads_namespaced_toml_config() {
        let raw = r#"
default_provider = "anthropic"
sub_day = 5

[providers.anthropic]
admin_key = "sk-ant-admin-x"
allowance = 200.0

[providers."opencode"]
auth_cookie = "x"
workspace_id = "w"
server_id = "s"
function_id = 31
"#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert_eq!(cfg.default_provider, "anthropic");
        assert_eq!(cfg.sub_day, 5);
        assert_eq!(cfg.providers.len(), 2);
        cfg.validate().unwrap();
    }

    #[test]
    fn deny_unknown_top_level_fields() {
        let raw = r#"
default_provider = "anthropic"
sub_day = 5
mystery_field = 42
providers = {}
"#;
        let err = toml::from_str::<Config>(raw).unwrap_err();
        assert!(
            err.to_string().contains("unknown field"),
            "expected unknown-field error, got: {err}"
        );
    }

    #[test]
    fn validate_rejects_sub_day_out_of_range() {
        let cfg = Config {
            sub_day: 0,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
        let cfg = Config {
            sub_day: 32,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_default_not_in_providers() {
        let cfg = Config {
            sub_day: 5,
            default_provider: "ghost".to_string(),
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_unknown_provider_id() {
        let raw = r#"
sub_day = 5

[providers."unknown-provider"]
foo = "bar"
"#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_bad_provider_blob_missing_field() {
        let raw = r#"
sub_day = 5

[providers."opencode"]
auth_cookie = "x"
"#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_bad_provider_blob_unknown_field() {
        let raw = r#"
sub_day = 5

[providers.anthropic]
admin_key = "sk-ant-admin-x"
allowance = 200.0
typo_field = "oops"
"#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn pick_provider_falls_back_to_single_configured() {
        let mut cfg = Config::default();
        cfg.providers.insert("anthropic".to_string(), Value::Null);
        assert_eq!(cfg.pick_provider_id(None).unwrap(), ProviderId::Anthropic);
    }

    #[test]
    fn pick_provider_respects_explicit_flag() {
        let mut cfg = Config::default();
        cfg.providers.insert("opencode".to_string(), Value::Null);
        cfg.providers.insert("anthropic".to_string(), Value::Null);
        cfg.default_provider = "opencode".to_string();
        assert_eq!(
            cfg.pick_provider_id(Some(ProviderId::Anthropic)).unwrap(),
            ProviderId::Anthropic
        );
    }

    #[test]
    fn pick_provider_rejects_unconfigured_explicit() {
        let cfg = Config::default();
        assert!(cfg.pick_provider_id(Some(ProviderId::Anthropic)).is_err());
    }
}
