use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::models::{Theme, UsageStats};
use crate::themes::ThemeColors;

pub fn render(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors, theme: Theme) {
    // Truncar username a 10 caracteres + ".." si es necesario
    let display_username = if stats.username.len() > 10 {
        format!("{}..", &stats.username[..10])
    } else {
        stats.username.clone()
    };

    // Theme name capitalized
    let theme_name = match theme {
        Theme::Dark => "Dark",
        Theme::Light => "Light",
        Theme::Dracula => "Dracula",
        Theme::Nord => "Nord",
        Theme::Monokai => "Monokai",
        Theme::Gruvbox => "Gruvbox",
    };

    // Título simple sin username
    let title_line = Line::from(vec![Span::styled(
        "GitHub Copilot Usage",
        Style::default()
            .fg(colors.foreground)
            .add_modifier(Modifier::BOLD),
    )]);

    // Línea de fecha con username y theme al final
    let date_line = Line::from(vec![
        Span::styled(
            format!("{}", Utc::now().format("%d %B %Y")),
            Style::default().fg(colors.muted),
        ),
        Span::styled(" • ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("Resets: {}", stats.reset_date.format("%b %d")),
            Style::default().fg(colors.muted),
        ),
        Span::styled(" • ", Style::default().fg(colors.muted)),
        Span::styled(display_username, Style::default().fg(colors.muted)),
        Span::styled(" • ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{} ", theme_name),
            Style::default().fg(colors.foreground),
        ),
        // Color dots showing theme palette
        Span::styled("●", Style::default().fg(colors.foreground)),
        Span::styled("●", Style::default().fg(colors.success)),
        Span::styled("●", Style::default().fg(colors.warning)),
        Span::styled("●", Style::default().fg(colors.error)),
        Span::styled("●", Style::default().fg(colors.muted)),
        Span::styled("●", Style::default().fg(colors.border)),
    ]);

    let header = Paragraph::new(Text::from(vec![title_line, date_line]));

    f.render_widget(header, area);
}
