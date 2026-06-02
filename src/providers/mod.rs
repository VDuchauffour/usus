use crate::billing::BillingPeriod;
use anyhow::{Context, Result, bail};

pub mod opencode_go;

pub trait Provider {
    fn id(&self) -> &'static str; // "opencode-go"
    fn display_name(&self) -> &'static str; // "OpenCode GO"
    fn allowance(&self) -> f64;
    fn currency(&self) -> &'static str;
    fn login_interactive(&self) -> Result<serde_json::Value>; // per-provider config blob
    fn fetch_report(&self, cfg: &serde_json::Value, period: &BillingPeriod) -> Result<ReportView>;
}
