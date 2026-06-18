use anyhow::{Result, anyhow};
use console::style;
use serde_json::Value;

use crate::cli::render::prompt_sub_day;
use crate::config::{config_path, load_or_default, save};
use crate::providers::by_id;

pub fn run(provider_id: &str) -> Result<()> {
    let provider =
        by_id(provider_id).ok_or_else(|| anyhow!("Unknown provider id '{provider_id}'"))?;
    let blob = provider.login()?;
    persist_provider(provider.id(), blob)
}

/// Persist a freshly configured provider blob, seeding `default`/`sub_day`
/// on first setup. All fields are gathered interactively.
fn persist_provider(id: &str, provider_blob: Value) -> Result<()> {
    let mut top = load_or_default()?;
    top.providers.insert(id.to_string(), provider_blob);
    if top.default_provider.is_empty() {
        top.default_provider = id.to_string();
    }
    if top.sub_day == 0 {
        println!();
        println!("What day of the month does your billing cycle start?");
        println!(
            "{}",
            style("(e.g., if you subscribed on 20-Apr, enter 20)").dim()
        );
        top.sub_day = prompt_sub_day()?;
    }

    save(&top)?;
    println!(
        "{}",
        style(format!(
            "Provider '{id}' saved to {}",
            config_path()?.display()
        ))
        .green()
    );
    Ok(())
}
