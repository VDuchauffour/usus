// Provider abstraction. Each provider owns its HTTP, parsing, and aggregation;
// the orchestrator only consumes ReportView.

use anyhow::{Result, bail};
use serde_json::Value;

use crate::billing::BillingPeriod;

pub mod anthropic;
pub mod opencode_go;

pub struct ReportView {
    pub title: String,
    /// Monthly allowance in `currency` units (e.g. 60.0 for $60).
    pub allowance: f64,
    pub currency: &'static str,
    /// Pre-formatted period end, e.g. "15 Jun 2026".
    pub period_end: String,
    pub rows: Vec<(String, f64)>,
    pub total_cost: f64,
}

/// A single rolling usage window (e.g. the 5-hour, weekly, or monthly bucket).
#[derive(Debug)]
pub struct UsageWindowView {
    pub label: &'static str,
    /// Percentage of the window's allowance consumed, 0-100.
    pub percent: f64,
    pub reset_in_sec: i64,
}

/// Rolling subscription usage as reported by the provider's dashboard.
#[derive(Debug)]
pub struct RollingUsageView {
    pub title: String,
    pub windows: Vec<UsageWindowView>,
}

pub trait Provider {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn fetch_report(&self, cfg: &Value, period: &BillingPeriod) -> Result<ReportView>;
    fn login(&self) -> Result<Value>;

    /// Fetch the rolling subscription usage windows, if the provider exposes
    /// them. Returns `Ok(None)` when the provider has no rolling-usage concept,
    /// letting the caller fall back to the per-key cost report.
    fn fetch_rolling_usage(&self, _cfg: &Value) -> Result<Option<RollingUsageView>> {
        Ok(None)
    }
}

pub fn by_id(id: &str) -> Option<Box<dyn Provider>> {
    match id {
        opencode_go::ID => Some(Box::new(opencode_go::OpenCodeGo)),
        anthropic::ID => Some(Box::new(anthropic::Anthropic)),
        _ => None,
    }
}

pub const ALL_IDS: &[&str] = &[opencode_go::ID, anthropic::ID];

pub fn validate_provider_blob(id: &str, blob: &Value) -> Result<()> {
    match id {
        opencode_go::ID => opencode_go::validate(blob),
        anthropic::ID => anthropic::validate(blob),
        other => bail!(
            "Unknown provider id '{other}'. Known: {}",
            ALL_IDS.join(", ")
        ),
    }
}
