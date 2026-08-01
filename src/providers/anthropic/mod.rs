use std::collections::HashMap;

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    billing::BillingPeriod,
    providers::{Provider, ProviderId, ReportView},
};

pub mod login;

const ADMIN_API_BASE: &str = "https://api.anthropic.com/v1/organizations";
const ANTHROPIC_VERSION: &str = "2023-06-01";
pub(crate) const DEFAULT_ALLOWANCE: f64 = 200.0;

/// Bucket cap per call. cost_report/usage_report return time buckets;
/// 31 covers a single month at `bucket_width=1d`.
const BUCKETS_PER_PAGE: u32 = 31;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub admin_key: String,
    #[serde(default = "default_allowance")]
    pub allowance: f64,
}

fn default_allowance() -> f64 {
    DEFAULT_ALLOWANCE
}

pub fn validate(blob: &Value) -> Result<()> {
    let _: Config = serde_json::from_value(blob.clone()).context("anthropic provider config")?;
    Ok(())
}

pub struct Anthropic;

impl Provider for Anthropic {
    fn id(&self) -> ProviderId {
        ProviderId::Anthropic
    }

    fn display_name(&self) -> &'static str {
        "Anthropic API"
    }

    fn login(&self) -> Result<Value> {
        login::run()
    }

    fn fetch_report(&self, cfg: &Value, period: &BillingPeriod) -> Result<ReportView> {
        let cfg: Config =
            serde_json::from_value(cfg.clone()).context("Parsing anthropic config")?;
        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Building HTTP client")?;
        let starting_at = format!("{}T00:00:00Z", period.start);

        let total_cost_usd = fetch_total_cost(&client, &cfg, &starting_at)?;
        let tokens_by_key = fetch_tokens_by_key(&client, &cfg, &starting_at)?;
        let names_by_id = fetch_key_names(&client, &cfg)?;

        let rows = allocate_cost_by_share(total_cost_usd, &tokens_by_key, &names_by_id);
        Ok(ReportView {
            title: self.display_name().to_string(),
            allowance: cfg.allowance,
            currency: "$",
            period_end: period.end.clone(),
            rows,
            total_cost: total_cost_usd,
        })
    }
}

fn allocate_cost_by_share(
    total_cost_usd: f64,
    tokens_by_key: &HashMap<String, u64>,
    names_by_id: &HashMap<String, String>,
) -> Vec<(String, f64)> {
    let total_tokens: u64 = tokens_by_key.values().copied().sum();
    let mut rows: Vec<(String, f64)> = Vec::with_capacity(tokens_by_key.len());

    if total_tokens == 0 {
        return rows;
    }

    for (id, tokens) in tokens_by_key {
        let name = names_by_id
            .get(id)
            .cloned()
            .unwrap_or_else(|| display_id(id));
        let share = *tokens as f64 / total_tokens as f64;
        rows.push((name, total_cost_usd * share));
    }
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    rows
}

fn display_id(id: &str) -> String {
    if id.is_empty() {
        "Workbench / unattributed".to_string()
    } else if id.len() > 12 {
        format!("{}…", &id[..12])
    } else {
        id.to_string()
    }
}

fn fetch_total_cost(
    client: &reqwest::blocking::Client,
    cfg: &Config,
    starting_at: &str,
) -> Result<f64> {
    let mut total_cents: f64 = 0.0;
    let mut page: Option<String> = None;

    loop {
        let mut req = client.get(format!("{ADMIN_API_BASE}/cost_report")).query(&[
            ("starting_at", starting_at),
            ("bucket_width", "1d"),
            ("limit", &BUCKETS_PER_PAGE.to_string()),
        ]);
        if let Some(p) = &page {
            req = req.query(&[("page", p)]);
        }
        let resp = send_admin(req, cfg)?;
        let parsed: Value = serde_json::from_str(&resp).context("Parsing cost_report JSON")?;

        if let Some(buckets) = parsed.get("data").and_then(Value::as_array) {
            for bucket in buckets {
                let Some(results) = bucket.get("results").and_then(Value::as_array) else {
                    continue;
                };
                for r in results {
                    if let Some(amount_str) = r.get("amount").and_then(Value::as_str)
                        && let Ok(cents) = amount_str.parse::<f64>()
                    {
                        total_cents += cents;
                    }
                }
            }
        }

        if parsed.get("has_more").and_then(Value::as_bool) != Some(true) {
            break;
        }
        let Some(next) = parsed.get("next_page").and_then(Value::as_str) else {
            break;
        };
        page = Some(next.to_string());
    }

    Ok(total_cents / 100.0)
}

