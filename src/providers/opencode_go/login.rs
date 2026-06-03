use anyhow::{Result, anyhow, bail};

use crate::config::{config_path, load_or_default, save};
use crate::helper::{prompt_sub_day, read_line, read_with_default};
use crate::providers::opencode_go::{Config, ID};
use crate::style::{BOLD, DIM, GREEN, RESET};

pub fn cmd_login(
    workspace_id: Option<String>,
    server_id: Option<String>,
    function_id: Option<i64>,
    sub_day: Option<u32>,
) -> Result<()> {
    println!("{BOLD}OpenCode GO setup{RESET}");
    println!();
    println!("Steps to get your auth cookie:");
    println!("  1. Log in to your OpenCode account in the browser");
    println!("  2. Open DevTools (F12) -> Application -> Cookies -> https://opencode.ai");
    println!("  3. Copy the 'auth' cookie value (starts with 'Fe26.2**')");
    println!();
    let auth_cookie = read_line("Paste your auth cookie: ")?;
    if auth_cookie.is_empty() {
        bail!("Auth cookie cannot be empty.");
    }
    if !auth_cookie.starts_with("Fe26.2**") {
        bail!("This is not a valid auth cookie.")
    }
    println!("{GREEN}Auth cookie saved.{RESET}");
    println!();

    let workspace_id = match workspace_id {
        Some(v) => v,
        None => read_with_default(
            "Workspace ID [wrk_01KDSXX2YK0SSF30AKBTQGWQM9]: ",
            "wrk_01KDSXX2YK0SSF30AKBTQGWQM9",
        )?,
    };
    let server_id = match server_id {
        Some(v) => v,
        None => read_with_default(
            "Server ID [15702f3a12ff8bff357f8c2aa154a17e65b746d5f6b96adc9002c86ee0c15205]: ",
            "15702f3a12ff8bff357f8c2aa154a17e65b746d5f6b96adc9002c86ee0c15205",
        )?,
    };
    let function_id = match function_id {
        Some(v) => v,
        None => {
            let s = read_with_default("Function ID [31]: ", "31")?;
            s.parse()
                .map_err(|_| anyhow!("Function ID must be a number"))?
        }
    };

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
                println!("{DIM}(e.g., if you subscribed on 20-Apr, enter 20){RESET}");
                prompt_sub_day()?
            }
        };
    } else if let Some(v) = sub_day {
        top.sub_day = v;
    }

    save(&top)?;
    println!(
        "{GREEN}Provider '{ID}' saved to {}{RESET}",
        config_path()?.display()
    );
    Ok(())
}
