use crossterm::event::{Event, KeyCode, KeyEventKind};

use super::state::{AppState, AppStateManager};

/// Procesa eventos de teclado y actualiza el estado de la aplicación
pub struct EventHandler;

impl EventHandler {
    pub fn handle_event(app: &mut AppStateManager, event: Event, total_models: usize) -> bool {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                return Self::handle_key_press(app, key.code, total_models);
            }
        }
        false
    }

    fn handle_key_press(app: &mut AppStateManager, code: KeyCode, total_models: usize) -> bool {
        match app.state {
            AppState::Dashboard => Self::handle_dashboard(app, code, total_models),
            AppState::CommandMenu => Self::handle_command_menu(app, code),
            AppState::ThemeSelector => Self::handle_theme_selector(app, code),
            AppState::ConfirmRefresh => Self::handle_confirm(app, code, "refresh"),
            AppState::ConfirmReconfigure => Self::handle_confirm(app, code, "reconfigure"),
            AppState::ShowHelp => Self::handle_help(app, code),
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
                app.action_taken = Some("refresh".to_string());
                return true;
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

    fn handle_command_menu(app: &mut AppStateManager, code: KeyCode) -> bool {
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
                return Self::execute_selected_command(app);
            }
            KeyCode::Char(c) => {
                // Atajo rápido por letra
                if let Some(pos) = app
                    .commands
                    .iter()
                    .position(|cmd| cmd.shortcut.map_or(false, |s| s == c.to_ascii_lowercase()))
                {
                    app.selected_command = pos;
                    return Self::execute_selected_command(app);
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
                let theme = app.themes[app.selected_theme];
                app.action_taken = Some(format!("theme:{}", theme));
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_confirm(app: &mut AppStateManager, code: KeyCode, action: &str) -> bool {
        match code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.action_taken = Some(action.to_string());
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

    fn execute_selected_command(app: &mut AppStateManager) -> bool {
        match app.get_selected_command_id() {
            "refresh" => app.state = AppState::ConfirmRefresh,
            "theme" => app.state = AppState::ThemeSelector,
            "reconfigure" => app.state = AppState::ConfirmReconfigure,
            "cache" => {
                app.action_taken = Some("cache".to_string());
                return true;
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
}
