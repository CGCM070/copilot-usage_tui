use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::models::UsageStats;
use crate::themes::ThemeColors;
use crate::ui::state::AppStateManager;

use super::super::get_usage_color;

pub fn render(
    f: &mut Frame,
    area: Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
    app: &AppStateManager,
) {
    let has_scroll = stats.models.len() > 8;
    let title = build_title(has_scroll, app.model_scroll_offset, stats.models.len(), 8);

    let block = Block::default()
        .title(title)
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

    if stats.models.is_empty() {
        render_empty_state(f, inner, colors);
        return;
    }

    render_table(f, inner, stats, colors, app);
}

fn build_title(has_scroll: bool, scroll: usize, total: usize, visible: usize) -> String {
    let mut title = " Per-Model Usage: ".to_string();
    if has_scroll {
        if scroll > 0 {
            title.insert_str(0, "↑ ");
        }
        if scroll + visible < total {
            title.push_str(" ↓");
        }
    }
    title
}

fn render_empty_state(f: &mut Frame, area: Rect, colors: &ThemeColors) {
    let no_data = Paragraph::new("No model usage data available")
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));
    f.render_widget(no_data, area);
}

fn render_table(
    f: &mut Frame,
    area: Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
    app: &AppStateManager,
) {
    let visible_count = (area.height as usize).saturating_sub(1);
    let scroll = app
        .model_scroll_offset
        .min(stats.models.len().saturating_sub(visible_count));

    let visible_models: Vec<_> = stats
        .models
        .iter()
        .skip(scroll)
        .take(visible_count)
        .collect();

    let rows: Vec<Row> = visible_models
        .iter()
        .map(|model| {
            let percentage_str = format!("{:.1}%", model.percentage);
            let usage_str = format!("{:.0}/{:.0}", model.used, model.limit);
            let display_name = model.name.strip_prefix("Auto: ").unwrap_or(&model.name);

            // Segmented progress bar (green -> orange -> red) - Vero style
            let bar_width: usize = 30; // Reduced because each char has a trailing space
            let filled = ((model.percentage / 100.0) * bar_width as f64) as usize;
            let filled = filled.min(bar_width);

            // Zone boundaries
            let zone_success_end = ((75.0 / 100.0) * bar_width as f64) as usize;
            let zone_warning_end = ((90.0 / 100.0) * bar_width as f64) as usize;

            // Build segmented bar spans (Vero-style: spaced squares)
            let mut bar_spans: Vec<Span> = Vec::new();

            // Segment 1: Success (0-75%)
            let success_chars = filled.min(zone_success_end);
            if success_chars > 0 {
                bar_spans.push(Span::styled(
                    "■ ".repeat(success_chars),
                    Style::default().fg(colors.success),
                ));
            }

            // Segment 2: Warning (75-90%)
            if filled > zone_success_end {
                let warning_chars = filled
                    .min(zone_warning_end)
                    .saturating_sub(zone_success_end);
                if warning_chars > 0 {
                    bar_spans.push(Span::styled(
                        "■ ".repeat(warning_chars),
                        Style::default().fg(Color::Rgb(255, 184, 108)),
                    ));
                }
            }

            // Segment 3: Error (90-100%)
            if filled > zone_warning_end {
                let error_chars = filled.saturating_sub(zone_warning_end);
                if error_chars > 0 {
                    bar_spans.push(Span::styled(
                        "■ ".repeat(error_chars),
                        Style::default().fg(Color::Rgb(255, 85, 85)),
                    ));
                }
            }

            // Empty part (Vero-style: spaced dots)
            let empty_len = bar_width.saturating_sub(filled);
            if empty_len > 0 {
                bar_spans.push(Span::styled(
                    "· ".repeat(empty_len),
                    Style::default().fg(colors.muted),
                ));
            }

            Row::new(vec![
                Cell::from(""), // Left Spacer
                Cell::from(Span::styled(
                    display_name.to_string(),
                    Style::default().fg(colors.foreground),
                )),
                Cell::from(Line::from(bar_spans)),
                Cell::from(Span::styled(
                    format!("{:^8}", percentage_str),
                    Style::default().fg(get_usage_color(model.percentage, colors)),
                )),
                Cell::from(Span::styled(
                    format!("{:^10}", usage_str),
                    Style::default().fg(colors.muted),
                )),
                Cell::from(""), // Right Spacer
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(1),      // Left Spacer (Margin)
            Constraint::Percentage(22), // Model (Reduced to 22%)
            Constraint::Percentage(56), // Progress (Increased to 56%)
            Constraint::Percentage(10), // Usage
            Constraint::Percentage(10), // Count
            Constraint::Length(1),      // Right Spacer (Margin)
        ],
    )
    .header(
        Row::new(vec!["", "Model", "Progress", " Usage", "  Count", ""]).style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .column_spacing(1);

    f.render_widget(table, area);
}
