use anyhow::Result;
use console::style;
use serde_json::Value;

use crate::config::{config_path, load_or_default, save};
use crate::providers::ProviderId;
use crate::ui::prompt::prompt_sub_day;

pub fn run(provider_id: ProviderId) -> Result<()> {
    let provider = provider_id.provider();
    let blob = provider.login()?;
    persist_provider(provider.id(), blob)
}

/// Persist a freshly configured provider blob, seeding `default`/`sub_day`
/// on first setup. All fields are gathered interactively.
fn persist_provider(id: ProviderId, provider_blob: Value) -> Result<()> {
    let mut top = load_or_default()?;
    top.providers.insert(id.as_str().to_string(), provider_blob);
    if top.default_provider.is_empty() {
        top.default_provider = id.as_str().to_string();
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
