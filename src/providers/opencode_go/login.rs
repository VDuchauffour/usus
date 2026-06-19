use anyhow::{Result, bail};
use dialoguer::Input;
use serde_json::Value;

use crate::providers::opencode_go::Config;
use crate::ui::prompt::{initial_login_message, prompt_number, prompt_string};

pub fn run() -> Result<Value> {
    initial_login_message(
        "OpenCode GO setup",
        "Steps to get your auth cookie:
1. Log in to your OpenCode account in the browser
2. Open DevTools (F12) -> Application -> Cookies -> https://opencode.ai
3. Copy the 'auth' cookie value (starts with 'Fe26.2**')",
    );

    let auth_cookie: String = Input::new()
        .with_prompt("Auth cookie")
        .interact_text()
        .unwrap();
    if auth_cookie.is_empty() {
        bail!("Auth cookie cannot be empty.");
    }
    if !auth_cookie.starts_with("Fe26.2**") {
        bail!("This is not a valid auth cookie.");
    }

    let workspace_id: String = prompt_string("Workspace ID", "wrk_01KDSXX2YK0SSF30AKBTQGWQM9")?;
    let server_id: String = prompt_string(
        "Server ID",
        "15702f3a12ff8bff357f8c2aa154a17e65b746d5f6b96adc9002c86ee0c15205",
    )?;
    let function_id: i64 = prompt_number("Function ID", 31)?;

    let cfg = Config {
        auth_cookie,
        workspace_id,
        server_id,
        function_id,
    };
    Ok(serde_json::to_value(&cfg)?)
}
