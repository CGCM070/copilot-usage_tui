use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::models::{Theme, UsageStats};
use crate::themes::ThemeColors;
use crate::ui::styles::{ICON_CALENDAR, ICON_RESET, ICON_ROBOT, ICON_THEME, ICON_USER};

pub fn render(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors, theme: Theme) {
    // Check for compact mode
    let is_compact = area.width < 60;

    // Truncar username a 10 caracteres + ".." si es necesario
    let display_username = if stats.username.len() > 10 {
        format!("{}..", &stats.username[..10])
    } else {
        stats.username.clone()
    };

    // Theme name capitalized
    let theme_name = match theme {
        Theme::Dark => "Dark",
        Theme::Nord => "Nord",
        Theme::Monokai => "Monokai",
        Theme::Gruvbox => "Gruvbox",
        Theme::Catppuccin => "Catppuccin",
        Theme::OneDark => "OneDark",
        Theme::TokyoNight => "TokyoNight",
        Theme::SolarizedDark => "Solarized",
        Theme::Kanagawa => "Kanagawa",
    };

    if is_compact {
        render_compact(f, area, stats, colors, theme_name, display_username);
    } else {
        render_full(f, area, stats, colors, theme_name, display_username);
    }
}

fn render_compact(
    f: &mut Frame,
    area: Rect,
    _stats: &UsageStats,
    colors: &ThemeColors,
    theme_name: &str,
    display_username: String,
) {
    // Split area into rows: title, spacer, and info line
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .horizontal_margin(1)
        .split(area);

    // Title row (left aligned)
    let title = Paragraph::new(Line::from(vec![Span::styled(
        format!("{} GitHub Copilot Usage", ICON_ROBOT),
        Style::default()
            .fg(colors.foreground)
            .add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(title, rows[0]);

    // Info row: split into left (username only) and right (theme)
    let info_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(rows[2]);

    // Left side: only username
    let user_info = Paragraph::new(Line::from(vec![Span::styled(
        format!("{} {}", ICON_USER, display_username),
        Style::default().fg(colors.muted),
    )]));
    f.render_widget(user_info, info_cols[0]);

    // Right side: theme name + color dots (right aligned)
    let theme_info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} {} ", ICON_THEME, theme_name),
            Style::default().fg(colors.foreground),
        ),
        Span::styled("●", Style::default().fg(colors.foreground)),
        Span::styled("●", Style::default().fg(colors.success)),
        Span::styled("●", Style::default().fg(colors.warning)),
        Span::styled("●", Style::default().fg(colors.error)),
        Span::styled("●", Style::default().fg(colors.muted)),
        Span::styled("●", Style::default().fg(colors.border)),
    ]))
    .alignment(Alignment::Right);
    f.render_widget(theme_info, info_cols[1]);
}

fn render_full(
    f: &mut Frame,
    area: Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
    theme_name: &str,
    display_username: String,
) {
    // Split area into rows: title, spacer, and info line
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .horizontal_margin(1)
        .split(area);

    // Title row (left aligned)
    let title = Paragraph::new(Line::from(vec![Span::styled(
        format!("{} GitHub Copilot Usage", ICON_ROBOT),
        Style::default()
            .fg(colors.foreground)
            .add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(title, rows[0]);

    // Info row: split into left (date/user) and right (theme)
    let info_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(rows[2]);

    // Left side: date, reset, username
    let date_info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} {}", ICON_CALENDAR, Local::now().format("%d %B %Y")),
            Style::default().fg(colors.muted),
        ),
        Span::styled(" • ", Style::default().fg(colors.muted)),
        Span::styled(
            format!(
                "{} Resets: {}",
                ICON_RESET,
                stats.reset_date.format("%b %d")
            ),
            Style::default().fg(colors.muted),
        ),
        Span::styled(" • ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{} {}", ICON_USER, display_username),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(date_info, info_cols[0]);

    // Right side: theme name + color dots (right aligned)
    let theme_info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} {} ", ICON_THEME, theme_name),
            Style::default().fg(colors.foreground),
        ),
        Span::styled("●", Style::default().fg(colors.foreground)),
        Span::styled("●", Style::default().fg(colors.success)),
        Span::styled("●", Style::default().fg(colors.warning)),
        Span::styled("●", Style::default().fg(colors.error)),
        Span::styled("●", Style::default().fg(colors.muted)),
        Span::styled("●", Style::default().fg(colors.border)),
    ]))
    .alignment(Alignment::Right);
    f.render_widget(theme_info, info_cols[1]);
}
