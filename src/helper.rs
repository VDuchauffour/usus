use std::io::{self, BufRead, Write};

use anyhow::{Result, anyhow, bail};
use console::style;

pub fn read_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().lock().read_line(&mut s)?;
    // Match `read -r`: strip trailing newline only.
    Ok(s.trim_end_matches(['\n', '\r']).to_string())
}

pub fn read_with_default(prompt: &str, default: &str) -> Result<String> {
    let s = read_line(prompt)?;
    Ok(if s.is_empty() { default.to_string() } else { s })
}

pub fn prompt_sub_day() -> Result<u32> {
    let s = read_with_default("Billing cycle day [1]: ", "1")?;
    let n: u32 = s
        .parse()
        .map_err(|_| anyhow!("Invalid day. Must be a number between 1 and 31."))?;
    if !(1..=31).contains(&n) {
        bail!("Invalid day. Must be a number between 1 and 31.");
    }
    Ok(n)
}

pub fn initial_login_message(header: &str, description: &str) {
    println!("{}\n", style(header).bold());
    println!(r#"{}"#, description);
}
