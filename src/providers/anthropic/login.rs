use anyhow::{Result, anyhow, bail};

use crate::config::{config_path, load_or_default, save};
use crate::helper::{prompt_sub_day, read_line, read_with_default};
use crate::providers::anthropic::{Config, DEFAULT_ALLOWANCE, ID};
use crate::style::{BOLD, DIM, GREEN, RESET};

pub fn cmd_login(
    admin_key: Option<String>,
    allowance: Option<f64>,
    sub_day: Option<u32>,
) -> Result<()> {
    println!("{BOLD}Anthropic API setup{RESET}");
    println!();
    println!("You need an Admin API key:");
    println!("  1. Open https://console.anthropic.com/settings/admin-keys");
    println!("  2. Create an Admin key (starts with 'sk-ant-admin01-...')");
    println!();

    let admin_key = match admin_key {
        Some(v) => v,
        None => read_line("Paste your Admin API key: ")?,
    };
    if admin_key.is_empty() {
        bail!("Admin key cannot be empty.");
    }
    if !admin_key.starts_with("sk-ant-admin") {
        bail!("This is not a valid admin key.")
    }

    let allowance = match allowance {
        Some(v) => v,
        None => {
            let s = read_with_default(
                &format!("Monthly allowance in USD [{DEFAULT_ALLOWANCE}]: "),
                &DEFAULT_ALLOWANCE.to_string(),
            )?;
            s.parse()
                .map_err(|_| anyhow!("Allowance must be a number"))?
        }
    };

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
                println!("{DIM}(e.g., if you subscribed on 20-Apr, enter 20){RESET}");
                prompt_sub_day()?
            }
        };
    } else if let Some(v) = sub_day {
        top.sub_day = v;
    }

    save(&top)?;
    println!(
        "{GREEN}Provider '{ID}' saved to {}{RESET}",
        config_path()?.display()
    );
    Ok(())
}
