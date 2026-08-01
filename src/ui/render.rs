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

use crate::providers::RollingUsageView;

const PANEL_WIDTH: usize = 52;

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

fn draw_lines(lines: Vec<Line>) -> Result<()> {
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
    drop(terminal); // restore cursor before the trailing newline
    println!(); // ensure stdout ends with \n so the shell doesn't print its % marker

    Ok(())
}

fn bar_color(pct: f64) -> Color {
    if pct > 80.0 {
        Color::Red
    } else if pct > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn format_reset(secs: i64) -> String {
    if secs <= 0 {
        return "now".to_string();
    }
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

pub fn render_rolling(view: &RollingUsageView) -> Result<()> {
    let hr: String = "─".repeat(PANEL_WIDTH);
    let renews_str = if view.renews.is_empty() {
        String::new()
    } else {
        format!("Renews {}", view.renews)
    };
    let title = view.title.as_str();
    let h_pad_len = PANEL_WIDTH
        .saturating_sub(title.chars().count() + renews_str.chars().count())
        .max(1);
    let h_pad = " ".repeat(h_pad_len);
    let mut lines: Vec<Line> = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                format!("  {title}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(h_pad),
            Span::styled(renews_str, Style::default().dim()),
        ]),
        Line::from(vec![Span::styled(
            format!("  {hr}"),
            Style::default().dim(),
        )]),
    ];

    let bar_w = PANEL_WIDTH - 7;
    for window in &view.windows {
        let pct = window.percent.clamp(0.0, 100.0);
        let filled = (((pct / 100.0) * bar_w as f64).round() as usize).min(bar_w);
        let empty = bar_w - filled;
        let pct_label = pad_left(&format!("{pct:.1}%"), 6);
        let reset = format!("resets in {}", format_reset(window.reset_in_sec));

        let head_pad = PANEL_WIDTH
            .saturating_sub(window.label.chars().count() + reset.chars().count())
            .max(1);

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {}", window.label),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(head_pad)),
            Span::styled(reset, Style::default().dim()),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("█".repeat(filled), Style::default().fg(bar_color(pct))),
            Span::styled("░".repeat(empty), Style::default().dim()),
            Span::raw(format!(" {pct_label}")),
        ]));
    }

    lines.push(Line::raw(""));

    draw_lines(lines)
}
