use ratatui::{
    Frame,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;
use crate::ui::state::AppStateManager;

use super::super::get_usage_color;

pub fn render(f: &mut Frame, colors: &ThemeColors, app: &AppStateManager) {
    let area = centered_rect(40, 40, f.area());

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

    let items: Vec<ListItem> = app
        .themes
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let padding = " ".repeat(inner.width as usize - theme.len());
            let text = format!("{}{}", theme, padding);

            let style = if i == app.selected_theme {
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
