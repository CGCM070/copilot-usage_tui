use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::models::Theme;
use crate::themes::ThemeColors;
use crate::ui::layout::{centered_rect, POPUP_HEIGHT, POPUP_WIDTH};
use crate::ui::state::AppStateManager;

const MIN_VISIBLE_FOR_SCROLL: usize = 5;

/// Generate color preview dots for a theme
fn theme_color_dots(theme_name: &str) -> Vec<Span<'static>> {
    let theme = Theme::from_str(theme_name);
    let colors = ThemeColors::from_theme(theme);

    vec![
        Span::styled("●", Style::default().fg(colors.foreground)),
        Span::styled("●", Style::default().fg(colors.success)),
        Span::styled("●", Style::default().fg(colors.warning)),
        Span::styled("●", Style::default().fg(colors.error)),
        Span::styled("●", Style::default().fg(colors.muted)),
    ]
}

pub fn render(f: &mut Frame, colors: &ThemeColors, app: &AppStateManager) {
    let area = centered_rect(POPUP_WIDTH, POPUP_HEIGHT, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Theme ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let total_themes = app.themes.len();

    let (start, end) = if total_themes > MIN_VISIBLE_FOR_SCROLL {
        let visible_rows = inner.height as usize;
        if visible_rows >= total_themes {
            (0, total_themes)
        } else {
            let start = app.theme_scroll_offset;
            (start, (start + visible_rows).min(total_themes))
        }
    } else {
        (0, total_themes)
    };

    // Calculate space for theme name and dots
    // Format: "themename     ●●●●●"
    let dots_width = 5; // 5 color dots
    let available_name_width = (inner.width as usize).saturating_sub(dots_width + 2); // 2 for spacing

    let items: Vec<ListItem> = app.themes[start..end]
        .iter()
        .enumerate()
        .map(|(i, theme_name)| {
            let actual_index = start + i;

            // Build the line with theme name and color dots
            let mut spans: Vec<Span> = Vec::new();

            // Theme name (padded to fixed width)
            let display_name = if theme_name.len() > available_name_width {
                &theme_name[..available_name_width]
            } else {
                theme_name
            };
            let padding = available_name_width.saturating_sub(display_name.len());

            let name_style = if actual_index == app.selected_theme {
                Style::default()
                    .fg(colors.success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.foreground)
            };

            spans.push(Span::styled(
                format!(" {}{} ", display_name, " ".repeat(padding)),
                name_style,
            ));

            // Add color dots for this theme
            spans.extend(theme_color_dots(theme_name));

            let line = Line::from(spans);

            // Apply background highlight for selected item
            let item_style = if actual_index == app.selected_theme {
                Style::default().bg(colors.bar_empty)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).highlight_symbol("");
    f.render_widget(list, inner);
}
