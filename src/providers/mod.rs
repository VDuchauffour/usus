// Provider abstraction. Each provider owns its HTTP, parsing, and aggregation;
// the orchestrator only consumes ReportView.

use anyhow::Result;
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

pub trait Provider {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn fetch_report(&self, cfg: &Value, period: &BillingPeriod) -> Result<ReportView>;
}

pub fn by_id(id: &str) -> Option<Box<dyn Provider>> {
    match id {
        opencode_go::ID => Some(Box::new(opencode_go::OpenCodeGo)),
        anthropic::ID => Some(Box::new(anthropic::Anthropic)),
        _ => None,
    }
}

pub const ALL_IDS: &[&str] = &[opencode_go::ID, anthropic::ID];
