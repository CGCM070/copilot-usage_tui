use chrono::{Datelike, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::models::UsageStats;
use crate::themes::ThemeColors;

use super::super::get_usage_color;

pub fn render(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors) {
    let block = Block::default()
        .title(" Overall Usage ")
        .borders(Borders::ALL)
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(inner);

    // Uso actual
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

    // Barra de progreso
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(get_usage_color(stats.percentage, colors))
                .bg(colors.bar_empty),
        )
        .percent(stats.percentage as u16)
        .label("");
    f.render_widget(gauge, layout[1]);

    // Progreso del mes
    let days_in_month = days_in_current_month();
    let current_day = Utc::now().day() as f64;
    let month_progress = (current_day / days_in_month as f64) * 100.0;

    let month_text = Paragraph::new(Line::from(vec![
        Span::styled("Month: ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{:.1}% elapsed", month_progress),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(month_text, layout[2]);
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
