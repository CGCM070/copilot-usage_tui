use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::themes::ThemeColors;
use crate::ui::layout::centered_rect;
use crate::ui::state::CacheInfo;

/// Renderiza un diálogo con información del cache
pub fn render(f: &mut Frame, colors: &ThemeColors, info: &CacheInfo) {
    let area = centered_rect(50, 40, f.area());

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
            Constraint::Length(1), // Espaciado
            Constraint::Length(1), // Hint
        ])
        .margin(2)
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
    f.render_widget(status, layout[0]);
    f.render_widget(last_updated, layout[1]);
    f.render_widget(ttl, layout[2]);
    f.render_widget(hint, layout[4]);
}
