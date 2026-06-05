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

pub fn prompt_i64(value: Option<i64>, prompt: &str, default: i64) -> Result<i64> {
    match value {
        Some(v) => Ok(v),
        None => Ok(Input::new()
            .with_prompt(prompt)
            .default(default)
            .interact_text()?),
    }
}
