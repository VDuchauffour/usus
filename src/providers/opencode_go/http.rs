// HTTP

use anyhow::{Context as _, Result, bail};

use crate::providers::opencode_go::Config;

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
