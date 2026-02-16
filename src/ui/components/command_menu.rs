use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::themes::ThemeColors;
use crate::ui::layout::{centered_rect, POPUP_HEIGHT, POPUP_WIDTH};
use crate::ui::state::AppStateManager;
use crate::ui::styles::get_usage_color;

const MIN_VISIBLE_FOR_SCROLL: usize = 5;

pub fn render(f: &mut Frame, colors: &ThemeColors, app: &AppStateManager) {
    let area = centered_rect(POPUP_WIDTH, POPUP_HEIGHT, f.area());

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

    let total_commands = app.commands.len();

    let (start, end) = if total_commands > MIN_VISIBLE_FOR_SCROLL {
        let visible_rows = inner.height as usize;
        if visible_rows >= total_commands {
            (0, total_commands)
        } else {
            let start = app.command_scroll_offset;
            (start, (start + visible_rows).min(total_commands))
        }
    } else {
        (0, total_commands)
    };

    let items: Vec<ListItem> = app.commands[start..end]
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let actual_index = start + i;
            let shortcut_char = cmd.shortcut.map_or(' ', |s| s.to_ascii_uppercase());
            let label = format!("[{}] {}", shortcut_char, cmd.label);
            let padding = " ".repeat(inner.width as usize - label.len());
            let text = format!("{}{}", label, padding);

            let style = if actual_index == app.selected_command {
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
