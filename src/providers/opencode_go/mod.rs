use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::billing::BillingPeriod;
use crate::providers::{Provider, ReportView};

pub mod http;
pub mod login;
pub mod parser;

pub const ID: &str = "opencode-go";

const ALLOWANCE: f64 = 60.0;
const COST_DIVISOR: f64 = 100_000_000.0;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "authCookie")]
    pub auth_cookie: String,
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "functionId")]
    pub function_id: i64,
}

pub struct OpenCodeGo;

impl Provider for OpenCodeGo {
    fn id(&self) -> &'static str {
        ID
    }

    fn display_name(&self) -> &'static str {
        "OpenCode GO"
    }

    fn fetch_report(&self, cfg: &Value, period: &BillingPeriod) -> Result<ReportView> {
        let cfg: Config =
            serde_json::from_value(cfg.clone()).context("Parsing opencode-go config")?;
        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Building HTTP client")?;

        let mut all_usage: Vec<Value> = Vec::new();
        let mut all_keys: Vec<Value> = Vec::new();
        let mut seen_keys: HashSet<String> = HashSet::new();

        for (year, month) in &period.months_to_fetch {
            let js_month = month - 1;
            let text = http::fetch_month(&client, &cfg, *year, js_month)?;
            let (usage, keys) = parser::extract_data(&text)?;
            all_usage.extend(usage);
            for k in keys {
                let id = k
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if seen_keys.insert(id) {
                    all_keys.push(k);
                }
            }
        }

        let (rows, total_cost) = aggregate(&all_usage, &all_keys);
        Ok(ReportView {
            title: self.display_name().to_string(),
            allowance: ALLOWANCE,
            currency: "$",
            period_end: period.end.clone(),
            rows,
            total_cost,
        })
    }
}

struct KeyInfo {
    cost: f64,
    name: String,
    deleted: bool,
}

fn aggregate(all_usage: &[Value], all_keys: &[Value]) -> (Vec<(String, f64)>, f64) {
    let mut key_costs: HashMap<String, KeyInfo> = HashMap::new();

    for k in all_keys {
        let deleted = k.get("deleted").and_then(Value::as_bool).unwrap_or(false);
        if deleted {
            continue;
        }
        let id = k
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let name = k
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or("Unknown Key")
            .to_string();
        key_costs.insert(
            id,
            KeyInfo {
                cost: 0.0,
                name,
                deleted: false,
            },
        );
    }

    for row in all_usage {
        let plan = row.get("plan").and_then(Value::as_str).unwrap_or("");
        if plan != "sub" && plan != "lite" {
            continue;
        }
        let key_id = row
            .get("keyId")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let cost = row.get("totalCost").and_then(Value::as_f64).unwrap_or(0.0);

        let entry = key_costs.entry(key_id.clone()).or_insert_with(|| {
            let info = all_keys
                .iter()
                .find(|k| k.get("id").and_then(Value::as_str) == Some(&key_id));
            KeyInfo {
                cost: 0.0,
                name: info
                    .and_then(|k| k.get("displayName"))
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown Key")
                    .to_string(),
                deleted: info
                    .and_then(|k| k.get("deleted"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            }
        });
        entry.cost += cost;
    }

    let mut total_cost = 0.0;
    let mut results: Vec<(String, f64)> = Vec::new();
    for info in key_costs.values() {
        if info.deleted {
            continue;
        }
        let dollars = info.cost / COST_DIVISOR;
        total_cost += dollars;
        results.push((info.name.clone(), dollars));
    }
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    (results, total_cost)
}
