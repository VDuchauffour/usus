use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "usus", version, about = "Your best partner for AI harnesses")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Save your auth cookie and workspace config
    Login {
        #[arg(long, short)]
        workspace_id: Option<String>,

        #[arg(long, short)]
        server_id: Option<String>,

        #[arg(long, short)]
        function_id: Option<i64>,

        #[arg(long, value_parser = clap::value_parser!(u32).range(1..=31))]
        sub_day: Option<u32>,
    },
    /// Fetch and display current usage
    Report,
}
