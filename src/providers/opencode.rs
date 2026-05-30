use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "authCookie")]
    pub auth_cookie: String,
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "functionId")]
    pub function_id: i64,
    #[serde(rename = "subDay", default = "default_sub_day")]
    pub sub_day: u32,
}

fn default_sub_day() -> u32 {
    1
}

pub fn config_dir() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home.join(".config/usus"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        let prog = env::args().next().unwrap_or_else(|| "usage".into());
        bail!("Configuration not found. Run '{prog} login' first.");
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("Reading config at {}", path.display()))?;
    serde_json::from_str(&raw).context("Parsing config JSON")
}

pub fn save_config(cfg: &Config) -> Result<()> {
    fs::create_dir_all(config_dir()?)?;
    let path = config_path()?;
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, json).with_context(|| format!("Writing config to {}", path.display()))?;
    Ok(())
}
