use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::themes::ThemeColors;
use crate::ui::layout::POPUP_WIDTH;
use crate::ui::state::CacheInfo;

/// Renderiza un diálogo con información del cache
pub fn render(f: &mut Frame, colors: &ThemeColors, info: &CacheInfo) {
    // Usar altura fija en lugar de porcentaje para evitar problemas en pantallas pequeñas
    let height = 12;
    let area = centered_rect_fixed_height(POPUP_WIDTH, height, f.area());

    let block = Block::default()
        .title(" Cache Status ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.border));

    let inner = block.inner(area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status
            Constraint::Length(1), // Last updated
            Constraint::Length(1), // TTL
            Constraint::Length(2), // Espaciado
            Constraint::Length(1), // Hint
        ])
        .vertical_margin(1)
        .horizontal_margin(2)
        .split(inner);

    // Status con color
    let (status_text, status_color) = if info.is_fresh {
        ("Status: Fresh", colors.success)
    } else {
        ("Status: Expired", colors.warning)
    };

    let status = Paragraph::new(status_text).style(Style::default().fg(status_color));

    // Last updated
    let last_updated_text = match &info.last_updated {
        Some(time) => format!("Last updated: {}", time),
        None => "Last updated: Never".to_string(),
    };
    let last_updated =
        Paragraph::new(last_updated_text).style(Style::default().fg(colors.foreground));

    // TTL
    let ttl = Paragraph::new(format!("TTL: {} minutes", info.ttl_minutes))
        .style(Style::default().fg(colors.muted));

    // Hint
    let hint = Paragraph::new("Press any key to close")
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));

    // Render
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // Safety check: ensure layout has enough chunks (in case vertical_margin reduces space too much)
    if layout.len() >= 5 {
        f.render_widget(status, layout[0]);
        f.render_widget(last_updated, layout[1]);
        f.render_widget(ttl, layout[2]);
        f.render_widget(hint, layout[4]);
    }
}

/// Helper local para centrar con altura fija
fn centered_rect_fixed_height(
    percent_x: u16,
    height: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
