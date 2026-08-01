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

use crate::providers::{RollingUsageView, UsageWindowView};

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

/// Build the right-aligned "Renews <date>" label, or empty when the provider
/// exposes no renewal date.
fn renews_label(renews: &str) -> String {
    if renews.is_empty() {
        String::new()
    } else {
        format!("Renews {renews}")
    }
}

/// Spaces needed between `left` and `right` so they sit on opposite ends of the
/// panel width, always at least one space.
fn gap_between(left: &str, right: &str) -> usize {
    PANEL_WIDTH
        .saturating_sub(left.chars().count() + right.chars().count())
        .max(1)
}

/// Header row: provider name on the left (bold, cyan), optional renew date on
/// the right (dim).
fn header_line(view: &RollingUsageView) -> Line<'static> {
    let title = view.title.as_str();
    let renews = renews_label(&view.renews);
    let gap = gap_between(title, &renews);
    Line::from(vec![
        Span::styled(
            format!("  {title}"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gap)),
        Span::styled(renews, Style::default().dim()),
    ])
}

/// Separator row: a full-width dim horizontal rule under the header.
fn separator_line() -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  {}", "─".repeat(PANEL_WIDTH)),
        Style::default().dim(),
    )])
}

/// One body entry's info row: window label on the left (bold), reset text on
/// the right (dim).
fn info_line(window: &UsageWindowView) -> Line<'static> {
    let label = window.label;
    let reset = format!("resets in {}", format_reset(window.reset_in_sec));
    let gap = gap_between(label, &reset);
    Line::from(vec![
        Span::styled(
            format!("  {label}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gap)),
        Span::styled(reset, Style::default().dim()),
    ])
}

/// One body entry's progress-bar row: filled glyphs colored by usage, empty
/// glyphs dim, and a right-aligned percentage label.
fn bar_line(window: &UsageWindowView) -> Line<'static> {
    let pct = window.percent.clamp(0.0, 100.0);
    let bar_w = PANEL_WIDTH - 7;
    let filled = (((pct / 100.0) * bar_w as f64).round() as usize).min(bar_w);
    let empty = bar_w - filled;
    let pct_label = pad_left(&format!("{pct:.1}%"), 6);
    Line::from(vec![
        Span::raw("  "),
        Span::styled("█".repeat(filled), Style::default().fg(bar_color(pct))),
        Span::styled("░".repeat(empty), Style::default().dim()),
        Span::raw(format!(" {pct_label}")),
    ])
}

pub fn render_rolling(view: &RollingUsageView) -> Result<()> {
    let mut lines: Vec<Line> = Vec::with_capacity(view.windows.len() * 2 + 4);
    lines.push(Line::raw(""));
    lines.push(header_line(view));
    lines.push(separator_line());
    for window in &view.windows {
        lines.push(info_line(window));
        lines.push(bar_line(window));
    }
    lines.push(Line::raw(""));
    draw_lines(lines)
}
