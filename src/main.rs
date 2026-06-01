use clap::Parser;
pub mod style;
use style::{RED, RESET};

fn main() {
    let cli = usus::cli::Cli::parse();
    if let Err(e) = usus::run(cli) {
        eprintln!("{RED}Error:{RESET} {e:#}");
        std::process::exit(1);
    }
}
