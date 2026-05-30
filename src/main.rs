use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(e) = usus::run(&args) {
        eprintln!("\x1b[0;31mError:\x1b[0m {e:#}");
        process::exit(1);
    }
}
