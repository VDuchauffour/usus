// Interactive prompt primitives shared by the CLI and provider login flows.

use std::fmt::{Debug, Display};
use std::str::FromStr;

use anyhow::Result;
use console::style;
use dialoguer::Input;

pub fn initial_login_message(header: &str, description: &str) {
    println!("{}\n", style(header).bold());
    println!(r#"{}"#, description);
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
