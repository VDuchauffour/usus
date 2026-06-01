use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "usus", version, about = "Your best partner for AI harnesses")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Login,
    SetSubDay,
    Report,
}
