use anyhow::Result;
use clap::Parser;
use console::style;
use usus::cli::command::{
    AnthropicAction, Cli, Command, OpencodeGoAction, completion, login, report,
};
use usus::providers::ProviderId;

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Command::OpencodeGo { action }) => match action {
            Some(OpencodeGoAction::Login) => login::run(ProviderId::OpencodeGo),
            Some(OpencodeGoAction::Report) | None => report::run(Some(ProviderId::OpencodeGo)),
        },
        Some(Command::Anthropic { action }) => match action {
            Some(AnthropicAction::Login) => login::run(ProviderId::Anthropic),
            Some(AnthropicAction::Report) | None => report::run(Some(ProviderId::Anthropic)),
        },
        Some(Command::Completion { shell }) => completion::run(shell),
        None => report::run(None),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{} {e:#}", style("Error:").red());
        std::process::exit(1);
    }
}
