use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::models::Theme;

use super::async_handler::AsyncHandler;
use super::state::{AppState, AppStateManager};

/// Procesa eventos de teclado y actualiza el estado de la aplicación
pub struct EventHandler;

impl EventHandler {
    pub fn handle_event(
        app: &mut AppStateManager,
        event: Event,
        total_models: usize,
        async_handler: &AsyncHandler,
    ) -> bool {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            return Self::handle_key_press(app, key.code, total_models, async_handler);
        }
        false
    }

    fn handle_key_press(
        app: &mut AppStateManager,
        code: KeyCode,
        total_models: usize,
        async_handler: &AsyncHandler,
    ) -> bool {
        match app.state {
            AppState::Dashboard => Self::handle_dashboard(app, code, total_models),
            AppState::CommandMenu => Self::handle_command_menu(app, code, async_handler),
            AppState::ThemeSelector => Self::handle_theme_selector(app, code),
            AppState::ConfirmRefresh => Self::handle_confirm_refresh(app, code, async_handler),
            AppState::ConfirmReconfigure => Self::handle_confirm_reconfigure(app, code),
            AppState::ShowHelp => Self::handle_help(app, code),
            AppState::LoadingRefresh | AppState::LoadingCache => Self::handle_loading(app, code),
            AppState::ShowCacheInfo(_) => Self::handle_cache_info(app, code),
            AppState::ShowError { .. } => Self::handle_error(app, code),
        }
    }

    fn handle_dashboard(app: &mut AppStateManager, code: KeyCode, total_models: usize) -> bool {
        match code {
            KeyCode::Char('/') | KeyCode::Char(':') => {
                app.state = AppState::CommandMenu;
            }
            KeyCode::Char('q') => {
                app.action_taken = Some("quit".to_string());
                return true;
            }
            KeyCode::Char('r') => {
                app.state = AppState::ConfirmRefresh;
            }
            KeyCode::Char('t') => {
                app.state = AppState::ThemeSelector;
            }
            KeyCode::Char('h') => {
                app.state = AppState::ShowHelp;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_models_down(total_models, 8);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_models_up();
            }
            _ => {}
        }
        false
    }

    fn handle_command_menu(
        app: &mut AppStateManager,
        code: KeyCode,
        async_handler: &AsyncHandler,
    ) -> bool {
        match code {
            KeyCode::Esc => {
                app.state = AppState::Dashboard;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_command();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_command();
            }
            KeyCode::Enter => {
                return Self::execute_selected_command(app, async_handler);
            }
            KeyCode::Char(c) => {
                // Atajo rápido por letra
                if let Some(pos) = app
                    .commands
                    .iter()
                    .position(|cmd| cmd.shortcut.is_some_and(|s| s == c.to_ascii_lowercase()))
                {
                    app.selected_command = pos;
                    return Self::execute_selected_command(app, async_handler);
                }
            }
            _ => {}
        }
        false
    }

    fn handle_theme_selector(app: &mut AppStateManager, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => {
                app.state = AppState::Dashboard;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_theme();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_theme();
            }
            KeyCode::Enter => {
                // Apply theme change in-place (don't exit TUI)
                let theme_name = app.themes[app.selected_theme];
                let new_theme = Theme::from_str(theme_name);
                app.pending_theme_change = Some(new_theme);
                app.state = AppState::Dashboard;
            }
            _ => {}
        }
        false
    }

    fn handle_confirm_refresh(
        app: &mut AppStateManager,
        code: KeyCode,
        async_handler: &AsyncHandler,
    ) -> bool {
        match code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.state = AppState::LoadingRefresh;
                async_handler.spawn_refresh();
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.state = AppState::Dashboard;
            }
            _ => {}
        }
        false
    }

    fn handle_confirm_reconfigure(app: &mut AppStateManager, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.action_taken = Some("reconfigure".to_string());
                return true;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.state = AppState::Dashboard;
            }
            _ => {}
        }
        false
    }

    fn handle_help(app: &mut AppStateManager, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.state = AppState::Dashboard;
            }
            _ => {}
        }
        false
    }

    fn execute_selected_command(app: &mut AppStateManager, async_handler: &AsyncHandler) -> bool {
        match app.get_selected_command_id() {
            "refresh" => app.state = AppState::ConfirmRefresh,
            "theme" => app.state = AppState::ThemeSelector,
            "reconfigure" => app.state = AppState::ConfirmReconfigure,
            "cache" => {
                app.state = AppState::LoadingCache;
                async_handler.spawn_cache_info();
            }
            "help" => app.state = AppState::ShowHelp,
            "quit" => {
                app.action_taken = Some("quit".to_string());
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_loading(app: &mut AppStateManager, code: KeyCode) -> bool {
        if matches!(code, KeyCode::Esc) {
            app.state = AppState::Dashboard;
        }
        false
    }

    fn handle_cache_info(app: &mut AppStateManager, _code: KeyCode) -> bool {
        app.state = AppState::Dashboard;
        false
    }

    fn handle_error(app: &mut AppStateManager, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('d') => {
                // Toggle debug view
                if let AppState::ShowError {
                    message,
                    debug_message,
                    show_debug,
                } = &app.state
                {
                    app.state = AppState::ShowError {
                        message: message.clone(),
                        debug_message: debug_message.clone(),
                        show_debug: !show_debug,
                    };
                }
            }
            _ => {
                app.state = AppState::Dashboard;
            }
        }
        false
    }
}
