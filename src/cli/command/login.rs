use anyhow::Result;
use console::style;
use serde_json::Value;

use crate::config::{config_path, load_or_default, save};
use crate::providers::ProviderId;

pub fn run(provider_id: ProviderId) -> Result<()> {
    let provider = provider_id.provider();
    let blob = provider.login()?;
    persist_provider(provider.id(), blob)
}

fn persist_provider(id: ProviderId, provider_blob: Value) -> Result<()> {
    let mut top = load_or_default()?;
    top.providers.insert(id.as_str().to_string(), provider_blob);
    if top.default_provider.is_empty() {
        top.default_provider = id.as_str().to_string();
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
