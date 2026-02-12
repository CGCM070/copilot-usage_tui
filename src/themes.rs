use crate::models::Theme;
use ratatui::style::Color;

pub struct ThemeColors {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
    pub border: Color,
    pub highlight: Color,
    pub bar_empty: Color,
    pub bar_filled: Color,
}

impl ThemeColors {
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self::dark(),
            Theme::Light => Self::light(),
            Theme::Dracula => Self::dracula(),
            Theme::Nord => Self::nord(),
            Theme::Monokai => Self::monokai(),
            Theme::Gruvbox => Self::gruvbox(),
        }
    }

    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(30, 30, 30),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(98, 114, 164),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 184, 108),
            error: Color::Rgb(255, 85, 85),
            muted: Color::Rgb(98, 114, 164),
            border: Color::Rgb(68, 71, 90),
            highlight: Color::Rgb(68, 71, 90),
            bar_empty: Color::Rgb(40, 42, 54),
            bar_filled: Color::Rgb(80, 250, 123),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::Rgb(250, 250, 250),
            foreground: Color::Rgb(60, 60, 60),
            accent: Color::Rgb(120, 120, 120),
            success: Color::Rgb(34, 139, 34),
            warning: Color::Rgb(255, 140, 0),
            error: Color::Rgb(220, 20, 60),
            muted: Color::Rgb(128, 128, 128),
            border: Color::Rgb(200, 200, 200),
            highlight: Color::Rgb(230, 230, 230),
            bar_empty: Color::Rgb(220, 220, 220),
            bar_filled: Color::Rgb(34, 139, 34),
        }
    }

    pub fn dracula() -> Self {
        Self {
            background: Color::Rgb(40, 42, 54),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(98, 114, 164),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 184, 108),
            error: Color::Rgb(255, 85, 85),
            muted: Color::Rgb(98, 114, 164),
            border: Color::Rgb(68, 71, 90),
            highlight: Color::Rgb(68, 71, 90),
            bar_empty: Color::Rgb(68, 71, 90),
            bar_filled: Color::Rgb(189, 147, 249),
        }
    }

    pub fn nord() -> Self {
        Self {
            background: Color::Rgb(46, 52, 64),
            foreground: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(136, 192, 208),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            muted: Color::Rgb(76, 86, 106),
            border: Color::Rgb(76, 86, 106),
            highlight: Color::Rgb(76, 86, 106),
            bar_empty: Color::Rgb(59, 66, 82),
            bar_filled: Color::Rgb(136, 192, 208),
        }
    }

    pub fn monokai() -> Self {
        Self {
            background: Color::Rgb(39, 40, 34),
            foreground: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(117, 113, 94),
            success: Color::Rgb(166, 226, 46),
            warning: Color::Rgb(253, 151, 31),
            error: Color::Rgb(249, 38, 114),
            muted: Color::Rgb(117, 113, 94),
            border: Color::Rgb(73, 72, 62),
            highlight: Color::Rgb(73, 72, 62),
            bar_empty: Color::Rgb(73, 72, 62),
            bar_filled: Color::Rgb(166, 226, 46),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            background: Color::Rgb(40, 40, 40),
            foreground: Color::Rgb(235, 219, 178),
            accent: Color::Rgb(146, 131, 116),
            success: Color::Rgb(184, 187, 38),
            warning: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            muted: Color::Rgb(146, 131, 116),
            border: Color::Rgb(102, 92, 84),
            highlight: Color::Rgb(102, 92, 84),
            bar_empty: Color::Rgb(60, 56, 54),
            bar_filled: Color::Rgb(184, 187, 38),
        }
    }
}
