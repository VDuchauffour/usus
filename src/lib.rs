use anyhow::Result;
use std::env;

pub mod commands;
pub mod helper;
pub mod http;
pub mod parser;
pub mod providers;
pub mod render;
pub mod report;
pub mod style;

use commands::{cmd_login, cmd_set_sub_day};
use report::cmd_report;
use style::{BOLD, RESET};

fn show_help() {
    let prog = env::args().next().unwrap_or_else(|| "usage".into());
    println!("{BOLD}OpenCode Go Usage CLI{RESET}");
    println!();
    println!("Usage: {prog} <command>");
    println!();
    println!("Commands:");
    println!("  login        Save your auth cookie and workspace config");
    println!("  set-sub-day  Update your billing cycle start day");
    println!("  report       Fetch and display current usage");
}

pub fn run(args: &[String]) -> Result<()> {
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    match cmd {
        "login" => cmd_login(),
        "set-sub-day" => cmd_set_sub_day(),
        "report" => cmd_report(),
        "help" | "--help" | "-h" => {
            show_help();
            Ok(())
        }
        _ => {
            show_help();
            std::process::exit(1);
        }
    }
}
