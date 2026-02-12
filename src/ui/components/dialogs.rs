use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;

pub fn render_confirm(f: &mut Frame, colors: &ThemeColors, title: &str, message: &str) {
    let area = centered_rect(50, 20, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.warning))
        .title_style(
            Style::default()
                .fg(colors.warning)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = Paragraph::new(Text::from(vec![
        Line::from(message),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "y",
                Style::default()
                    .fg(colors.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Yes  ", Style::default().fg(colors.muted)),
            Span::styled(
                "n",
                Style::default()
                    .fg(colors.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": No", Style::default().fg(colors.muted)),
        ]),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(text, inner);
}
