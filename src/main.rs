use anyhow::Result;
use clap::Parser;
use console::style;
use usus::cli::command::{Cli, Command, login, report};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Login { provider } => login::run(provider),
        Command::Report { provider } => report::run(provider.as_deref()),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{} {e:#}", style("Error:").red());
        std::process::exit(1);
    }
}
