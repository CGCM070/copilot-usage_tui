use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::themes::ThemeColors;
use crate::ui::layout::{centered_rect, POPUP_HEIGHT, POPUP_WIDTH};
use crate::ui::state::AppStateManager;

use super::super::get_usage_color;

const MIN_VISIBLE_FOR_SCROLL: usize = 5;

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

    let items: Vec<ListItem> = app.themes[start..end]
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let actual_index = start + i;
            let padding = " ".repeat(inner.width as usize - theme.len() - 1);
            let text = format!("{} {}", theme, padding);

            let style = if actual_index == app.selected_theme {
                Style::default()
                    .fg(colors.foreground)
                    .bg(get_usage_color(50.0, colors))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.foreground)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).highlight_symbol("");
    f.render_widget(list, inner);
}
