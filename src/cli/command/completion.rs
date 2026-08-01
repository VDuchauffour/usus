// Completion command — emit a shell completion script.

use std::io;

use anyhow::{Result, anyhow};
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use super::Cli;

pub fn run(shell: Option<Shell>) -> Result<()> {
    let shell = shell.or_else(Shell::from_env).ok_or_else(|| {
        anyhow!(
            "could not detect current shell from $SHELL; specify one explicitly, e.g. \
             `usus completion bash` (supported: bash, zsh, fish, elvish, powershell)"
        )
    })?;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_bin_name().unwrap_or(cmd.get_name()).to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}
