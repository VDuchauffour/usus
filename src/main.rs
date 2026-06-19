use anyhow::Result;
use clap::Parser;
use console::style;
use usus::cli::command::{AnthropicAction, Cli, Command, OpencodeGoAction, login, report};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Command::OpencodeGo { action }) => match action {
            Some(OpencodeGoAction::Login) => login::run("opencode-go"),
            Some(OpencodeGoAction::Report) | None => report::run(Some("opencode-go")),
        },
        Some(Command::Anthropic { action }) => match action {
            Some(AnthropicAction::Login) => login::run("anthropic"),
            Some(AnthropicAction::Report) | None => report::run(Some("anthropic")),
        },
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
