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

/// Estado completo de la aplicación UI
pub struct AppStateManager {
    pub state: AppState,
    pub selected_command: usize,
    pub selected_theme: usize,
    pub model_scroll_offset: usize,
    pub commands: Vec<Command>,
    pub themes: Vec<&'static str>,
    pub action_taken: Option<String>,
}

impl AppStateManager {
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

    // Navegación en menú de comandos
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

    // Navegación en selector de temas
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

    // Scroll en tabla de modelos
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

    // Utilidades
    pub fn get_selected_command_id(&self) -> &str {
        self.commands[self.selected_command].id
    }

    pub fn reset_scroll(&mut self) {
        self.model_scroll_offset = 0;
    }
}

impl Default for AppStateManager {
    fn default() -> Self {
        Self::new()
    }
}
