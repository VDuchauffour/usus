use anyhow::{bail, Result};
use console::style;
use serde_json::json;

use crate::ui::prompt::initial_login_message;

pub fn run() -> Result<serde_json::Value> {
    initial_login_message(
        "Anthropic setup",
        "usus reads your Claude Code OAuth credentials to show personal rate-limit usage.\n\
         1. Install Claude Code: https://claude.com/claude-code\n\
         2. Run `claude login` and complete the browser flow\n\
         3. This stores credentials at ~/.claude/.credentials.json",
    );

    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let path = home.join(".claude/.credentials.json");
    if !path.exists() {
        bail!(
            "Credentials not found at {}.\n\
             Run `claude login` first, then re-run `usus anthropic login`.",
            path.display()
        );
    }

    // Verify the file contains an access token so we fail early at login time
    // rather than at fetch time.
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Reading {}: {e}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("Parsing {}: {e}", path.display()))?;
    let has_token = parsed
        .get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|s| !s.is_empty());
    if !has_token {
        bail!(
            "No accessToken found in {}.\n\
             Run `claude login` again to refresh credentials.",
            path.display()
        );
    }

    println!(
        "{}",
        style(format!(
            "Found Claude Code credentials at {}",
            path.display()
        ))
        .green()
    );

    // No config fields needed — credentials are read from the well-known path
    // at fetch time. Return an empty object so the provider section is created
    // and pickable by the orchestrator.
    Ok(json!({}))
}
