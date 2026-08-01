// Rolling subscription usage parser.
//
// The `/workspace/{id}/go` page embeds a TanStack-Start serialized JS payload
// where each window appears as
// `rollingUsage:$R[..]={status:"ok",resetInSec:<n>,usagePercent:<n>}`.
// The same page also carries a bare `monthlyUsage:<bigint>` cost field, so each
// candidate object is bounded to its `{...}` braces before extracting numbers —
// this skips the cost field exactly like the upstream regex `usagePercent`
// lookahead does.

use anyhow::{Result, bail};

use crate::providers::{RollingUsageView, UsageWindowView};

struct UsageWindow {
    percent: f64,
    reset_in_sec: i64,
}

struct RollingUsage {
    rolling: UsageWindow,
    weekly: UsageWindow,
    monthly: Option<UsageWindow>,
}

fn looks_signed_out(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("auth/authorize")
        || lower.contains("not associated with an account")
        || lower.contains(r#"actor of type "public""#)
}

fn number_after(obj: &str, field: &str) -> Option<f64> {
    let start = obj.find(field)? + field.len();
    let rest = obj[start..].trim_start_matches([':', ' ', '\t', '=']);
    let num: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num.parse().ok()
}

fn extract_window(text: &str, key: &str) -> Option<UsageWindow> {
    let mut from = 0;
    while let Some(pos) = text[from..].find(key) {
        let abs = from + pos;
        let rest = &text[abs..];
        let end = rest.find('}').unwrap_or(rest.len());
        let obj = &rest[..end];
        if let (Some(percent), Some(reset)) = (
            number_after(obj, "usagePercent"),
            number_after(obj, "resetInSec"),
        ) {
            return Some(UsageWindow {
                percent,
                reset_in_sec: reset as i64,
            });
        }
        from = abs + key.len();
    }
    None
}

fn parse(text: &str) -> Result<RollingUsage> {
    match (
        extract_window(text, "rollingUsage"),
        extract_window(text, "weeklyUsage"),
    ) {
        (Some(rolling), Some(weekly)) => Ok(RollingUsage {
            rolling,
            weekly,
            monthly: extract_window(text, "monthlyUsage"),
        }),
        _ if looks_signed_out(text) => {
            bail!("OpenCode GO session cookie is invalid or expired. Run 'usus opencode login'.")
        }
        _ => bail!("Could not find rolling usage data in the OpenCode GO response."),
    }
}

pub fn parse_rolling_usage(title: &str, text: &str) -> Result<RollingUsageView> {
    let usage = parse(text)?;
    let mut windows = vec![
        UsageWindowView {
            label: "5-hour",
            percent: usage.rolling.percent,
            reset_in_sec: usage.rolling.reset_in_sec,
        },
        UsageWindowView {
            label: "Weekly",
            percent: usage.weekly.percent,
            reset_in_sec: usage.weekly.reset_in_sec,
        },
    ];
    let mut monthly_reset: i64 = 0;
    if let Some(monthly) = &usage.monthly {
        windows.push(UsageWindowView {
            label: "Monthly",
            percent: monthly.percent,
            reset_in_sec: monthly.reset_in_sec,
        });
        monthly_reset = monthly.reset_in_sec;
    }
    let renews = if monthly_reset > 0 {
        let dt = chrono::Local::now() + chrono::Duration::seconds(monthly_reset);
        dt.format("%d %b %Y").to_string()
    } else {
        String::new()
    };
    Ok(RollingUsageView {
        title: title.to_string(),
        windows,
        renews,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = concat!(
        r#"...,monthlyLimit:null,monthlyUsage:8501160325,timeMonthlyUsageUpdated:$R[30]=new Date("2026-02-27T11:11:07.000Z"),"#,
        r#"reloadError:null,subscription:null,lite:$R[31]={foo:1}});$R[28]($R[18],$R[32]={mine:!0,useBalance:!1,"#,
        r#"rollingUsage:$R[33]={status:"ok",resetInSec:8639,usagePercent:18},"#,
        r#"weeklyUsage:$R[34]={status:"ok",resetInSec:228871,usagePercent:85},"#,
        r#"monthlyUsage:$R[35]={status:"ok",resetInSec:869287,usagePercent:77}});"#,
    );

    #[test]
    fn parses_all_three_windows() {
        let view = parse_rolling_usage("OpenCode GO", PAGE).unwrap();
        assert_eq!(view.windows.len(), 3);
        assert_eq!(view.windows[0].label, "5-hour");
        assert_eq!(view.windows[0].percent, 18.0);
        assert_eq!(view.windows[0].reset_in_sec, 8639);
        assert_eq!(view.windows[1].percent, 85.0);
        assert_eq!(view.windows[1].reset_in_sec, 228871);
        assert_eq!(view.windows[2].label, "Monthly");
        assert_eq!(view.windows[2].percent, 77.0);
        assert_eq!(view.windows[2].reset_in_sec, 869287);
    }

    #[test]
    fn skips_bare_monthly_usage_cost_field() {
        let usage = parse(PAGE).unwrap();
        let monthly = usage.monthly.unwrap();
        assert_eq!(monthly.percent, 77.0);
        assert_ne!(monthly.percent, 8501160325.0);
    }

    #[test]
    fn parses_fractional_percent() {
        let text = r#"rollingUsage:$R[1]={status:"ok",resetInSec:10,usagePercent:18.5},weeklyUsage:$R[2]={status:"ok",resetInSec:20,usagePercent:0}"#;
        let view = parse_rolling_usage("x", text).unwrap();
        assert_eq!(view.windows[0].percent, 18.5);
        assert_eq!(view.windows.len(), 2);
    }

    #[test]
    fn errors_when_signed_out() {
        let text = r#"<html>redirecting to /auth/authorize</html>"#;
        let err = parse_rolling_usage("x", text).unwrap_err().to_string();
        assert!(err.contains("invalid or expired"), "got: {err}");
    }

    #[test]
    fn errors_when_no_usage_data() {
        let err = parse_rolling_usage("x", "<html>nothing here</html>")
            .unwrap_err()
            .to_string();
        assert!(err.contains("Could not find rolling usage"), "got: {err}");
    }
}
