use crate::models::Theme;
use ratatui::style::Color;

pub struct ThemeColors {
    pub foreground: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
    pub border: Color,
    pub bar_empty: Color,
}

impl ThemeColors {
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self::dark(),
            Theme::Nord => Self::nord(),
            Theme::Monokai => Self::monokai(),
            Theme::Gruvbox => Self::gruvbox(),
            Theme::Catppuccin => Self::catppuccin(),
            Theme::OneDark => Self::one_dark(),
            Theme::TokyoNight => Self::tokyo_night(),
            Theme::SolarizedDark => Self::solarized_dark(),
            Theme::Kanagawa => Self::kanagawa(),
        }
    }

    pub fn dark() -> Self {
        Self {
            foreground: Color::Rgb(248, 248, 242),
            success: Color::Rgb(100, 255, 140),
            warning: Color::Rgb(255, 200, 120),
            error: Color::Rgb(255, 100, 100),
            muted: Color::Rgb(98, 114, 164),
            border: Color::Rgb(68, 71, 90),
            bar_empty: Color::Rgb(40, 42, 54),
        }
    }

    pub fn nord() -> Self {
        Self {
            foreground: Color::Rgb(216, 222, 233),
            success: Color::Rgb(180, 210, 150),
            warning: Color::Rgb(245, 215, 150),
            error: Color::Rgb(210, 110, 120),
            muted: Color::Rgb(76, 86, 106),
            border: Color::Rgb(76, 86, 106),
            bar_empty: Color::Rgb(59, 66, 82),
        }
    }

    pub fn monokai() -> Self {
        Self {
            foreground: Color::Rgb(248, 248, 242),
            success: Color::Rgb(180, 240, 60),
            warning: Color::Rgb(255, 170, 50),
            error: Color::Rgb(255, 60, 130),
            muted: Color::Rgb(117, 113, 94),
            border: Color::Rgb(73, 72, 62),
            bar_empty: Color::Rgb(73, 72, 62),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            foreground: Color::Rgb(235, 219, 178),
            success: Color::Rgb(200, 205, 55),
            warning: Color::Rgb(255, 205, 60),
            error: Color::Rgb(255, 90, 70),
            muted: Color::Rgb(146, 131, 116),
            border: Color::Rgb(102, 92, 84),
            bar_empty: Color::Rgb(60, 56, 54),
        }
    }

    // Catppuccin Mocha - Soft pastel colors
    pub fn catppuccin() -> Self {
        Self {
            foreground: Color::Rgb(205, 214, 244), // text
            success: Color::Rgb(180, 240, 175),    // green (brighter)
            warning: Color::Rgb(255, 235, 190),    // yellow (brighter)
            error: Color::Rgb(255, 155, 185),      // red (brighter)
            muted: Color::Rgb(147, 153, 178),      // overlay1
            border: Color::Rgb(88, 91, 112),       // surface2
            bar_empty: Color::Rgb(49, 50, 68),     // surface0
        }
    }

    // One Dark - Atom/VSCode classic
    pub fn one_dark() -> Self {
        Self {
            foreground: Color::Rgb(171, 178, 191), // fg
            success: Color::Rgb(165, 210, 135),    // green (brighter)
            warning: Color::Rgb(240, 205, 135),    // yellow (brighter)
            error: Color::Rgb(235, 120, 130),      // red (brighter)
            muted: Color::Rgb(92, 99, 112),        // comment
            border: Color::Rgb(62, 68, 81),        // gutter
            bar_empty: Color::Rgb(40, 44, 52),     // bg
        }
    }

    // Tokyo Night - Modern dark theme
    pub fn tokyo_night() -> Self {
        Self {
            foreground: Color::Rgb(169, 177, 214), // fg
            success: Color::Rgb(170, 220, 115),    // green (brighter)
            warning: Color::Rgb(235, 185, 115),    // yellow (brighter)
            error: Color::Rgb(255, 130, 155),      // red (brighter)
            muted: Color::Rgb(86, 95, 137),        // comment
            border: Color::Rgb(61, 89, 161),       // blue accent
            bar_empty: Color::Rgb(36, 40, 59),     // bg dark
        }
    }

    // Solarized Dark - Classic ergonomic theme
    pub fn solarized_dark() -> Self {
        Self {
            foreground: Color::Rgb(131, 148, 150), // base0
            success: Color::Rgb(150, 175, 20),     // green (brighter)
            warning: Color::Rgb(200, 155, 20),     // yellow (brighter)
            error: Color::Rgb(235, 70, 65),        // red (brighter)
            muted: Color::Rgb(88, 110, 117),       // base01
            border: Color::Rgb(73, 80, 87),        // base02
            bar_empty: Color::Rgb(0, 43, 54),      // base03
        }
    }

    // Kanagawa - Japanese ink painting inspired
    pub fn kanagawa() -> Self {
        Self {
            foreground: Color::Rgb(220, 215, 186), // fujiWhite
            success: Color::Rgb(165, 205, 120),    // springGreen (brighter)
            warning: Color::Rgb(255, 185, 105),    // surimiOrange (brighter)
            error: Color::Rgb(215, 80, 85),        // autumnRed (brighter)
            muted: Color::Rgb(114, 113, 105),      // fujiGray
            border: Color::Rgb(84, 84, 109),       // sumiInk4
            bar_empty: Color::Rgb(54, 54, 70),     // sumiInk3
        }
    }
}
