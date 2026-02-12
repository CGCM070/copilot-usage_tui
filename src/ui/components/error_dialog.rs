use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;

/// Renderiza un diálogo de error con stack trace completo
pub fn render(f: &mut Frame, colors: &ThemeColors, error_msg: &str) {
    let area = centered_rect(70, 60, f.area());

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.error));

    let inner = block.inner(area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Error message (expandible)
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Hint
        ])
        .margin(2)
        .split(inner);

    // Error message con wrap para manejar mensajes largos
    let error = Paragraph::new(error_msg)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(colors.error));

    // Hint
    let hint = Paragraph::new("Press any key to close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));

    // Render
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(error, layout[0]);
    f.render_widget(hint, layout[2]);
}
