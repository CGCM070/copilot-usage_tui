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
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Wrap},
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
            Constraint::Length(3), // Title
            Constraint::Length(8), // Overall usage
            Constraint::Min(10),   // Per-model usage
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Title
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
    .alignment(Alignment::Center)
    .block(Block::default());

    f.render_widget(title, chunks[0]);

    // Overall usage block
    let overall_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title(" Overall Usage ");

    let overall_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Label
            Constraint::Length(1), // Bar
            Constraint::Length(1), // Details
        ])
        .split(overall_block.inner(chunks[1]));

    f.render_widget(overall_block, chunks[1]);

    // Usage label
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
    f.render_widget(usage_label, overall_chunks[0]);

    // Gauge bar
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.bar_filled).bg(colors.bar_empty))
        .percent(stats.percentage as u16)
        .label("");
    f.render_widget(gauge, overall_chunks[1]);

    // Month progress
    let days_in_month = 30.0; // Approximation
    let current_day = Utc::now().day() as f64;
    let month_progress = (current_day / days_in_month) * 100.0;

    let month_label = Paragraph::new(Line::from(vec![
        Span::styled("Month: ", Style::default().fg(colors.foreground)),
        Span::styled(
            format!("{:.1}% of month elapsed", month_progress),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(month_label, overall_chunks[2]);

    // Per-model usage block
    let models_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title(" Per-model Usage ");

    f.render_widget(models_block.clone(), chunks[2]);

    let models_area = models_block.inner(chunks[2]);

    if stats.models.is_empty() {
        let no_data = Paragraph::new("No model usage data available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(colors.muted));
        f.render_widget(no_data, models_area);
    } else {
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

    // Footer
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
    f.render_widget(footer, chunks[3]);
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
