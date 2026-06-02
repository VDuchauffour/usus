use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub default: String,
    #[serde(default)]
    pub sub_day: u32,
    #[serde(default)]
    pub providers: BTreeMap<String, Value>,
}

pub fn config_dir() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".config/usus"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        let prog = env::args().next().unwrap_or_else(|| "usus".into());
        bail!("Configuration not found. Run '{prog} login <provider>' first.");
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Reading config at {}", path.display()))?;
    serde_json::from_str(&raw).context("Parsing config JSON")
}

pub fn load_or_default() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Reading config at {}", path.display()))?;
    serde_json::from_str(&raw).context("Parsing config JSON")
}

pub fn save(cfg: &Config) -> Result<()> {
    fs::create_dir_all(config_dir()?)?;
    let path = config_path()?;
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, json).with_context(|| format!("Writing config to {}", path.display()))?;
    Ok(())
}

pub fn pick_provider_id(cfg: &Config, explicit: Option<&str>) -> Result<String> {
    if let Some(id) = explicit {
        if cfg.providers.contains_key(id) {
            return Ok(id.to_string());
        }
        let prog = env::args().next().unwrap_or_else(|| "usus".into());
        bail!("Provider '{id}' is not configured. Run '{prog} login {id}' first.");
    }
    if !cfg.default.is_empty() && cfg.providers.contains_key(&cfg.default) {
        return Ok(cfg.default.clone());
    }
    if cfg.providers.len() == 1 {
        return Ok(cfg.providers.keys().next().unwrap().clone());
    }
    let configured: Vec<String> = cfg.providers.keys().cloned().collect();
    bail!(
        "No default provider configured. Pass --provider <id>. Configured: {}",
        if configured.is_empty() {
            "none".to_string()
        } else {
            configured.join(", ")
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_namespaced_config() {
        let raw = r#"{
            "default": "anthropic",
            "sub_day": 5,
            "providers": {
                "anthropic": { "admin_key": "sk-ant-admin-x" },
                "opencode-go": { "authCookie": "x", "workspaceId": "w", "serverId": "s", "functionId": 31 }
            }
        }"#;
        let cfg: Config = serde_json::from_str(raw).unwrap();
        assert_eq!(cfg.default, "anthropic");
        assert_eq!(cfg.sub_day, 5);
        assert_eq!(cfg.providers.len(), 2);
    }

    #[test]
    fn pick_provider_falls_back_to_single_configured() {
        let mut cfg = Config::default();
        cfg.providers.insert("anthropic".to_string(), Value::Null);
        assert_eq!(pick_provider_id(&cfg, None).unwrap(), "anthropic");
    }

    #[test]
    fn pick_provider_respects_explicit_flag() {
        let mut cfg = Config::default();
        cfg.providers.insert("opencode-go".to_string(), Value::Null);
        cfg.providers.insert("anthropic".to_string(), Value::Null);
        cfg.default = "opencode-go".to_string();
        assert_eq!(
            pick_provider_id(&cfg, Some("anthropic")).unwrap(),
            "anthropic"
        );
    }

    #[test]
    fn pick_provider_rejects_unknown_explicit() {
        let cfg = Config::default();
        assert!(pick_provider_id(&cfg, Some("does-not-exist")).is_err());
    }
}
