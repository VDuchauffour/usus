use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub fn get_spinner(message: &'static str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
    );
    spinner.set_message(message);
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}
