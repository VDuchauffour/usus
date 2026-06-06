use anyhow::{Result, bail};
use console::style;
use dialoguer::Input;

use crate::cli::prompt::prompt_number;
use crate::config::{config_path, load_or_default, save};
use crate::helper::{initial_login_message, prompt_sub_day};
use crate::providers::anthropic::{Config, DEFAULT_ALLOWANCE, ID};

pub fn cmd_login(
    admin_key: Option<String>,
    allowance: Option<f64>,
    sub_day: Option<u32>,
) -> Result<()> {
    initial_login_message(
        "Anthropic API setup",
        "You need an Admin API key:
1. Open https://console.anthropic.com/settings/admin-keys
2. Create an Admin key (starts with 'sk-ant-admin01-...')",
    );

    let admin_key: String = match admin_key {
        Some(v) => v,
        None => Input::new()
            .with_prompt("Admin API key")
            .interact_text()
            .unwrap(),
    };
    if admin_key.is_empty() {
        bail!("Admin key cannot be empty.");
    }
    if !admin_key.starts_with("sk-ant-admin") {
        bail!("This is not a valid admin key.");
    }

    let allowance: f64 = prompt_number(allowance, "Monthly allowance in USD", DEFAULT_ALLOWANCE)?;

    let cfg = Config {
        admin_key,
        allowance,
    };
    let provider_blob = serde_json::to_value(&cfg)?;

    let mut top = load_or_default()?;
    top.providers.insert(ID.to_string(), provider_blob);
    if top.default.is_empty() {
        top.default = ID.to_string();
    }
    if top.sub_day == 0 {
        top.sub_day = match sub_day {
            Some(v) => v,
            None => {
                println!();
                println!("What day of the month does your billing cycle start?");
                println!(
                    "{}",
                    style("(e.g., if you subscribed on 20-Apr, enter 20)").dim()
                );
                prompt_sub_day()?
            }
        };
    } else if let Some(v) = sub_day {
        top.sub_day = v;
    }

    save(&top)?;
    println!(
        "{}",
        style(format!(
            "Provider '{ID}' saved to {}",
            config_path()?.display()
        ))
        .green()
    );
    Ok(())
}
