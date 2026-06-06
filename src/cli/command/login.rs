use anyhow::{Result, bail};
use console::style;
use dialoguer::Input;
use serde_json::Value;

use crate::cli::command::LoginProvider;
use crate::cli::prompt::{prompt_number, prompt_string, prompt_sub_day};
use crate::config::{config_path, load_or_default, save};
use crate::providers::{anthropic, opencode_go};

pub fn run(provider: LoginProvider) -> Result<()> {
    match provider {
        LoginProvider::OpencodeGo {
            workspace_id,
            server_id,
            function_id,
            sub_day,
        } => opencode_go(workspace_id, server_id, function_id, sub_day),
        LoginProvider::Anthropic {
            admin_key,
            allowance,
            sub_day,
        } => anthropic(admin_key, allowance, sub_day),
    }
}

pub fn initial_login_message(header: &str, description: &str) {
    println!("{}\n", style(header).bold());
    println!(r#"{}"#, description);
}

fn opencode_go(
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

    let cfg = opencode_go::Config {
        auth_cookie,
        workspace_id,
        server_id,
        function_id,
    };
    persist_provider(opencode_go::ID, serde_json::to_value(&cfg)?, sub_day)
}

fn anthropic(
    admin_key: Option<String>,
    allowance: Option<f64>,
    sub_day: Option<u32>,
) -> Result<()> {
    initial_login_message(
        "Anthropic API setup",
        "You need an Admin API key:
1. Open https://console.anthropic.com/settings/admin-keys
2. Create an Admin key (starts with 'sk-ant-admin01-...')",
    );

    let admin_key: String = match admin_key {
        Some(v) => v,
        None => Input::new()
            .with_prompt("Admin API key")
            .interact_text()
            .unwrap(),
    };
    if admin_key.is_empty() {
        bail!("Admin key cannot be empty.");
    }
    if !admin_key.starts_with("sk-ant-admin") {
        bail!("This is not a valid admin key.");
    }

    let allowance: f64 = prompt_number(
        allowance,
        "Monthly allowance in USD",
        anthropic::DEFAULT_ALLOWANCE,
    )?;

    let cfg = anthropic::Config {
        admin_key,
        allowance,
    };
    persist_provider(anthropic::ID, serde_json::to_value(&cfg)?, sub_day)
}

/// Persist a freshly configured provider blob, seeding `default`/`sub_day`
/// on first setup and honouring an explicit `--sub-day` override.
fn persist_provider(id: &str, provider_blob: Value, sub_day: Option<u32>) -> Result<()> {
    let mut top = load_or_default()?;
    top.providers.insert(id.to_string(), provider_blob);
    if top.default.is_empty() {
        top.default = id.to_string();
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
            "Provider '{id}' saved to {}",
            config_path()?.display()
        ))
        .green()
    );
    Ok(())
}
