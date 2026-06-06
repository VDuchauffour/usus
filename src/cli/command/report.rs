// Report command - thin orchestrator. Provider does the work.

use std::time::Duration;

use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{billing::BillingPeriod, config::load, providers::by_id, ui::render::render};

fn get_spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    spinner.set_message("Fetching usage data...");
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

pub fn run(provider_flag: Option<&str>) -> Result<()> {
    let cfg = load()?;
    let provider_id = cfg.pick_provider_id(provider_flag)?;
    let provider =
        by_id(&provider_id).ok_or_else(|| anyhow!("Unknown provider id '{provider_id}'"))?;
    let provider_cfg = cfg
        .providers
        .get(&provider_id)
        .ok_or_else(|| anyhow!("Provider '{provider_id}' not configured"))?;

    let period = BillingPeriod::current(cfg.sub_day);
    let spinner = get_spinner();
    let result = provider.fetch_report(provider_cfg, &period);
    spinner.finish_and_clear();

    let view = result?;
    render(&view)?;
    Ok(())
}
