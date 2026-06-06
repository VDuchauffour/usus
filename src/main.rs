use anyhow::Result;
use clap::Parser;
use usus::{
    cli::command::{Cli, Command, login, report},
    style::{RED, RESET},
};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Login { provider } => login::run(provider),
        Command::Report { provider } => report::run(provider.as_deref()),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{RED}Error:{RESET} {e:#}");
        std::process::exit(1);
    }
}
