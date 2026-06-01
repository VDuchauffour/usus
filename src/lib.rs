use anyhow::Result;

pub mod cli;
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

pub fn run(cli: cli::Cli) -> Result<()> {
    match cli.command {
        cli::Command::Login {
            workspace_id,
            server_id,
            function_id,
            sub_day,
        } => cmd_login(workspace_id, server_id, function_id, sub_day),

        cli::Command::SetSubDay => cmd_set_sub_day(),
        cli::Command::Report => cmd_report(),
    }
}
