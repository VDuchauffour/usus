use anyhow::{Result, bail};
use console::style;
use dialoguer::Input;

use crate::cli::prompt::{prompt_number, prompt_string};
use crate::config::{config_path, load_or_default, save};
use crate::helper::{initial_login_message, prompt_sub_day};
use crate::providers::opencode_go::{Config, ID};

pub fn cmd_login(
    workspace_id: Option<String>,
    server_id: Option<String>,
    function_id: Option<i64>,
    sub_day: Option<u32>,
) -> Result<()> {
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

    let workspace_id: String = prompt_string(
        workspace_id,
        "Workspace ID",
        "wrk_01KDSXX2YK0SSF30AKBTQGWQM9",
    )?;
    let server_id: String = prompt_string(
        server_id,
        "Server ID",
        "15702f3a12ff8bff357f8c2aa154a17e65b746d5f6b96adc9002c86ee0c15205",
    )?;
    let function_id: i64 = prompt_number(function_id, "Function ID", 31)?;

    let cfg = Config {
        auth_cookie,
        workspace_id,
        server_id,
        function_id,
    };
    let provider_blob = serde_json::to_value(&cfg)?;

    let mut top = load_or_default()?;
    top.providers.insert(ID.to_string(), provider_blob);
    if top.default.is_empty() {
        top.default = ID.to_string();
    }
    if top.sub_day == 0 {
        top.sub_day = match sub_day {
            Some(v) => v,
            None => {
                println!();
                println!("What day of the month does your billing cycle start?");
                println!(
                    "{}",
                    style("(e.g., if you subscribed on 20-Apr, enter 20)").dim()
                );
                prompt_sub_day()?
            }
        };
    } else if let Some(v) = sub_day {
        top.sub_day = v;
    }

    save(&top)?;
    println!(
        "{}",
        style(format!(
            "Provider '{ID}' saved to {}",
            config_path()?.display()
        ))
        .green()
    );
    Ok(())
}
