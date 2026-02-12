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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;

/// Estados de la aplicación
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Dashboard,
    CommandMenu,
    ThemeSelector,
    ConfirmRefresh,
    ConfirmReconfigure,
    ShowHelp,
}

/// Comandos disponibles en el menú
#[derive(Debug, Clone)]
pub struct Command {
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: Option<char>,
}

pub struct App {
    pub state: AppState,
    pub selected_command: usize,
    pub selected_theme: usize,
    pub model_scroll_offset: usize,
    pub commands: Vec<Command>,
    pub themes: Vec<&'static str>,
    pub action_taken: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Dashboard,
            selected_command: 0,
            selected_theme: 0,
            model_scroll_offset: 0,
            commands: vec![
                Command {
                    id: "refresh",
                    label: "Refresh Data",
                    shortcut: Some('r'),
                },
                Command {
                    id: "theme",
                    label: "Change Theme",
                    shortcut: Some('t'),
                },
                Command {
                    id: "reconfigure",
                    label: "Reconfigure",
                    shortcut: Some('c'),
                },
                Command {
                    id: "cache",
                    label: "Cache Status",
                    shortcut: Some('s'),
                },
                Command {
                    id: "help",
                    label: "Help",
                    shortcut: Some('h'),
                },
                Command {
                    id: "quit",
                    label: "Quit",
                    shortcut: Some('q'),
                },
            ],
            themes: vec!["dark", "light", "dracula", "nord", "monokai", "gruvbox"],
            action_taken: None,
        }
    }

    pub fn next_command(&mut self) {
        self.selected_command = (self.selected_command + 1) % self.commands.len();
    }

    pub fn previous_command(&mut self) {
        if self.selected_command == 0 {
            self.selected_command = self.commands.len() - 1;
        } else {
            self.selected_command -= 1;
        }
    }

    pub fn next_theme(&mut self) {
        self.selected_theme = (self.selected_theme + 1) % self.themes.len();
    }

    pub fn previous_theme(&mut self) {
        if self.selected_theme == 0 {
            self.selected_theme = self.themes.len() - 1;
        } else {
            self.selected_theme -= 1;
        }
    }

    pub fn get_selected_command_id(&self) -> &str {
        self.commands[self.selected_command].id
    }

    pub fn scroll_models_down(&mut self, total_models: usize, visible_count: usize) {
        if self.model_scroll_offset + visible_count < total_models {
            self.model_scroll_offset += 1;
        }
    }

    pub fn scroll_models_up(&mut self) {
        if self.model_scroll_offset > 0 {
            self.model_scroll_offset -= 1;
        }
    }
}

pub fn run_ui(stats: &UsageStats, theme: Theme) -> Result<Option<String>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, stats, theme, &mut app);

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

    Ok(app.action_taken)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    stats: &UsageStats,
    theme: Theme,
    app: &mut App,
) -> io::Result<()> {
    let colors = ThemeColors::from_theme(theme);

    loop {
        terminal.draw(|f| ui(f, stats, &colors, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.state {
                    AppState::Dashboard => match key.code {
                        KeyCode::Char('/') | KeyCode::Char(':') => {
                            app.state = AppState::CommandMenu;
                        }
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('r') => {
                            app.action_taken = Some("refresh".to_string());
                            return Ok(());
                        }
                        KeyCode::Char('t') => {
                            app.state = AppState::ThemeSelector;
                        }
                        KeyCode::Char('h') => {
                            app.state = AppState::ShowHelp;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.scroll_models_down(stats.models.len(), 8);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.scroll_models_up();
                        }
                        _ => {}
                    },
                    AppState::CommandMenu => match key.code {
                        KeyCode::Esc => app.state = AppState::Dashboard,
                        KeyCode::Down | KeyCode::Char('j') => app.next_command(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_command(),
                        KeyCode::Enter => {
                            execute_command(app);
                            if app.action_taken.is_some() {
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(pos) = app.commands.iter().position(|cmd| {
                                cmd.shortcut.map_or(false, |s| s == c.to_ascii_lowercase())
                            }) {
                                app.selected_command = pos;
                                execute_command(app);
                                if app.action_taken.is_some() {
                                    return Ok(());
                                }
                            }
                        }
                        _ => {}
                    },
                    AppState::ThemeSelector => match key.code {
                        KeyCode::Esc => app.state = AppState::Dashboard,
                        KeyCode::Down | KeyCode::Char('j') => app.next_theme(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_theme(),
                        KeyCode::Enter => {
                            app.action_taken =
                                Some(format!("theme:{}", app.themes[app.selected_theme]));
                            return Ok(());
                        }
                        _ => {}
                    },
                    AppState::ConfirmRefresh => match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            app.action_taken = Some("refresh".to_string());
                            return Ok(());
                        }
                        KeyCode::Char('n') | KeyCode::Esc => app.state = AppState::Dashboard,
                        _ => {}
                    },
                    AppState::ConfirmReconfigure => match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            app.action_taken = Some("reconfigure".to_string());
                            return Ok(());
                        }
                        KeyCode::Char('n') | KeyCode::Esc => app.state = AppState::Dashboard,
                        _ => {}
                    },
                    AppState::ShowHelp => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => app.state = AppState::Dashboard,
                        _ => {}
                    },
                }
            }
        }
    }
}

