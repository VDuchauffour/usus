use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use console::style;
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};

pub fn initial_login_message(header: &str, description: &str) {
    println!("{}\n", style(header).bold());
    println!(r#"{}"#, description);
}

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

pub fn prompt_string(prompt: &str, default: &str) -> Result<String> {
    Ok(Input::new()
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_text()?)
}

pub fn prompt_number<T>(prompt: &str, default: T) -> Result<T>
where
    T: Clone + ToString + FromStr,
    <T as FromStr>::Err: Display + Debug,
{
    Ok(Input::<T>::new()
        .with_prompt(prompt)
        .default(default)
        .interact_text()?)
}

pub fn prompt_sub_day() -> Result<u32> {
    Ok(Input::<u32>::new()
        .with_prompt("Billing cycle day")
        .default(1)
        .validate_with(|n: &u32| -> Result<(), &str> {
            if (1..=31).contains(n) {
                Ok(())
            } else {
                Err("Must be a number between 1 and 31.")
            }
        })
        .interact_text()?)
}
