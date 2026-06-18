use std::{collections::BTreeMap, env, fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::providers;

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
            providers::validate_provider_blob(id, blob)
                .with_context(|| format!("Invalid config for provider '{id}'"))?;
        }
        Ok(())
    }

    pub fn pick_provider_id(&self, explicit: Option<&str>) -> Result<String> {
        if let Some(id) = explicit {
            if self.providers.contains_key(id) {
                return Ok(id.to_string());
            }
            let prog = env::args().next().unwrap_or_else(|| "usus".into());
            bail!("Provider '{id}' is not configured. Run '{prog} login {id}' first.");
        }
        if !self.default_provider.is_empty() && self.providers.contains_key(&self.default_provider)
        {
            return Ok(self.default_provider.clone());
        }
        if self.providers.len() == 1 {
            return Ok(self.providers.keys().next().unwrap().clone());
        }
        let configured: Vec<String> = self.providers.keys().cloned().collect();
        bail!(
            "No default provider configured. Pass --provider <id>. Configured: {}",
            if configured.is_empty() {
                "none".to_string()
            } else {
                configured.join(", ")
            }
        );
    }
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
        bail!("Configuration not found. Run '{prog} login <provider>' first.");
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Reading config at {}", path.display()))?;
    let cfg: Config = toml::from_str(&raw).context("Parsing config TOML")?;
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
    fn reads_namespaced_toml_config() {
        let raw = r#"
default_provider = "anthropic"
sub_day = 5

[providers.anthropic]
admin_key = "sk-ant-admin-x"
allowance = 200.0

[providers."opencode-go"]
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

[providers."opencode-go"]
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
        assert_eq!(cfg.pick_provider_id(None).unwrap(), "anthropic");
    }

    #[test]
    fn pick_provider_respects_explicit_flag() {
        let mut cfg = Config::default();
        cfg.providers.insert("opencode-go".to_string(), Value::Null);
        cfg.providers.insert("anthropic".to_string(), Value::Null);
        cfg.default_provider = "opencode-go".to_string();
        assert_eq!(
            cfg.pick_provider_id(Some("anthropic")).unwrap(),
            "anthropic"
        );
    }

    #[test]
    fn pick_provider_rejects_unknown_explicit() {
        let cfg = Config::default();
        assert!(cfg.pick_provider_id(Some("does-not-exist")).is_err());
    }
}
