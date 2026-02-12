use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;

/// Renderiza un diálogo de error con mensaje limpio para el usuario
pub fn render(
    f: &mut Frame,
    colors: &ThemeColors,
    message: &str,
    debug_message: &str,
    show_debug: bool,
) {
    let area = centered_rect(70, 60, f.area());

    let title = if show_debug {
        " Error (Debug) "
    } else {
        " Error "
    };

    let block = Block::default()
        .title(title)
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

    // Show user message or debug message based on toggle
    let display_message = if show_debug { debug_message } else { message };

    // Error message con wrap para manejar mensajes largos
    let error = Paragraph::new(display_message)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(colors.error));

    // Hint - mostrar opción de debug si no está activo
    let hint_text = if show_debug {
        "Press 'd' to hide details, any other key to close"
    } else {
        "Press 'd' for details, any other key to close"
    };

    let hint = Paragraph::new(hint_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));

    // Render
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(error, layout[0]);
    f.render_widget(hint, layout[2]);
}