fn execute_command(app: &mut App) {
    match app.get_selected_command_id() {
        "refresh" => app.state = AppState::ConfirmRefresh,
        "theme" => app.state = AppState::ThemeSelector,
        "reconfigure" => app.state = AppState::ConfirmReconfigure,
        "cache" => {
            app.action_taken = Some("cache".to_string());
        }
        "help" => app.state = AppState::ShowHelp,
        "quit" => {
            app.action_taken = Some("quit".to_string());
        }
        _ => {}
    }
}

fn ui(f: &mut Frame, stats: &UsageStats, colors: &ThemeColors, app: &App) {
    // Centrar todo el contenido
    let centered_area = centered_rect(80, 90, f.area());

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(centered_area);

    let content_area = main_layout[0];
    let help_bar_area = main_layout[1];

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(10),
            Constraint::Length(1),
            Constraint::Min(10),
        ])
        .split(content_area);

    render_header(f, content_layout[0], stats, colors);
    render_overall_usage(f, content_layout[2], stats, colors);
    render_model_usage(f, content_layout[4], stats, colors, app);
    render_help_bar(f, help_bar_area, colors, app, stats.models.len());

    match app.state {
        AppState::CommandMenu => render_command_menu(f, colors, app),
        AppState::ThemeSelector => render_theme_selector(f, colors, app),
        AppState::ConfirmRefresh => render_confirm_dialog(
            f,
            colors,
            "Refresh data from API?",
            "This will invalidate cache",
        ),
        AppState::ConfirmReconfigure => render_confirm_dialog(
            f,
            colors,
            "Reconfigure settings?",
            "Current config will be reset",
        ),
        AppState::ShowHelp => render_help_dialog(f, colors),
        _ => {}
    }
}

