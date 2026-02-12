use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::models::UsageStats;
use crate::themes::ThemeColors;

pub fn render(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors) {
    // Título con CGCM070 a la derecha
    let title_line = Line::from(vec![
        Span::styled(
            "GitHub Copilot Usage",
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:>width$}", "CGCM070", width = area.width as usize - 20),
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let header = Paragraph::new(Text::from(vec![
        title_line,
        Line::from(vec![
            Span::styled(
                format!("{}", Utc::now().format("%d %B %Y")),
                Style::default().fg(colors.muted),
            ),
            Span::styled(" • ", Style::default().fg(colors.muted)),
            Span::styled(
                format!("Resets: {}", stats.reset_date.format("%b %d")),
                Style::default().fg(colors.muted),
            ),
        ]),
    ]));

    f.render_widget(header, area);
}
