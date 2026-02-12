use chrono::{Datelike, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::models::UsageStats;
use crate::themes::ThemeColors;

use super::super::get_usage_color;

pub fn render(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors) {
    let block = Block::default()
        .title(" Overall Usage: ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Requests Label
            Constraint::Length(1), // Requests Bar
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Month Label
            Constraint::Length(1), // Month Bar
        ])
        .margin(1)
        .split(inner);

    // 1. Requests Label
    let usage_text = Paragraph::new(Line::from(vec![
        Span::styled("Requests: ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{:.0}/{:.0}", stats.total_used, stats.total_limit),
            Style::default()
                .fg(get_usage_color(stats.percentage, colors))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({:.1}%)", stats.percentage),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(usage_text, layout[0]);

    // 2. Requests Bar (Custom Text-based, no background)
    let bar_width = layout[1].width as usize;
    let filled_len = ((stats.percentage / 100.0) * bar_width as f64) as usize;
    let filled_len = filled_len.min(bar_width);

    let filled_str = "█".repeat(filled_len);
    // Use "░" for the empty part to match the requested "gray box" style
    let empty_str = "░".repeat(bar_width.saturating_sub(filled_len));

    let bar_text = Paragraph::new(Line::from(vec![
        Span::styled(
            filled_str,
            Style::default().fg(get_usage_color(stats.percentage, colors)),
        ),
        Span::styled(empty_str, Style::default().fg(colors.muted)),
    ]));
    f.render_widget(bar_text, layout[1]);

    // 3. Spacer (Empty)

    // 4. Month Label
    let days_in_month = days_in_current_month();
    let current_day = Utc::now().day();
    let month_progress = (current_day as f64 / days_in_month as f64) * 100.0;

    let month_text = Paragraph::new(Line::from(vec![
        Span::styled("Month: ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{:.1}% elapsed", month_progress),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(month_text, layout[3]);

    // 5. Month Indicator (Style: ......|......)
    let month_bar_width = layout[4].width as usize;
    let pipe_pos =
        ((current_day as f64 / days_in_month as f64) * (month_bar_width as f64 - 1.0)) as usize;

    let mut month_spans = Vec::new();
    if pipe_pos > 0 {
        month_spans.push(Span::styled(
            ".".repeat(pipe_pos),
            Style::default().fg(colors.muted),
        ));
    }
    month_spans.push(Span::styled(
        "|",
        Style::default()
            .fg(colors.foreground)
            .add_modifier(Modifier::BOLD),
    ));
    if pipe_pos < month_bar_width - 1 {
        // Use darker dots for future, or just muted.
        month_spans.push(Span::styled(
            ".".repeat(month_bar_width.saturating_sub(pipe_pos + 1)),
            Style::default().fg(colors.bar_empty),
        ));
    }

    let month_bar = Paragraph::new(Line::from(month_spans));
    f.render_widget(month_bar, layout[4]);
}

fn days_in_current_month() -> u32 {
    let now = Utc::now();
    let next_month = if now.month() == 12 {
        now.with_month(1)
            .unwrap()
            .with_year(now.year() + 1)
            .unwrap()
    } else {
        now.with_month(now.month() + 1).unwrap()
    };
    (next_month - now).num_days() as u32 + now.day()
}
