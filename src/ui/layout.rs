use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Crea un rectángulo centrado con porcentajes dados
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
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

/// Layout principal del dashboard
pub fn dashboard_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(1),  // Spacing
            Constraint::Length(10), // Overall usage
            Constraint::Length(1),  // Spacing
            Constraint::Min(10),    // Model usage
        ])
        .split(area)
        .to_vec()
}

/// Layout del área de contenido centrada
pub fn centered_content_layout(area: Rect) -> (Rect, Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    (main_layout[0], main_layout[1]) // (content, help_bar)
}
