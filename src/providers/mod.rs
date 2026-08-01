// Provider abstraction. Each provider owns its HTTP, parsing, and aggregation;
// the orchestrator only consumes ReportView.

use std::{fmt, str::FromStr};

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
    pub renews: String,
}

pub trait Provider {
    fn id(&self) -> ProviderId;
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

/// A known provider identifier — the type-safe replacement for the
/// `"anthropic"` / `"opencode"` string literals that used to be threaded
/// through the CLI and config layers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProviderId {
    Anthropic,
    OpencodeGo,
}

impl ProviderId {
    /// All known provider ids, in the order providers are matched against the
    /// config map. Kept stable for error-message output.
    pub const ALL: &'static [ProviderId] = &[ProviderId::OpencodeGo, ProviderId::Anthropic];

    /// The string form used as the TOML config key and CLI verb for this
    /// provider (e.g. `"anthropic"`, `"opencode"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpencodeGo => "opencode",
        }
    }

    /// Instantiate the concrete [`Provider`] implementation for this id.
    pub fn provider(self) -> Box<dyn Provider> {
        match self {
            Self::OpencodeGo => Box::new(opencode_go::OpenCodeGo),
            Self::Anthropic => Box::new(anthropic::Anthropic),
        }
    }

    /// Validate a raw config blob against this provider's expected schema.
    pub fn validate_blob(self, blob: &Value) -> Result<()> {
        match self {
            Self::OpencodeGo => opencode_go::validate(blob),
            Self::Anthropic => anthropic::validate(blob),
        }
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "anthropic" => Ok(Self::Anthropic),
            "opencode" => Ok(Self::OpencodeGo),
            other => bail!(
                "Unknown provider id '{other}'. Known: {}",
                Self::ALL
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
