// Report command - thin orchestrator. Provider does the work.

use anyhow::{Result, anyhow};

use crate::cli::render::get_spinner;
use crate::{config::load, providers::ProviderId, ui::render::render_rolling};

pub fn run(provider_flag: Option<ProviderId>) -> Result<()> {
    let cfg = load()?;
    let provider_id = cfg.pick_provider_id(provider_flag)?;
    let provider = provider_id.provider();
    let provider_cfg = cfg
        .providers
        .get(provider_id.as_str())
        .ok_or_else(|| anyhow!("Provider '{provider_id}' not configured"))?;

    let spinner = get_spinner("Fetching usage data...");
    let rolling = provider.fetch_rolling_usage(provider_cfg);
    spinner.finish_and_clear();

    let view = rolling?
        .ok_or_else(|| anyhow!("Provider '{provider_id}' does not support rolling usage"))?;
    render_rolling(&view)?;
    Ok(())
}
