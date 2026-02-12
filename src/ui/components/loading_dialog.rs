use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;

/// Renderiza un diálogo de carga con spinner animado
pub fn render(f: &mut Frame, colors: &ThemeColors, spinner_char: char, message: &str) {
    let area = centered_rect(40, 25, f.area());

    // Block con bordes redondeados
    let block = Block::default()
        .title(" Loading ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.border));

    let inner = block.inner(area);

    // Layout interno: mensaje | spinner | hint
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Mensaje
            Constraint::Length(1), // Spinner
            Constraint::Length(1), // Espaciado
            Constraint::Length(1), // Hint
        ])
        .margin(1)
        .split(inner);

    // Mensaje principal
    let msg = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.foreground));

    // Spinner
    let spinner_text = format!("{}", spinner_char);
    let spinner = Paragraph::new(spinner_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.success));

    // Hint de cancelación
    let hint = Paragraph::new("Press Esc to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));

    // Render
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(msg, layout[0]);
    f.render_widget(spinner, layout[1]);
    f.render_widget(hint, layout[3]);
}