fn fetch_tokens_by_key(
    client: &reqwest::blocking::Client,
    cfg: &Config,
    starting_at: &str,
) -> Result<HashMap<String, u64>> {
    let mut tokens_by_key: HashMap<String, u64> = HashMap::new();
    let mut page: Option<String> = None;

    loop {
        let mut req = client
            .get(format!("{ADMIN_API_BASE}/usage_report/messages"))
            .query(&[
                ("starting_at", starting_at),
                ("bucket_width", "1d"),
                ("group_by[]", "api_key_id"),
                ("limit", &BUCKETS_PER_PAGE.to_string()),
            ]);
        if let Some(p) = &page {
            req = req.query(&[("page", p)]);
        }
        let resp = send_admin(req, cfg)?;
        let parsed: Value = serde_json::from_str(&resp).context("Parsing usage_report JSON")?;

        if let Some(buckets) = parsed.get("data").and_then(Value::as_array) {
            for bucket in buckets {
                let Some(results) = bucket.get("results").and_then(Value::as_array) else {
                    continue;
                };
                for r in results {
                    let key = r
                        .get("api_key_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let tokens = sum_token_fields(r);
                    *tokens_by_key.entry(key).or_default() += tokens;
                }
            }
        }

        if parsed.get("has_more").and_then(Value::as_bool) != Some(true) {
            break;
        }
        let Some(next) = parsed.get("next_page").and_then(Value::as_str) else {
            break;
        };
        page = Some(next.to_string());
    }

    Ok(tokens_by_key)
}

fn sum_token_fields(result: &Value) -> u64 {
    const SCALAR_FIELDS: &[&str] = &[
        "uncached_input_tokens",
        "output_tokens",
        "cache_read_input_tokens",
    ];
    let mut total: u64 = 0;
    for f in SCALAR_FIELDS {
        if let Some(v) = result.get(*f).and_then(Value::as_u64) {
            total += v;
        }
    }
    if let Some(cc) = result.get("cache_creation").and_then(Value::as_object) {
        for v in cc.values() {
            if let Some(n) = v.as_u64() {
                total += n;
            }
        }
    }
    total
}

fn fetch_key_names(
    client: &reqwest::blocking::Client,
    cfg: &Config,
) -> Result<HashMap<String, String>> {
    let mut names: HashMap<String, String> = HashMap::new();
    let mut page: Option<String> = None;

    loop {
        let mut req = client
            .get(format!("{ADMIN_API_BASE}/api_keys"))
            .query(&[("limit", "100")]);
        if let Some(p) = &page {
            req = req.query(&[("after", p)]);
        }
        let resp = send_admin(req, cfg)?;
        let parsed: Value = serde_json::from_str(&resp).context("Parsing api_keys JSON")?;

        if let Some(arr) = parsed.get("data").and_then(Value::as_array) {
            for k in arr {
                let id = k.get("id").and_then(Value::as_str).unwrap_or("");
                let name = k.get("name").and_then(Value::as_str).unwrap_or("");
                if !id.is_empty() {
                    names.insert(
                        id.to_string(),
                        if name.is_empty() {
                            id.to_string()
                        } else {
                            name.to_string()
                        },
                    );
                }
            }
        }

        if parsed.get("has_more").and_then(Value::as_bool) != Some(true) {
            break;
        }
        let Some(next) = parsed.get("next_page").and_then(Value::as_str) else {
            break;
        };
        page = Some(next.to_string());
    }

    Ok(names)
}

fn send_admin(req: reqwest::blocking::RequestBuilder, cfg: &Config) -> Result<String> {
    let resp = req
        .header("x-api-key", &cfg.admin_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("accept", "application/json")
        .send()
        .context("Anthropic Admin API request failed")?;
    let status = resp.status();
    let text = resp.text().context("Reading response body")?;
    if !status.is_success() {
        bail!(
            "Anthropic Admin API returned HTTP {} - {text}",
            status.as_u16()
        );
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn sum_token_fields_handles_missing_and_cache_creation_object() {
        let r = json!({
            "uncached_input_tokens": 100,
            "output_tokens": 50,
            "cache_read_input_tokens": 25,
            "cache_creation": { "ephemeral_5m_input_tokens": 200, "ephemeral_1h_input_tokens": 300 }
        });
        assert_eq!(sum_token_fields(&r), 100 + 50 + 25 + 200 + 300);
    }

    #[test]
    fn sum_token_fields_ignores_non_numeric() {
        let r = json!({ "output_tokens": "bogus", "uncached_input_tokens": 10 });
        assert_eq!(sum_token_fields(&r), 10);
    }

    #[test]
    fn allocate_cost_by_share_distributes_proportionally() {
        let mut tokens = HashMap::new();
        tokens.insert("k1".to_string(), 300u64);
        tokens.insert("k2".to_string(), 100u64);
        let mut names = HashMap::new();
        names.insert("k1".to_string(), "Alice".to_string());
        names.insert("k2".to_string(), "Bob".to_string());

        let rows = allocate_cost_by_share(40.0, &tokens, &names);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "Alice");
        assert!((rows[0].1 - 30.0).abs() < 1e-9);
        assert_eq!(rows[1].0, "Bob");
        assert!((rows[1].1 - 10.0).abs() < 1e-9);
    }

    #[test]
    fn allocate_cost_by_share_returns_empty_when_no_tokens() {
        let tokens = HashMap::new();
        let names = HashMap::new();
        let rows = allocate_cost_by_share(40.0, &tokens, &names);
        assert!(rows.is_empty());
    }

    #[test]
    fn allocate_cost_by_share_uses_id_when_name_missing() {
        let mut tokens = HashMap::new();
        tokens.insert("apikey_01abcdefghij".to_string(), 10u64);
        let names = HashMap::new();
        let rows = allocate_cost_by_share(5.0, &tokens, &names);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].0.starts_with("apikey_01abc"));
    }

    #[test]
    fn display_id_handles_empty() {
        assert_eq!(display_id(""), "Workbench / unattributed");
    }
}
