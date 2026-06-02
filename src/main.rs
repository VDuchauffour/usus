use anyhow::Result;
use clap::Parser;

use usus::cli::{Cli, Command, LoginProvider};
use usus::providers::{anthropic, opencode_go};
use usus::report;
use usus::style::{RED, RESET};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Login { provider } => match provider {
            LoginProvider::OpencodeGo {
                workspace_id,
                server_id,
                function_id,
                sub_day,
            } => opencode_go::login::cmd_login(workspace_id, server_id, function_id, sub_day),
            LoginProvider::Anthropic {
                admin_key,
                allowance,
                sub_day,
            } => anthropic::login::cmd_login(admin_key, allowance, sub_day),
        },
        Command::Report { provider } => report::cmd_report(provider.as_deref()),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{RED}Error:{RESET} {e:#}");
        std::process::exit(1);
    }
}
