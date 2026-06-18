use anyhow::{Result, bail};
use dialoguer::Input;
use serde_json::Value;

use crate::providers::anthropic::{Config, DEFAULT_ALLOWANCE};
use crate::ui::prompt::{initial_login_message, prompt_number};

pub fn run() -> Result<Value> {
    initial_login_message(
        "Anthropic API setup",
        "You need an Admin API key:
1. Open https://console.anthropic.com/settings/admin-keys
2. Create an Admin key (starts with 'sk-ant-admin01-...')",
    );

    let admin_key: String = Input::new()
        .with_prompt("Admin API key")
        .interact_text()
        .unwrap();
    if admin_key.is_empty() {
        bail!("Admin key cannot be empty.");
    }
    if !admin_key.starts_with("sk-ant-admin") {
        bail!("This is not a valid admin key.");
    }

    let allowance: f64 = prompt_number("Monthly allowance in USD", DEFAULT_ALLOWANCE)?;

    let cfg = Config {
        admin_key,
        allowance,
    };
    Ok(serde_json::to_value(&cfg)?)
}
