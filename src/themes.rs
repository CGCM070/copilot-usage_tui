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
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 184, 108),
            error: Color::Rgb(255, 85, 85),
            muted: Color::Rgb(98, 114, 164),
            border: Color::Rgb(68, 71, 90),
            bar_empty: Color::Rgb(40, 42, 54),
        }
    }

    pub fn nord() -> Self {
        Self {
            foreground: Color::Rgb(216, 222, 233),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            muted: Color::Rgb(76, 86, 106),
            border: Color::Rgb(76, 86, 106),
            bar_empty: Color::Rgb(59, 66, 82),
        }
    }

    pub fn monokai() -> Self {
        Self {
            foreground: Color::Rgb(248, 248, 242),
            success: Color::Rgb(166, 226, 46),
            warning: Color::Rgb(253, 151, 31),
            error: Color::Rgb(249, 38, 114),
            muted: Color::Rgb(117, 113, 94),
            border: Color::Rgb(73, 72, 62),
            bar_empty: Color::Rgb(73, 72, 62),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            foreground: Color::Rgb(235, 219, 178),
            success: Color::Rgb(184, 187, 38),
            warning: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            muted: Color::Rgb(146, 131, 116),
            border: Color::Rgb(102, 92, 84),
            bar_empty: Color::Rgb(60, 56, 54),
        }
    }

    // Catppuccin Mocha - Soft pastel colors
    pub fn catppuccin() -> Self {
        Self {
            foreground: Color::Rgb(205, 214, 244), // text
            success: Color::Rgb(166, 227, 161),    // green
            warning: Color::Rgb(249, 226, 175),    // yellow
            error: Color::Rgb(243, 139, 168),      // red
            muted: Color::Rgb(147, 153, 178),      // overlay1
            border: Color::Rgb(88, 91, 112),       // surface2
            bar_empty: Color::Rgb(49, 50, 68),     // surface0
        }
    }

    // One Dark - Atom/VSCode classic
    pub fn one_dark() -> Self {
        Self {
            foreground: Color::Rgb(171, 178, 191), // fg
            success: Color::Rgb(152, 195, 121),    // green
            warning: Color::Rgb(229, 192, 123),    // yellow
            error: Color::Rgb(224, 108, 117),      // red
            muted: Color::Rgb(92, 99, 112),        // comment
            border: Color::Rgb(62, 68, 81),        // gutter
            bar_empty: Color::Rgb(40, 44, 52),     // bg
        }
    }

    // Tokyo Night - Modern dark theme
    pub fn tokyo_night() -> Self {
        Self {
            foreground: Color::Rgb(169, 177, 214), // fg
            success: Color::Rgb(158, 206, 106),    // green
            warning: Color::Rgb(224, 175, 104),    // yellow
            error: Color::Rgb(247, 118, 142),      // red
            muted: Color::Rgb(86, 95, 137),        // comment
            border: Color::Rgb(61, 89, 161),       // blue accent
            bar_empty: Color::Rgb(36, 40, 59),     // bg dark
        }
    }

    // Solarized Dark - Classic ergonomic theme
    pub fn solarized_dark() -> Self {
        Self {
            foreground: Color::Rgb(131, 148, 150), // base0
            success: Color::Rgb(133, 153, 0),      // green
            warning: Color::Rgb(181, 137, 0),      // yellow
            error: Color::Rgb(220, 50, 47),        // red
            muted: Color::Rgb(88, 110, 117),       // base01
            border: Color::Rgb(73, 80, 87),        // base02
            bar_empty: Color::Rgb(0, 43, 54),      // base03
        }
    }

    // Kanagawa - Japanese ink painting inspired
    pub fn kanagawa() -> Self {
        Self {
            foreground: Color::Rgb(220, 215, 186), // fujiWhite
            success: Color::Rgb(152, 187, 108),    // springGreen
            warning: Color::Rgb(255, 169, 89),     // surimiOrange
            error: Color::Rgb(195, 64, 67),        // autumnRed
            muted: Color::Rgb(114, 113, 105),      // fujiGray
            border: Color::Rgb(84, 84, 109),       // sumiInk4
            bar_empty: Color::Rgb(54, 54, 70),     // sumiInk3
        }
    }
}
