use crate::models::{Theme, UsageStats};
use crate::themes::ThemeColors;
use anyhow::Result;
use chrono::{Datelike, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;

pub fn run_ui(stats: &UsageStats, theme: Theme) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, stats, theme);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    stats: &UsageStats,
    theme: Theme,
) -> io::Result<()> {
    let colors = ThemeColors::from_theme(theme);

    loop {
        terminal.draw(|f| ui(f, stats, &colors))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, stats: &UsageStats, colors: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title(f, chunks[0], stats, colors);
    render_overall_usage(f, chunks[1], stats, colors);
    render_model_usage(f, chunks[2], stats, colors);
    render_footer(f, chunks[3], stats, colors);
}

fn render_title(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
) {
    let title = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            "GitHub Copilot Pro — Premium Requests Usage",
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!(
                "{} {} • {}",
                Utc::now().format("%B %Y"),
                Utc::now().format("•"),
                stats.reset_date.format("Resets: %B %d, %Y at %H:%M UTC")
            ),
            Style::default().fg(colors.muted),
        )]),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn render_overall_usage(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
) {
    let overall_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title(" Overall Usage ");

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(overall_block.inner(area));

    f.render_widget(overall_block, area);

    let usage_label = Paragraph::new(Line::from(vec![
        Span::styled("Usage: ", Style::default().fg(colors.foreground)),
        Span::styled(
            format!(
                "{:.0}/{:.0} ({:.1}%)",
                stats.total_used, stats.total_limit, stats.percentage
            ),
            Style::default()
                .fg(get_usage_color(stats.percentage, colors))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    f.render_widget(usage_label, inner_chunks[0]);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.bar_filled).bg(colors.bar_empty))
        .percent(stats.percentage as u16)
        .label("");
    f.render_widget(gauge, inner_chunks[1]);

    let days_in_month = days_in_current_month();
    let current_day = Utc::now().day() as f64;
    let month_progress = (current_day / days_in_month as f64) * 100.0;

    let month_label = Paragraph::new(Line::from(vec![
        Span::styled("Month: ", Style::default().fg(colors.foreground)),
        Span::styled(
            format!("{:.1}% of month elapsed", month_progress),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(month_label, inner_chunks[2]);
}

fn render_model_usage(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
) {
    let models_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title(" Per-model Usage ");

    f.render_widget(models_block.clone(), area);

    let models_area = models_block.inner(area);

    if stats.models.is_empty() {
        let no_data = Paragraph::new("No model usage data available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(colors.muted));
        f.render_widget(no_data, models_area);
        return;
    }

    let rows: Vec<Row> = stats
        .models
        .iter()
        .map(|model| {
            let bar_width = 20;
            let filled = ((model.percentage / 100.0) * bar_width as f64) as usize;
            let bar = format!("{:<width$}", "█".repeat(filled), width = bar_width);

            Row::new(vec![
                Cell::from(model.name.clone()),
                Cell::from(format!("{:.0}/{:.0}", model.used, model.limit)),
                Cell::from(Span::styled(
                    bar,
                    Style::default().fg(get_usage_color(model.percentage, colors)),
                )),
                Cell::from(format!("{:.1}%", model.percentage)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ],
    )
    .header(
        Row::new(vec!["Model", "Usage", "Visual", "%"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    );

    f.render_widget(table, models_area);
}

fn render_footer(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
) {
    let footer_text = if stats.estimated_cost > 0.0 {
        format!(
            "Press 'q' or ESC to exit | Estimated cost: ${:.2}",
            stats.estimated_cost
        )
    } else {
        "Press 'q' or ESC to exit".to_string()
    };

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));
    f.render_widget(footer, area);
}

fn get_usage_color(percentage: f64, colors: &ThemeColors) -> Color {
    if percentage >= 90.0 {
        colors.error
    } else if percentage >= 75.0 {
        colors.warning
    } else {
        colors.success
    }
}

fn days_in_current_month() -> u32 {
    let now = Utc::now();
    let next_month = if now.month() == 12 {
        now.with_month(1)
            .unwrap()
            .with_year(now.year() + 1)
            .unwrap()
    } else {
        now.with_month(now.month() + 1).unwrap()
    };
    (next_month - now).num_days() as u32 + now.day()
}
