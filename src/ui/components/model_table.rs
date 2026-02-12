use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
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
    let mut title = " Per-Model Usage ".to_string();
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
            let percentage_str = format!("{:>6.1}%", model.percentage);
            let usage_str = format!("{:>6.0}/{:.0}", model.used, model.limit);

            // Barra visual sin fondo
            let bar_width = 15;
            let filled = ((model.percentage / 100.0) * bar_width as f64) as usize;
            let bar = "█".repeat(filled);
            let empty = "░".repeat(bar_width - filled);

            Row::new(vec![
                Cell::from(Span::styled(
                    model.name.clone(),
                    Style::default().fg(colors.foreground),
                )),
                Cell::from(Line::from(vec![
                    Span::styled(
                        bar,
                        Style::default().fg(get_usage_color(model.percentage, colors)),
                    ),
                    Span::styled(empty, Style::default().fg(colors.muted)),
                ])),
                Cell::from(Span::styled(
                    percentage_str,
                    Style::default().fg(get_usage_color(model.percentage, colors)),
                )),
                Cell::from(Span::styled(usage_str, Style::default().fg(colors.muted))),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Model", "Progress", "Usage", "Count"]).style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    );

    f.render_widget(table, area);
}
