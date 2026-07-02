// Report command - thin orchestrator. Provider does the work.

use anyhow::{Result, anyhow};

use crate::cli::render::get_spinner;
use crate::{
    billing::BillingPeriod,
    config::load,
    providers::by_id,
    ui::render::{render, render_rolling},
};

pub fn run(provider_flag: Option<&str>, per_keys: bool) -> Result<()> {
    let cfg = load()?;
    let provider_id = cfg.pick_provider_id(provider_flag)?;
    let provider =
        by_id(&provider_id).ok_or_else(|| anyhow!("Unknown provider id '{provider_id}'"))?;
    let provider_cfg = cfg
        .providers
        .get(&provider_id)
        .ok_or_else(|| anyhow!("Provider '{provider_id}' not configured"))?;

    if !per_keys {
        let spinner = get_spinner("Fetching usage data...");
        let rolling = provider.fetch_rolling_usage(provider_cfg);
        spinner.finish_and_clear();
        if let Some(view) = rolling? {
            render_rolling(&view)?;
            return Ok(());
        }
    }

    let period = BillingPeriod::current(cfg.sub_day);
    let spinner = get_spinner("Fetching usage data...");
    let result = provider.fetch_report(provider_cfg, &period);
    spinner.finish_and_clear();

    let view = result?;
    render(&view)?;
    Ok(())
}
