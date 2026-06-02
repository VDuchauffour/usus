use anyhow::Result;
use clap::Parser;

use usus::style::{RED, RESET};
use usus::{cli, login, report};

pub fn run(cli: cli::Cli) -> Result<()> {
    match cli.command {
        cli::Command::Login {
            workspace_id,
            server_id,
            function_id,
            sub_day,
        } => login::cmd_login(workspace_id, server_id, function_id, sub_day),

        cli::Command::Report => report::cmd_report(),
    }
}
fn main() {
    let cli = cli::Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{RED}Error:{RESET} {e:#}");
        std::process::exit(1);
    }
}
