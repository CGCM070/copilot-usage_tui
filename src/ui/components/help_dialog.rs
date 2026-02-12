use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;

pub fn render(f: &mut Frame, colors: &ThemeColors) {
    let area = centered_rect(70, 70, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Global:",
            Style::default().fg(colors.success),
        )]),
        Line::from("  / or :        Open command menu"),
        Line::from("  r             Refresh data from API"),
        Line::from("  t             Change theme"),
        Line::from("  h             Show this help"),
        Line::from("  q             Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "In Menus:",
            Style::default().fg(colors.success),
        )]),
        Line::from("  ↑/↓ or j/k    Navigate"),
        Line::from("  Enter         Select item"),
        Line::from("  Esc           Close/Cancel"),
        Line::from("  Letter        Quick jump to command"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().fg(colors.success),
        )]),
        Line::from("  The dashboard shows your Copilot usage"),
        Line::from("  statistics in real-time."),
    ];

    let help = Paragraph::new(Text::from(help_text));
    f.render_widget(help, inner);
}
