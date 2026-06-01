use anyhow::{Context as _, Result};
use chrono::{Datelike, Local};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::http::fetch_month;
use crate::parser::extract_data;
use crate::providers::opencode_go::load_config;
use crate::render::{COST_DIVISOR, render};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub struct BillingPeriod {
    pub months_to_fetch: Vec<(i32, i32)>, // (year, month_1_indexed)
    pub end: String,
}

pub fn compute_billing_period(
    now_day: i32,
    now_month: i32,
    now_year: i32,
    sub_day: i32,
) -> BillingPeriod {
    if now_day < sub_day {
        let (prev_month, prev_year) = if now_month == 1 {
            (12, now_year - 1)
        } else {
            (now_month - 1, now_year)
        };
        BillingPeriod {
            months_to_fetch: vec![(prev_year, prev_month), (now_year, now_month)],
            end: format!(
                "{} {} {}",
                sub_day,
                MONTH_NAMES[(now_month - 1) as usize],
                now_year
            ),
        }
    } else {
        let (next_month, next_year) = if now_month == 12 {
            (1, now_year + 1)
        } else {
            (now_month + 1, now_year)
        };
        BillingPeriod {
            months_to_fetch: vec![(now_year, now_month)],
            end: format!(
                "{} {} {}",
                sub_day,
                MONTH_NAMES[(next_month - 1) as usize],
                next_year
            ),
        }
    }
}

#[derive(Default)]
struct KeyInfo {
    cost: f64,
    name: String,
    deleted: bool,
}

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

pub fn cmd_report() -> Result<()> {
    let cfg = load_config()?;

    let now = Local::now();
    let period = compute_billing_period(
        now.day() as i32,
        now.month() as i32,
        now.year(),
        cfg.sub_day as i32,
    );

    let spinner = get_spinner();

    let client = reqwest::blocking::Client::builder()
        .build()
        .context("Building HTTP client")?;

    let mut responses = Vec::with_capacity(period.months_to_fetch.len());
    for (year, month) in &period.months_to_fetch {
        // spinner.set_message(format!(
        //     "Fetching {} {}...",
        //     MONTH_NAMES[(*month - 1) as usize],
        //     year
        // ));
        let js_month = month - 1;
        responses.push(fetch_month(&client, &cfg, *year, js_month)?);
    }

    // Aggregate across all months.
    let mut all_usage: Vec<Value> = Vec::new();
    let mut all_keys: Vec<Value> = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

    for r in &responses {
        let (usage, keys) = extract_data(r)?;
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

    let mut key_costs: HashMap<String, KeyInfo> = HashMap::new();

    // Seed entries for non-deleted known keys so zero-cost keys appear.
    for k in &all_keys {
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

    for row in &all_usage {
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

    spinner.finish_and_clear();

    render(&results, total_cost, &period.end)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_month_only_when_past_sub_day() {
        let p = compute_billing_period(20, 5, 2026, 15);
        assert_eq!(p.months_to_fetch, vec![(2026, 5)]);
        assert_eq!(p.end, "15 Jun 2026");
    }

    #[test]
    fn fetches_previous_month_when_before_sub_day() {
        let p = compute_billing_period(10, 5, 2026, 15);
        assert_eq!(p.months_to_fetch, vec![(2026, 4), (2026, 5)]);
        assert_eq!(p.end, "15 May 2026");
    }

    #[test]
    fn january_before_sub_day_wraps_to_previous_december() {
        let p = compute_billing_period(5, 1, 2026, 20);
        assert_eq!(p.months_to_fetch, vec![(2025, 12), (2026, 1)]);
        assert_eq!(p.end, "20 Jan 2026");
    }

    #[test]
    fn december_past_sub_day_wraps_end_to_next_january() {
        let p = compute_billing_period(25, 12, 2026, 20);
        assert_eq!(p.months_to_fetch, vec![(2026, 12)]);
        assert_eq!(p.end, "20 Jan 2027");
    }

    #[test]
    fn boundary_now_day_equals_sub_day_uses_current_month_only() {
        let p = compute_billing_period(15, 5, 2026, 15);
        assert_eq!(p.months_to_fetch, vec![(2026, 5)]);
        assert_eq!(p.end, "15 Jun 2026");
    }
}
