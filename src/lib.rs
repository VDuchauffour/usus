use anyhow::Result;

pub mod cli;
pub mod helper;
pub mod http;
pub mod login;
pub mod parser;
pub mod providers;
pub mod render;
pub mod report;
pub mod style;

use login::cmd_login;
use report::cmd_report;

pub fn run(cli: cli::Cli) -> Result<()> {
    match cli.command {
        cli::Command::Login {
            workspace_id,
            server_id,
            function_id,
            sub_day,
        } => cmd_login(workspace_id, server_id, function_id, sub_day),

        cli::Command::Report => cmd_report(),
    }
}
