use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::providers::{Provider, ProviderId, RollingUsageView};

pub mod http;
pub mod login;
pub mod rolling;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub auth_cookie: String,
    pub workspace_id: String,
    pub server_id: String,
    pub function_id: i64,
}

pub fn validate(blob: &Value) -> Result<()> {
    let _: Config = serde_json::from_value(blob.clone()).context("opencode provider config")?;
    Ok(())
}

pub struct OpenCodeGo;

impl Provider for OpenCodeGo {
    fn id(&self) -> ProviderId {
        ProviderId::OpencodeGo
    }

    fn display_name(&self) -> &'static str {
        "OpenCode GO"
    }

    fn login(&self) -> Result<Value> {
        login::run()
    }

    fn fetch_rolling_usage(&self, cfg: &Value) -> Result<Option<RollingUsageView>> {
        let cfg: Config = serde_json::from_value(cfg.clone()).context("Parsing opencode config")?;
        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Building HTTP client")?;
        let text = http::fetch_go_page(&client, &cfg)?;
        Ok(Some(rolling::parse_rolling_usage(
            self.display_name(),
            &text,
        )?))
    }
}
