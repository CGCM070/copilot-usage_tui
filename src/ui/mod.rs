pub mod components;
pub mod events;
pub mod layout;
pub mod state;

use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    Frame, Terminal,
};

use crate::models::{Theme, UsageStats};
use crate::themes::ThemeColors;

use self::components::*;
use self::events::EventHandler;
use self::layout::{centered_rect, dashboard_layout};
use self::state::{AppState, AppStateManager};

/// Ejecuta la UI interactiva y retorna la acción seleccionada
pub fn run_ui(stats: &UsageStats, theme: Theme) -> Result<Option<String>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppStateManager::new();
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
    app: &mut AppStateManager,
) -> io::Result<()> {
    let colors = ThemeColors::from_theme(theme);

    loop {
        terminal.draw(|f| render_ui(f, stats, &colors, app))?;

        if let Ok(event) = event::read() {
            if EventHandler::handle_event(app, event, stats.models.len()) {
                return Ok(());
            }
        }
    }
}

fn render_ui(f: &mut Frame, stats: &UsageStats, colors: &ThemeColors, app: &AppStateManager) {
    // Área centrada principal
    let centered_area = centered_rect(80, 90, f.area());

    // Layout principal
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(centered_area);

    let content_area = main_layout[0];
    let help_bar_area = main_layout[1];

    // Layout del contenido
    let content_chunks = dashboard_layout(content_area);

    // Renderizar componentes del dashboard
    header::render(f, content_chunks[0], stats, colors);
    usage_overall::render(f, content_chunks[2], stats, colors);
    model_table::render(f, content_chunks[4], stats, colors, app);
    render_help_bar(f, help_bar_area, colors, app, stats.models.len());

    // Renderizar modales según estado
    match app.state {
        AppState::CommandMenu => command_menu::render(f, colors, app),
        AppState::ThemeSelector => theme_selector::render(f, colors, app),
        AppState::ConfirmRefresh => dialogs::render_confirm(
            f,
            colors,
            "Refresh data from API?",
            "This will invalidate cache",
        ),
        AppState::ConfirmReconfigure => dialogs::render_confirm(
            f,
            colors,
            "Reconfigure settings?",
            "Current config will be reset",
        ),
        AppState::ShowHelp => help_dialog::render(f, colors),
        _ => {}
    }
}

fn render_help_bar(
    f: &mut Frame,
    area: Rect,
    colors: &ThemeColors,
    app: &AppStateManager,
    total_models: usize,
) {
    use ratatui::{layout::Alignment, widgets::Paragraph};

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
        .style(ratatui::style::Style::default().fg(colors.muted));

    f.render_widget(help, area);
}

/// Obtiene el color según el porcentaje de uso
fn get_usage_color(percentage: f64, colors: &ThemeColors) -> Color {
    if percentage >= 90.0 {
        colors.error
    } else if percentage >= 75.0 {
        colors.warning
    } else {
        colors.success
    }
}
