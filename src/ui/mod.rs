pub mod async_handler;
pub mod components;
pub mod events;
pub mod layout;
pub mod state;

use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

use crate::models::{Theme, UsageStats};
use crate::themes::ThemeColors;

use self::async_handler::{AsyncHandler, AsyncResult};
use self::components::*;
use self::events::EventHandler;
use self::layout::{centered_rect, dashboard_layout};
use self::state::{AppState, AppStateManager};

use std::time::Duration;

/// Formats error for user-friendly display
fn format_error_for_user(error: &anyhow::Error) -> String {
    // Use Display format (user-friendly) instead of Debug
    format!("{}", error)
}

/// Formats error with full debug details
fn format_error_debug(error: &anyhow::Error) -> String {
    // Use Debug format for full stack trace
    format!("{:?}", error)
}

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
    initial_stats: &UsageStats,
    initial_theme: Theme,
    app: &mut AppStateManager,
) -> io::Result<()> {
    let mut theme = initial_theme;
    let mut colors = ThemeColors::from_theme(theme);
    let async_handler = AsyncHandler::new();
    let mut stats = initial_stats.clone();

    loop {
        // Check for pending theme change (instant, in-place)
        if let Some(new_theme) = app.pending_theme_change.take() {
            theme = new_theme;
            colors = ThemeColors::from_theme(theme);
            // Save to config in background (non-blocking)
            async_handler.spawn_save_theme(theme.as_str().to_string());
        }

        terminal.draw(|f| render_ui(f, &stats, &colors, app, theme))?;

        // Poll events con timeout (non-blocking) - cada 50ms para spinner rápido
        if event::poll(Duration::from_millis(50))?
            && let Ok(evt) = event::read()
            && EventHandler::handle_event(app, evt, stats.models.len(), &async_handler)
        {
            return Ok(());
        }

        // Avanzar spinner si estamos en loading
        if matches!(app.state, AppState::LoadingRefresh | AppState::LoadingCache) {
            app.advance_spinner();
        }

        // Check si hay resultados async
        if let Some(result) = async_handler.try_recv() {
            match result {
                AsyncResult::RefreshComplete(Ok(new_stats)) => {
                    stats = new_stats;
                    app.state = AppState::Dashboard;
                }
                AsyncResult::RefreshComplete(Err(e)) => {
                    // Mostrar error limpio para el usuario
                    let error_msg = format_error_for_user(&e);
                    let debug_msg = format_error_debug(&e);
                    app.state = AppState::ShowError {
                        message: error_msg,
                        debug_message: debug_msg,
                        show_debug: false,
                    };
                }
                AsyncResult::CacheInfoReady(info) => {
                    app.state = AppState::ShowCacheInfo(info);
                }
                AsyncResult::ThemeSaved(Ok(())) => {
                    // Theme saved successfully, nothing to do (already applied)
                }
                AsyncResult::ThemeSaved(Err(_)) => {
                    // Silently ignore save errors - theme is already applied visually
                    // User can still use the app, config will be out of sync
                }
            }
        }
    }
}

fn render_ui(f: &mut Frame, stats: &UsageStats, colors: &ThemeColors, app: &AppStateManager, theme: Theme) {
    // With fixed terminal size, use almost full area (96% width, 96% height for small margins)
    let centered_area = centered_rect(96, 96, f.area());

    // Layout principal
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(centered_area);

    let content_area = main_layout[0];
    let help_bar_area = main_layout[1];

    // Layout del contenido
    let content_chunks = dashboard_layout(content_area, stats.models.len());

    // Renderizar componentes del dashboard
    header::render(f, content_chunks[0], stats, colors, theme);
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
        AppState::LoadingRefresh => loading_dialog::render(
            f,
            colors,
            app.get_spinner_char(),
            "Refreshing data from API...",
        ),
        AppState::LoadingCache => {
            loading_dialog::render(f, colors, app.get_spinner_char(), "Loading cache info...")
        }
        AppState::ShowCacheInfo(ref info) => cache_info_dialog::render(f, colors, info),
        AppState::ShowError {
            ref message,
            ref debug_message,
            show_debug,
        } => error_dialog::render(f, colors, message, debug_message, show_debug),
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
                "/: Menu • r: Refresh • t: Theme • ↑↓: Scroll • h: Help • q: Quit"
            } else {
                "/: Menu • r: Refresh • t: Theme • h: Help • q: Quit"
            }
        }
        AppState::CommandMenu => {
            "↑↓/jk: Navigate • Enter: Select • Esc: Close • Letter: Quick jump"
        }
        AppState::ThemeSelector => "↑↓/jk: Navigate • Enter: Select • Esc: Cancel",
        _ => "y: Yes • n: No • Esc: Cancel",
    };

    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(ratatui::style::Style::default().fg(colors.muted));

    f.render_widget(help, area);
}

/// Colores constantes para warning/error (universales)
const WARNING_COLOR: Color = Color::Rgb(255, 184, 108);
const ERROR_COLOR: Color = Color::Rgb(255, 85, 85);

/// Obtiene el color según el porcentaje de uso
fn get_usage_color(percentage: f64, colors: &ThemeColors) -> Color {
    if percentage >= 90.0 {
        ERROR_COLOR
    } else if percentage >= 75.0 {
        WARNING_COLOR
    } else {
        colors.success
    }
}
