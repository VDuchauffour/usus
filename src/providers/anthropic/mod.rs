use anyhow::{Context as _, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::providers::{Provider, ProviderId, RollingUsageView, UsageWindowView};

pub mod login;

/// Personal rate-limit usage endpoint (Claude Code OAuth).
const OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20";
/// Fallback User-Agent when the installed Claude Code version can't be detected.
const FALLBACK_CLAUDE_CODE_VERSION: &str = "2.1.0";
/// Path to the Claude Code OAuth credentials file, relative to the home dir.
const CREDENTIALS_REL_PATH: &str = ".claude/.credentials.json";

/// Anthropic provider configuration.
///
/// The provider reads Claude Code OAuth credentials from
/// `~/.claude/.credentials.json` (created by `claude login`) for personal
/// rate-limit usage. No API key or config fields are required.
#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {}

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
        "Anthropic"
    }

    fn login(&self) -> Result<Value> {
        login::run()
    }

    fn fetch_rolling_usage(&self, _cfg: &Value) -> Result<Option<RollingUsageView>> {
        let token = read_oauth_token()?;
        let client = reqwest::blocking::Client::builder()
            .build()
            .context("Building HTTP client")?;
        let resp = client
            .get(OAUTH_USAGE_URL)
            .header("authorization", format!("Bearer {token}"))
            .header("anthropic-beta", OAUTH_BETA_HEADER)
            .header("accept", "application/json")
            .header("user-agent", claude_code_user_agent())
            .send()
            .context("Fetching OAuth usage")?;
        let status = resp.status();
        let text = resp.text().context("Reading OAuth usage response")?;
        if !status.is_success() {
            if status.as_u16() == 401 {
                bail!(
                    "Claude Code OAuth token is expired or invalid. \
                     Run 'claude login' to refresh, then try again."
                );
            }
            bail!(
                "Anthropic OAuth usage returned HTTP {} - {text}",
                status.as_u16()
            );
        }
        let view = parse_oauth_usage(&text, self.display_name())?;
        Ok(Some(view))
    }
}

/// Read the OAuth access token from `~/.claude/.credentials.json`.
fn read_oauth_token() -> Result<String> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let path = home.join(CREDENTIALS_REL_PATH);
    if !path.exists() {
        bail!(
            "Claude Code credentials not found at {}.\n\
             Run 'claude login' first, then run 'usus anthropic login'.",
            path.display()
        );
    }
    let raw =
        std::fs::read_to_string(&path).with_context(|| format!("Reading {}", path.display()))?;
    let parsed: Value =
        serde_json::from_str(&raw).with_context(|| format!("Parsing {}", path.display()))?;
    let token = parsed
        .get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No accessToken in {}.\n\
                 Run 'claude login' to authenticate.",
                path.display()
            )
        })?;
    if token.is_empty() {
        bail!(
            "accessToken in {} is empty. Run 'claude login' again.",
            path.display()
        );
    }
    Ok(token.to_string())
}

fn claude_code_user_agent() -> String {
    format!("claude-code/{FALLBACK_CLAUDE_CODE_VERSION}")
}

/// Parsed OAuth usage response. Only the fields we care about; unknown keys
/// (e.g. `limits`, `extra_usage`) are silently ignored by serde.
#[derive(Deserialize)]
struct OAuthUsageResponse {
    #[serde(rename = "five_hour", default)]
    five_hour: Option<OAuthWindow>,
    #[serde(rename = "seven_day", default)]
    seven_day: Option<OAuthWindow>,
    #[serde(rename = "seven_day_opus", default)]
    seven_day_opus: Option<OAuthWindow>,
    #[serde(rename = "seven_day_sonnet", default)]
    seven_day_sonnet: Option<OAuthWindow>,
}

#[derive(Deserialize, Clone)]
struct OAuthWindow {
    #[serde(default)]
    utilization: Option<f64>,
    #[serde(rename = "resets_at", default)]
    resets_at: Option<String>,
}

