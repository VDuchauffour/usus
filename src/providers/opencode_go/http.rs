// HTTP

use anyhow::{Context as _, Result, bail};
use serde_json::json;

use crate::providers::opencode_go::Config;

const API_URL: &str = "https://opencode.ai/_server";

const PAGE_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

pub fn fetch_go_page(client: &reqwest::blocking::Client, cfg: &Config) -> Result<String> {
    let url = format!("https://opencode.ai/workspace/{}/go", cfg.workspace_id);
    let resp = client
        .get(&url)
        .header(
            "accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("accept-language", "en-GB,en;q=0.9")
        .header("cookie", format!("oc_locale=en; auth={}", cfg.auth_cookie))
        .header("user-agent", PAGE_UA)
        .send()
        .context("HTTP request failed")?;

    let status = resp.status();
    let text = resp.text().context("Reading response body")?;
    if !status.is_success() {
        bail!(
            "Failed to fetch usage page. HTTP {} - {text}",
            status.as_u16()
        );
    }
    Ok(text)
}

pub fn fetch_month(
    client: &reqwest::blocking::Client,
    cfg: &Config,
    year: i32,
    js_month: i32,
) -> Result<String> {
    let body = json!({
        "t": {
            "t": 9,
            "i": 0,
            "l": 4,
            "o": 0,
            "a": [
                { "t": 1, "s": cfg.workspace_id },
                { "t": 0, "s": year },
                { "t": 0, "s": js_month },
                { "t": 1, "s": "UTC" }
            ]
        },
        "f": cfg.function_id,
        "m": []
    });

    let resp = client
        .post(API_URL)
        .header("accept", "*/*")
        .header("accept-language", "en-GB,en;q=0.9")
        .header("content-type", "application/json")
        .header("cookie", format!("oc_locale=en; auth={}", cfg.auth_cookie))
        .header("origin", "https://opencode.ai")
        .header(
            "referer",
            format!("https://opencode.ai/workspace/{}/usage", cfg.workspace_id),
        )
        .header("x-server-id", &cfg.server_id)
        .header("x-server-instance", "server-fn:0")
        .body(serde_json::to_vec(&body)?)
        .send()
        .context("HTTP request failed")?;

    let status = resp.status();
    let text = resp.text().context("Reading response body")?;
    if !status.is_success() {
        bail!("Failed to fetch data. HTTP {} - {text}", status.as_u16());
    }
    Ok(text)
}
