use chrono::{Datelike, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::models::UsageStats;
use crate::themes::ThemeColors;

use super::super::get_usage_color;

/// Get color for a month progress dot based on its position
/// Colors indicate proximity to reset date (end of month)
fn get_month_dot_color(position_percent: f64, colors: &ThemeColors) -> Color {
    if position_percent >= 95.0 {
        colors.success // Green - Reset imminent! New requests coming!
    } else if position_percent >= 85.0 {
        colors.error // Red - Very close to reset
    } else if position_percent >= 70.0 {
        colors.warning // Yellow - Getting closer
    } else {
        colors.muted // Gray - Normal passed days
    }
}

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
            Constraint::Length(1), // Top Padding
            Constraint::Length(1), // Requests Label
            Constraint::Length(1), // Requests Bar
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Month Label
            Constraint::Length(1), // Month Bar
        ])
        .horizontal_margin(1)
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
    f.render_widget(usage_text, layout[1]);

    // 2. Requests Bar - Segmented gradient (green -> orange -> red)
    // Zone thresholds: 0-75% = success, 75-90% = warning, 90-100% = error
    let bar_width = layout[2].width as usize;
    let filled_len = ((stats.percentage / 100.0) * bar_width as f64) as usize;
    let filled_len = filled_len.min(bar_width);

    // Calculate zone boundaries in characters
    let zone_success_end = ((75.0 / 100.0) * bar_width as f64) as usize; // 0-75%
    let zone_warning_end = ((90.0 / 100.0) * bar_width as f64) as usize; // 75-90%
                                                                         // 90-100% is the rest

    // Build segmented bar
    let mut bar_spans: Vec<Span> = Vec::new();

    // Segment 1: Success zone (0-75%)
    let success_chars = filled_len.min(zone_success_end);
    if success_chars > 0 {
        bar_spans.push(Span::styled(
            "█".repeat(success_chars),
            Style::default().fg(colors.success),
        ));
    }

    // Segment 2: Warning zone (75-90%)
    if filled_len > zone_success_end {
        let warning_chars = filled_len
            .min(zone_warning_end)
            .saturating_sub(zone_success_end);
        if warning_chars > 0 {
            bar_spans.push(Span::styled(
                "█".repeat(warning_chars),
                Style::default().fg(Color::Rgb(255, 184, 108)), // Constant warning orange
            ));
        }
    }

    // Segment 3: Error zone (90-100%)
    if filled_len > zone_warning_end {
        let error_chars = filled_len.saturating_sub(zone_warning_end);
        if error_chars > 0 {
            bar_spans.push(Span::styled(
                "█".repeat(error_chars),
                Style::default().fg(Color::Rgb(255, 85, 85)), // Constant error red
            ));
        }
    }

    // Empty part
    let empty_len = bar_width.saturating_sub(filled_len);
    if empty_len > 0 {
        bar_spans.push(Span::styled(
            "░".repeat(empty_len),
            Style::default().fg(colors.muted),
        ));
    }

    let bar_text = Paragraph::new(Line::from(bar_spans));
    f.render_widget(bar_text, layout[2]);

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
    f.render_widget(month_text, layout[4]);

    // 5. Month Indicator with gradient colors
    // Colors show proximity to reset: muted -> warning -> error -> success (reset!)
    let month_bar_width = layout[5].width as usize;
    let pipe_pos =
        ((current_day as f64 / days_in_month as f64) * (month_bar_width as f64 - 1.0)) as usize;

    let mut month_spans: Vec<Span> = Vec::new();

    // Passed days - colored based on position in month
    for i in 0..pipe_pos {
        let position_percent = (i as f64 / (month_bar_width as f64 - 1.0)) * 100.0;
        let dot_color = get_month_dot_color(position_percent, colors);
        month_spans.push(Span::styled(".", Style::default().fg(dot_color)));
    }

    // Current position indicator - always success (green) to indicate "we're here, reset coming"
    month_spans.push(Span::styled(
        "|",
        Style::default()
            .fg(colors.success)
            .add_modifier(Modifier::BOLD),
    ));

    // Future days - use bar_empty (dark) color
    if pipe_pos < month_bar_width - 1 {
        month_spans.push(Span::styled(
            ".".repeat(month_bar_width.saturating_sub(pipe_pos + 1)),
            Style::default().fg(colors.bar_empty),
        ));
    }

    let month_bar = Paragraph::new(Line::from(month_spans));
    f.render_widget(month_bar, layout[5]);
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
