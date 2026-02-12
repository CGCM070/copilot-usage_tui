use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;
use crate::ui::state::AppStateManager;

use super::super::get_usage_color;

pub fn render(f: &mut Frame, colors: &ThemeColors, app: &AppStateManager) {
    let area = centered_rect(50, 60, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Commands ")
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
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let shortcut_char = cmd.shortcut.map_or(' ', |s| s.to_ascii_uppercase());
            let label = format!("[{}] {}", shortcut_char, cmd.label);
            let padding = " ".repeat(inner.width as usize - label.len());
            let text = format!("{}{}", label, padding);

            let style = if i == app.selected_command {
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
