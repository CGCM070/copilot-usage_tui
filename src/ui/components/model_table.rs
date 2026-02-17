use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::models::UsageStats;
use crate::themes::ThemeColors;
use crate::ui::state::AppStateManager;
use crate::ui::styles::{
    calculate_filled_cells, calculate_responsive_bar_width, calculate_zone_boundaries,
    error_style_bold, format_count, format_percentage, header_style, muted_style,
    success_style_bold, usage_style, warning_style_bold, with_horizontal_margin, BAR_EMPTY,
    BAR_FILLED,
};

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
        .title_style(header_style(colors));

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
        .style(muted_style(colors));
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

    // Check if we should use compact mode (hide progress bar when terminal is small)
    let is_compact = area.width < 60;

    if is_compact {
        render_compact_table(f, area, &visible_models, colors);
    } else {
        render_full_table(f, area, &visible_models, colors);
    }
}

fn render_compact_table(
    f: &mut Frame,
    area: Rect,
    visible_models: &[&crate::models::ModelUsage],
    colors: &ThemeColors,
) {
    let rows: Vec<Row> = visible_models
        .iter()
        .map(|model| {
            let percentage_str = format_percentage(model.percentage);
            let usage_str = format_count(model.used);
            let display_name = model.name.strip_prefix("Auto: ").unwrap_or(&model.name);

            Row::new(vec![
                Cell::from(Span::styled(
                    display_name.to_string(),
                    Style::default().fg(colors.foreground),
                )),
                Cell::from(Span::styled(
                    format!("{:^8}", percentage_str),
                    usage_style(model.percentage, colors),
                )),
                Cell::from(Span::styled(usage_str, muted_style(colors))),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(65), // Model (wider when no progress bar)
            Constraint::Percentage(20), // Usage
            Constraint::Length(7),      // Count
        ],
    )
    .header(Row::new(vec!["Model", "Usage", "Count"]).style(header_style(colors)))
    .column_spacing(2);

    let layout = with_horizontal_margin(area);
    f.render_widget(table, layout[0]);
}

fn render_full_table(
    f: &mut Frame,
    area: Rect,
    visible_models: &[&crate::models::ModelUsage],
    colors: &ThemeColors,
) {
    // Calculate responsive bar width
    let progress_col_width = ((area.width as f32 * 0.56) as u16).saturating_sub(4);
    let bar_width = calculate_responsive_bar_width(progress_col_width);

    let rows: Vec<Row> = visible_models
        .iter()
        .map(|model| {
            let percentage_str = format_percentage(model.percentage);
            let usage_str = format_count(model.used);
            let display_name = model.name.strip_prefix("Auto: ").unwrap_or(&model.name);

            // Build responsive segmented progress bar
            let filled = calculate_filled_cells(model.percentage, bar_width);
            let (zone_success_end, zone_warning_end) = calculate_zone_boundaries(bar_width);

            // Build segmented bar spans (Vero-style: spaced squares)
            let mut bar_spans: Vec<Span> = Vec::new();

            // Segment 1: Success (0-75%)
            let success_chars = filled.min(zone_success_end);
            if success_chars > 0 {
                bar_spans.push(Span::styled(
                    BAR_FILLED.repeat(success_chars),
                    success_style_bold(colors),
                ));
            }

            // Segment 2: Warning (75-90%)
            if filled > zone_success_end {
                let warning_chars = filled
                    .min(zone_warning_end)
                    .saturating_sub(zone_success_end);
                if warning_chars > 0 {
                    bar_spans.push(Span::styled(
                        BAR_FILLED.repeat(warning_chars),
                        warning_style_bold(),
                    ));
                }
            }

            // Segment 3: Error (90-100%)
            if filled > zone_warning_end {
                let error_chars = filled.saturating_sub(zone_warning_end);
                if error_chars > 0 {
                    bar_spans.push(Span::styled(
                        BAR_FILLED.repeat(error_chars),
                        error_style_bold(),
                    ));
                }
            }

            // Empty part (Vero-style: spaced dots)
            let empty_len = bar_width.saturating_sub(filled);
            if empty_len > 0 {
                bar_spans.push(Span::styled(
                    BAR_EMPTY.repeat(empty_len),
                    muted_style(colors),
                ));
            }

            Row::new(vec![
                Cell::from(Span::styled(
                    display_name.to_string(),
                    Style::default().fg(colors.foreground),
                )),
                Cell::from(Line::from(bar_spans)),
                Cell::from(Span::styled(
                    format!("{:^8}", percentage_str),
                    usage_style(model.percentage, colors),
                )),
                Cell::from(Span::styled(usage_str, muted_style(colors))),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25), // Model
            Constraint::Percentage(58), // Progress
            Constraint::Percentage(10), // Usage
            Constraint::Length(7),      // Count (fixed width)
        ],
    )
    .header(Row::new(vec!["Model", "Progress", "Usage", "Count"]).style(header_style(colors)))
    .column_spacing(2);

    let layout = with_horizontal_margin(area);
    f.render_widget(table, layout[0]);
}
