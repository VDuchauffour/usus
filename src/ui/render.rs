// Render via ratatui (Viewport::Inline - one-shot, no raw mode, no alt-screen).

use std::io;

use anyhow::Result;
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::providers::ReportView;

const PANEL_WIDTH: usize = 52;

/// Mimic the JS regex `/^[^\s]+@[^\s]+\s+-\s+/` stripping a `someone@host - ` prefix.
fn strip_email_prefix(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() && !chars[i].is_whitespace() {
        i += 1;
    }
    let token: String = chars[..i].iter().collect();
    if !token.contains('@') {
        return s.to_string();
    }
    let mut j = i;
    while j < chars.len() && chars[j].is_whitespace() {
        j += 1;
    }
    if j >= chars.len() || chars[j] != '-' {
        return s.to_string();
    }
    j += 1;
    while j < chars.len() && chars[j].is_whitespace() {
        j += 1;
    }
    chars[j..].iter().collect()
}

fn truncate_or_pad(name: &str, width: usize) -> String {
    let count = name.chars().count();
    if count > width {
        let truncated: String = name.chars().take(width - 3).collect();
        format!("{truncated}...")
    } else {
        let mut out = String::with_capacity(name.len() + (width - count));
        out.push_str(name);
        out.extend(std::iter::repeat_n(' ', width - count));
        out
    }
}

fn pad_left(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width {
        s.to_string()
    } else {
        let mut out = String::with_capacity(s.len() + (width - count));
        out.extend(std::iter::repeat_n(' ', width - count));
        out.push_str(s);
        out
    }
}

pub fn render(view: &ReportView) -> Result<()> {
    let currency = view.currency;
    let allowance = view.allowance;
    let total_cost = view.total_cost;
    let remaining = allowance - total_cost;
    let pct_used = if allowance > 0.0 {
        (total_cost / allowance) * 100.0
    } else {
        0.0
    };

    let bar_w = PANEL_WIDTH - 7;
    let filled = ((pct_used / 100.0) * bar_w as f64).round() as usize;
    let filled = filled.min(bar_w);
    let empty = bar_w - filled;
    let bar_color = if pct_used > 80.0 {
        Color::Red
    } else if pct_used > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    };
    let pct_label = pad_left(&format!("{pct_used:.1}%"), 6);

    let hr: String = "─".repeat(PANEL_WIDTH);
    let renew_str = if view.period_end.is_empty() {
        String::new()
    } else {
        format!("Renews {}", view.period_end)
    };
    let title = view.title.as_str();
    let h_pad_len = PANEL_WIDTH
        .saturating_sub(title.chars().count() + renew_str.chars().count())
        .max(1);
    let h_pad = " ".repeat(h_pad_len);

    let used_str = format!("{currency}{total_cost:.2} / {currency}{allowance:.2}");
    let rem_str = format!("{currency}{remaining:.2} remaining");
    let c_pad_len = PANEL_WIDTH
        .saturating_sub(used_str.chars().count() + rem_str.chars().count())
        .max(1);
    let c_pad = " ".repeat(c_pad_len);

    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                format!("  {title}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(h_pad.to_string()),
            Span::styled(renew_str, Style::default().dim()),
        ]),
        Line::from(vec![Span::styled(
            format!("  {hr}"),
            Style::default().dim(),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
            Span::styled("░".repeat(empty), Style::default().dim()),
            Span::raw(format!(" {pct_label}")),
        ]),
        Line::from(vec![
            Span::raw("  ".to_string()),
            Span::styled(used_str, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(c_pad),
            Span::styled(rem_str, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![Span::styled(
            format!("  {hr}"),
            Style::default().dim(),
        )]),
    ];

    for (name, cost) in &view.rows {
        let stripped = strip_email_prefix(name);
        let display = truncate_or_pad(&stripped, 32);
        let cost_str = pad_left(&format!("{currency}{cost:.4}"), 10);
        let pct = if allowance > 0.0 {
            pad_left(&format!("{:.1}%", (cost / allowance) * 100.0), 6)
        } else {
            pad_left("--", 6)
        };
        lines.push(Line::from(vec![
            Span::raw(format!("  {display}  ")),
            Span::styled(cost_str, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(pct, Style::default().dim()),
        ]));
    }

    lines.push(Line::raw(""));

    let backend = CrosstermBackend::new(io::stdout());
    let height = lines.len() as u16;
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;
    terminal.draw(|frame| {
        let paragraph = Paragraph::new(lines.clone());
        frame.render_widget(paragraph, frame.area());
    })?;

    Ok(())
}