fn render_header(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors) {
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
                format!("{}", Utc::now().format("%B %Y")),
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

fn render_overall_usage(f: &mut Frame, area: Rect, stats: &UsageStats, colors: &ThemeColors) {
    let block = Block::default()
        .title(" Overall Usage ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(inner);

    let usage_text = Paragraph::new(Line::from(vec![
        Span::styled("Requests: ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{:.0}/{:.0}", stats.total_used, stats.total_limit),
            Style::default()
                .fg(get_usage_color(stats.percentage, colors))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({:.1}%)", stats.percentage),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(usage_text, layout[0]);

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(get_usage_color(stats.percentage, colors))
                .bg(colors.bar_empty),
        )
        .percent(stats.percentage as u16)
        .label("");
    f.render_widget(gauge, layout[1]);

    let days_in_month = days_in_current_month();
    let current_day = Utc::now().day() as f64;
    let month_progress = (current_day / days_in_month as f64) * 100.0;

    let month_text = Paragraph::new(Line::from(vec![
        Span::styled("Month: ", Style::default().fg(colors.muted)),
        Span::styled(
            format!("{:.1}% elapsed", month_progress),
            Style::default().fg(colors.muted),
        ),
    ]));
    f.render_widget(month_text, layout[2]);
}

fn render_model_usage(
    f: &mut Frame,
    area: Rect,
    stats: &UsageStats,
    colors: &ThemeColors,
    app: &App,
) {
    let block = Block::default()
        .title(" Per-Model Usage ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    if stats.models.is_empty() {
        let no_data = Paragraph::new("No model usage data available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(colors.muted));
        f.render_widget(no_data, inner);
        return;
    }

    // Calcular cuántos modelos caben (aproximadamente 1 línea por modelo)
    let visible_count = (inner.height as usize).saturating_sub(1); // -1 para el header
    let total_models = stats.models.len();
    let scroll = app
        .model_scroll_offset
        .min(total_models.saturating_sub(visible_count));

    // Obtener solo los modelos visibles
    let visible_models: Vec<_> = stats
        .models
        .iter()
        .skip(scroll)
        .take(visible_count)
        .collect();

    let rows: Vec<Row> = visible_models
        .iter()
        .map(|model| {
            let percentage_str = format!("{:>6.1}%", model.percentage);
            let usage_str = format!("{:>6.0}/{:.0}", model.used, model.limit);

            // Barra visual sin fondo verde - solo caracteres
            let bar_width = 15;
            let filled = ((model.percentage / 100.0) * bar_width as f64) as usize;
            let bar = "█".repeat(filled);
            let empty = "░".repeat(bar_width - filled);

            Row::new(vec![
                Cell::from(Span::styled(
                    model.name.clone(),
                    Style::default().fg(colors.foreground),
                )),
                Cell::from(Line::from(vec![
                    Span::styled(
                        bar,
                        Style::default().fg(get_usage_color(model.percentage, colors)),
                    ),
                    Span::styled(empty, Style::default().fg(colors.muted)),
                ])),
                Cell::from(Span::styled(
                    percentage_str,
                    Style::default().fg(get_usage_color(model.percentage, colors)),
                )),
                Cell::from(Span::styled(usage_str, Style::default().fg(colors.muted))),
            ])
        })
        .collect();

    let mut table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Model", "Progress", "Usage", "Count"]).style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        ),
    );

    // Agregar indicadores de scroll si es necesario
    let mut title = " Per-Model Usage ".to_string();
    if scroll > 0 {
        title.insert_str(0, "↑ ");
    }
    if scroll + visible_count < total_models {
        title.push_str(" ↓");
    }

    f.render_widget(table, inner);
}

fn render_help_bar(
    f: &mut Frame,
    area: Rect,
    colors: &ThemeColors,
    app: &App,
    total_models: usize,
) {
    let help_text = match app.state {
        AppState::Dashboard => {
            if total_models > 8 {
                "/: Menu  •  r: Refresh  •  t: Theme  •  ↑↓: Scroll  •  h: Help  •  q: Quit"
            } else {
                "/: Menu  •  r: Refresh  •  t: Theme  •  h: Help  •  q: Quit"
            }
        }
        AppState::CommandMenu => {
            "↑↓/jk: Navigate  •  Enter: Select  •  Esc: Close  •  Letter: Quick jump"
        }
        AppState::ThemeSelector => "↑↓/jk: Navigate  •  Enter: Select  •  Esc: Cancel",
        _ => "y: Yes  •  n: No  •  Esc: Cancel",
    };

    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.muted));

    f.render_widget(help, area);
}

fn render_command_menu(f: &mut Frame, colors: &ThemeColors, app: &App) {
    let area = centered_rect(50, 60, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Commands ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Crear items del menú con selección de ancho completo
    let items: Vec<ListItem> = app
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let shortcut_char = cmd.shortcut.map_or(' ', |s| s.to_ascii_uppercase());
            let label = format!("[{}] {}", shortcut_char, cmd.label);
            let padding = " ".repeat(inner.width as usize - label.len());
            let text = format!("{}{}", label, padding);

            let style = if i == app.selected_command {
                Style::default()
                    .fg(colors.foreground)
                    .bg(get_usage_color(50.0, colors))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.foreground)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).highlight_symbol("");
    f.render_widget(list, inner);
}

fn render_theme_selector(f: &mut Frame, colors: &ThemeColors, app: &App) {
    let area = centered_rect(40, 40, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Theme ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .themes
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let padding = " ".repeat(inner.width as usize - theme.len());
            let text = format!("{}{}", theme, padding);

            let style = if i == app.selected_theme {
                Style::default()
                    .fg(colors.foreground)
                    .bg(get_usage_color(50.0, colors))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.foreground)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).highlight_symbol("");
    f.render_widget(list, inner);
}

fn render_confirm_dialog(f: &mut Frame, colors: &ThemeColors, title: &str, message: &str) {
    let area = centered_rect(50, 20, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.warning))
        .title_style(
            Style::default()
                .fg(colors.warning)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = Paragraph::new(Text::from(vec![
        Line::from(message),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "y",
                Style::default()
                    .fg(colors.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": Yes  ", Style::default().fg(colors.muted)),
            Span::styled(
                "n",
                Style::default()
                    .fg(colors.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": No", Style::default().fg(colors.muted)),
        ]),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(text, inner);
}

fn render_help_dialog(f: &mut Frame, colors: &ThemeColors) {
    let area = centered_rect(70, 70, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border))
        .title_style(
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);
    f.render_widget(block, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(colors.foreground)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Global:",
            Style::default().fg(colors.success),
        )]),
        Line::from("  / or :        Open command menu"),
        Line::from("  r             Refresh data from API"),
        Line::from("  t             Change theme"),
        Line::from("  h             Show this help"),
        Line::from("  q             Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "In Menus:",
            Style::default().fg(colors.success),
        )]),
        Line::from("  ↑/↓ or j/k    Navigate"),
        Line::from("  Enter         Select item"),
        Line::from("  Esc           Close/Cancel"),
        Line::from("  Letter        Quick jump to command"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().fg(colors.success),
        )]),
        Line::from("  The dashboard shows your Copilot usage"),
        Line::from("  statistics in real-time."),
    ];

    let help = Paragraph::new(Text::from(help_text));
    f.render_widget(help, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