fn parse_oauth_usage(text: &str, title: &str) -> Result<RollingUsageView> {
    let resp: OAuthUsageResponse =
        serde_json::from_str(text).context("Parsing OAuth usage response")?;
    let now = Utc::now();
    let mut windows = Vec::new();

    push_window(&mut windows, resp.five_hour, "5-hour", now);
    let weekly = resp.seven_day;
    push_window(&mut windows, weekly.clone(), "Weekly", now);
    push_window(&mut windows, resp.seven_day_opus, "Weekly Opus", now);
    push_window(&mut windows, resp.seven_day_sonnet, "Weekly Sonnet", now);

    if windows.is_empty() {
        bail!("No usage windows found in OAuth response: {text}");
    }

    let renews = weekly
        .as_ref()
        .and_then(|w| w.resets_at.as_deref())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc).format("%d %b %Y").to_string())
        .unwrap_or_default();

    Ok(RollingUsageView {
        title: title.to_string(),
        windows,
        renews,
    })
}

fn push_window(
    windows: &mut Vec<UsageWindowView>,
    window: Option<OAuthWindow>,
    label: &'static str,
    now: DateTime<Utc>,
) {
    if let Some(w) = window
        && let Some(pct) = w.utilization
    {
        windows.push(UsageWindowView {
            label,
            percent: pct,
            reset_in_sec: reset_seconds(w.resets_at.as_deref(), now),
        });
    }
}

fn reset_seconds(resets_at: Option<&str>, now: DateTime<Utc>) -> i64 {
    let Some(s) = resets_at else { return 0 };
    let Ok(dt) = DateTime::parse_from_rfc3339(s) else {
        return 0;
    };
    let dt_utc = dt.with_timezone(&Utc);
    (dt_utc - now).num_seconds().max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_oauth_usage_parses_both_windows() {
        let json = r#"{
            "five_hour": { "utilization": 7, "resets_at": "2025-12-23T16:00:00.000Z" },
            "seven_day": { "utilization": 21, "resets_at": "2025-12-29T23:00:00.000Z" }
        }"#;
        let view = parse_oauth_usage(json, "Anthropic").unwrap();
        assert_eq!(view.title, "Anthropic");
        assert_eq!(view.windows.len(), 2);
        assert_eq!(view.windows[0].label, "5-hour");
        assert_eq!(view.windows[0].percent, 7.0);
        assert_eq!(view.windows[1].label, "Weekly");
        assert_eq!(view.windows[1].percent, 21.0);
    }

    #[test]
    fn parse_oauth_usage_includes_model_windows() {
        let json = r#"{
            "five_hour": { "utilization": 7, "resets_at": "2025-12-23T16:00:00.000Z" },
            "seven_day": { "utilization": 21, "resets_at": "2025-12-29T23:00:00.000Z" },
            "seven_day_opus": { "utilization": 42, "resets_at": "2025-12-29T23:00:00.000Z" },
            "seven_day_sonnet": { "utilization": 10, "resets_at": "2025-12-29T23:00:00.000Z" }
        }"#;
        let view = parse_oauth_usage(json, "Anthropic").unwrap();
        assert_eq!(view.windows.len(), 4);
        assert_eq!(view.windows[2].label, "Weekly Opus");
        assert_eq!(view.windows[2].percent, 42.0);
        assert_eq!(view.windows[3].label, "Weekly Sonnet");
        assert_eq!(view.windows[3].percent, 10.0);
    }

    #[test]
    fn parse_oauth_usage_errors_when_empty() {
        let json = r#"{}"#;
        let err = parse_oauth_usage(json, "Anthropic")
            .unwrap_err()
            .to_string();
        assert!(err.contains("No usage windows"), "got: {err}");
    }

    #[test]
    fn reset_seconds_returns_zero_for_missing() {
        assert_eq!(reset_seconds(None, Utc::now()), 0);
        assert_eq!(reset_seconds(Some("garbage"), Utc::now()), 0);
    }

    #[test]
    fn reset_seconds_parses_future_timestamp() {
        let future = "2099-01-01T00:00:00Z";
        let secs = reset_seconds(Some(future), Utc::now());
        assert!(secs > 0, "expected positive seconds, got {secs}");
    }

    #[test]
    fn reset_seconds_returns_zero_for_past_timestamp() {
        let past = "2000-01-01T00:00:00Z";
        let secs = reset_seconds(Some(past), Utc::now());
        assert_eq!(secs, 0);
    }
}
