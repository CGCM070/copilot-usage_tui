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
pub fn dashboard_layout(area: Rect, model_count: usize) -> Vec<Rect> {
    // Calculate required height for models:
    // Header (1) + Borders (2) + Rows (model_count) + Bottom Padding (1)
    // We add a safety minimum of 4
    let model_height = ((model_count as u16) + 4).max(4);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),            // Header
            Constraint::Length(0),            // Spacing (Reduced to 0)
            Constraint::Length(10),           // Overall usage
            Constraint::Length(0),            // Spacing (Reduced to 0)
            Constraint::Length(model_height), // Model usage (Fixed height)
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
