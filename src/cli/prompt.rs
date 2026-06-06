use std::fmt::{Debug, Display};
use std::str::FromStr;

use anyhow::Result;
use dialoguer::Input;

pub fn prompt_string(value: Option<String>, prompt: &str, default: &str) -> Result<String> {
    match value {
        Some(v) => Ok(v),
        None => Ok(Input::new()
            .with_prompt(prompt)
            .default(default.to_string())
            .interact_text()?),
    }
}

pub fn prompt_number<T>(value: Option<T>, prompt: &str, default: T) -> Result<T>
where
    T: Clone + ToString + FromStr,
    <T as FromStr>::Err: Display + Debug,
{
    match value {
        Some(v) => Ok(v),
        None => Ok(Input::<T>::new()
            .with_prompt(prompt)
            .default(default)
            .interact_text()?),
    }
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
