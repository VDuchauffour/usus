use clap::{Parser, Subcommand};

pub mod login;
pub mod report;
use clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
};

pub fn cargo_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}

#[derive(Parser)]
#[command(name = "usus", version, about = "Your best partner for AI harnesses",styles = cargo_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Top-level provider selector. The action (report/login) is nested and
/// optional, defaulting to `report` when omitted.
#[derive(Subcommand)]
pub enum Command {
    /// Use the OpenCode GO provider
    #[command(name = "opencode")]
    OpencodeGo {
        #[command(subcommand)]
        action: Option<OpencodeGoAction>,
    },
    /// Use the Anthropic Admin API provider
    Anthropic {
        #[command(subcommand)]
        action: Option<AnthropicAction>,
    },
}

#[derive(Subcommand)]
pub enum OpencodeGoAction {
    /// Fetch and display current usage (default when no action is given)
    Report,
    /// Configure this provider
    Login,
}

#[derive(Subcommand)]
pub enum AnthropicAction {
    /// Fetch and display current usage (default when no action is given)
    Report,
    /// Configure this provider
    Login,
}
