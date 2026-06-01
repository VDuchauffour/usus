// Commands: login, set-sub-day

use anyhow::{Result, anyhow};
use std::fs;

use crate::helper::{die, prompt_sub_day, read_line, read_with_default};
use crate::providers::opencode_go::{Config, config_dir, config_path, load_config, save_config};
use crate::style::{BOLD, DIM, GREEN, RESET};

pub fn cmd_login(
    workspace_id: Option<String>,
    server_id: Option<String>,
    function_id: Option<i64>,
    sub_day: Option<u32>,
) -> Result<()> {
    // TODO read authCookie from stdin
    fs::create_dir_all(config_dir()?)?;

    println!("{BOLD}Login Setup{RESET}");
    println!();
    println!("Steps to get your auth cookie:");
    println!("  1. Log in to your OpenCode account in the browser");
    println!("  2. Open DevTools (F12) -> Application -> Cookies -> https://opencode.ai");
    println!("  3. Find the cookie named 'auth'");
    println!("  4. Copy its full value (starts with 'Fe26.2**')");
    println!();
    let auth_cookie = read_line("Paste your auth cookie: ")?;
    if auth_cookie.is_empty() {
        die("Auth cookie cannot be empty.");
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

    println!();
    println!("What day of the month does your billing cycle start?");
    println!("{DIM}(e.g., if you subscribed on 20-Apr, enter 20){RESET}");
    let sub_day = match sub_day {
        Some(v) => v,
        None => prompt_sub_day()?,
    };

    let cfg = Config {
        auth_cookie,
        workspace_id,
        server_id,
        function_id,
        sub_day,
    };
    save_config(&cfg)?;
    println!(
        "{GREEN}Configuration saved to {}{RESET}",
        config_path()?.display()
    );
    Ok(())
}

pub fn cmd_set_sub_day() -> Result<()> {
    let mut cfg = load_config()?;
    println!("Current billing cycle day: {BOLD}{}{RESET}", cfg.sub_day);
    println!();
    println!("What day of the month does your billing cycle start?");
    println!("{DIM}(e.g., if you subscribed on 20-Apr, enter 20){RESET}");
    let sub_day = prompt_sub_day()?;
    cfg.sub_day = sub_day;
    save_config(&cfg)?;
    println!("{GREEN}Billing cycle day updated to {sub_day}{RESET}");
    Ok(())
}
