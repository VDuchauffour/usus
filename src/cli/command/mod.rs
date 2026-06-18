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
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Configure a provider (OpenCode GO, Anthropic, ...)
    Login {
        #[command(subcommand)]
        provider: LoginProvider,
    },
    /// Fetch and display current usage
    Report {
        /// Provider id to query; fallback to the configured default
        #[arg(long, short)]
        provider: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum LoginProvider {
    /// Configure the OpenCode GO provider
    #[command(name = "opencode-go")]
    OpencodeGo {
        #[arg(long, short)]
        workspace_id: Option<String>,

        #[arg(long, short)]
        server_id: Option<String>,

        #[arg(long, short)]
        function_id: Option<i64>,

        #[arg(long, value_parser = clap::value_parser!(u32).range(1..=31))]
        sub_day: Option<u32>,
    },
    /// Configure the Anthropic Admin API provider
    Anthropic {
        #[arg(long)]
        admin_key: Option<String>,

        #[arg(long)]
        allowance: Option<f64>,

        #[arg(long, value_parser = clap::value_parser!(u32).range(1..=31))]
        sub_day: Option<u32>,
    },
}
