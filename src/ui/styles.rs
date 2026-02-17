use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
};

use crate::themes::ThemeColors;

// ============================================================================
// UI Constants
// ============================================================================

/// Standard horizontal margin (in cells)
pub const UI_MARGIN: u16 = 1;

/// Maximum number of cells for progress bars
pub const MAX_BAR_CELLS: usize = 40;

/// Width of each bar cell (char + trailing space)
pub const CELL_WIDTH: usize = 2;

/// Minimum bar width to ensure visibility
pub const MIN_BAR_CELLS: usize = 10;

/// Progress bar characters (Vero-style)
pub const BAR_FILLED: &str = "■ ";
pub const BAR_EMPTY: &str = "· ";

/// Legacy progress bar characters (solid style)
pub const BAR_SOLID_FILLED: &str = "█";
pub const BAR_SOLID_EMPTY: &str = "░";

/// Braille patterns (btop style)
pub const BAR_BRAILLE_FILLED: &str = "⣿";
pub const BAR_BRAILLE_EMPTY: &str = "⠀";

/// Color zone thresholds (percentages)
pub const ZONE_SUCCESS_END: f64 = 75.0;
pub const ZONE_WARNING_END: f64 = 90.0;

/// Constant warning color (orange)
pub const WARNING_COLOR: Color = Color::Rgb(255, 184, 108);

/// Constant error color (red)
pub const ERROR_COLOR: Color = Color::Rgb(255, 85, 85);

// ============================================================================
// Style Builders
// ============================================================================

/// Creates a bold header style with the foreground color
pub fn header_style(colors: &ThemeColors) -> Style {
    Style::default()
        .fg(colors.foreground)
        .add_modifier(Modifier::BOLD)
}

/// Creates a muted/secondary text style
pub fn muted_style(colors: &ThemeColors) -> Style {
    Style::default().fg(colors.muted)
}

/// Creates a success style
#[allow(dead_code)]
pub fn success_style(colors: &ThemeColors) -> Style {
    Style::default().fg(colors.success)
}

/// Creates a bold success style for bars
pub fn success_style_bold(colors: &ThemeColors) -> Style {
    Style::default()
        .fg(colors.success)
        .add_modifier(Modifier::BOLD)
}

/// Creates a style for usage percentage based on the value
#[allow(dead_code)]
pub fn usage_style(percentage: f64, colors: &ThemeColors) -> Style {
    let color = get_usage_color(percentage, colors);
    Style::default().fg(color)
}

/// Creates a warning style (constant orange)
#[allow(dead_code)]
pub fn warning_style() -> Style {
    Style::default().fg(WARNING_COLOR)
}

/// Creates a bold warning style for bars
pub fn warning_style_bold() -> Style {
    Style::default()
        .fg(WARNING_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Creates an error style (constant red)
#[allow(dead_code)]
pub fn error_style() -> Style {
    Style::default().fg(ERROR_COLOR)
}

/// Creates a bold error style for bars
pub fn error_style_bold() -> Style {
    Style::default()
        .fg(ERROR_COLOR)
        .add_modifier(Modifier::BOLD)
}

/// Creates a bordered block style
#[allow(dead_code)]
pub fn border_style(colors: &ThemeColors) -> Style {
    Style::default().fg(colors.border)
}

// ============================================================================
// Color Zone Logic
// ============================================================================

/// Gets the appropriate color based on usage percentage
/// - 0-75%: success color
/// - 75-90%: warning color (constant orange)
/// - 90-100%: error color (constant red)
pub fn get_usage_color(percentage: f64, colors: &ThemeColors) -> Color {
    if percentage >= ZONE_WARNING_END {
        ERROR_COLOR
    } else if percentage >= ZONE_SUCCESS_END {
        WARNING_COLOR
    } else {
        colors.success
    }
}

/// Calculates zone boundaries for segmented progress bars
/// Returns (success_end, warning_end) in cell counts
pub fn calculate_zone_boundaries(total_cells: usize) -> (usize, usize) {
    let success_end = ((ZONE_SUCCESS_END / 100.0) * total_cells as f64) as usize;
    let warning_end = ((ZONE_WARNING_END / 100.0) * total_cells as f64) as usize;
    (success_end, warning_end)
}

/// Determines which zone a percentage falls into
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum UsageZone {
    Success, // 0-75%
    Warning, // 75-90%
    Error,   // 90-100%
}

#[allow(dead_code)]
impl UsageZone {
    pub fn from_percentage(percentage: f64) -> Self {
        if percentage >= ZONE_WARNING_END {
            UsageZone::Error
        } else if percentage >= ZONE_SUCCESS_END {
            UsageZone::Warning
        } else {
            UsageZone::Success
        }
    }

    pub fn to_color(&self, colors: &ThemeColors) -> Color {
        match self {
            UsageZone::Success => colors.success,
            UsageZone::Warning => WARNING_COLOR,
            UsageZone::Error => ERROR_COLOR,
        }
    }
}

// ============================================================================
// Layout Helpers
// ============================================================================

/// Creates a layout with standard horizontal margin applied
pub fn with_horizontal_margin(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .horizontal_margin(UI_MARGIN)
        .split(area)
}

/// Creates a vertical layout with specified constraints and horizontal margin
#[allow(dead_code)]
pub fn vertical_layout_with_margin(area: Rect, constraints: &[Constraint]) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.iter().cloned().collect::<Vec<_>>())
        .horizontal_margin(UI_MARGIN)
        .split(area)
}

/// Standard table constraints for model table
#[allow(dead_code)]
pub fn model_table_constraints() -> [Constraint; 4] {
    [
        Constraint::Percentage(25), // Model
        Constraint::Percentage(58), // Progress
        Constraint::Percentage(10), // Usage
        Constraint::Length(7),      // Count
    ]
}

// ============================================================================
// Format Helpers
// ============================================================================

/// Formats a percentage value (e.g., 51.0 -> "51.0%")
pub fn format_percentage(percentage: f64) -> String {
    format!("{:.1}%", percentage)
}

/// Formats a count value (e.g., 153 -> "  153")
pub fn format_count(count: f64) -> String {
    format!("{:>5}", count as i64)
}

/// Formats usage with limit (e.g., 286, 300 -> "286/300")
#[allow(dead_code)]
pub fn format_usage_with_limit(used: f64, limit: f64) -> String {
    format!("{:.0}/{:.0}", used, limit)
}

/// Calculates responsive bar width based on available space
/// Returns the number of cells that fit, respecting MAX_BAR_CELLS and MIN_BAR_CELLS
pub fn calculate_responsive_bar_width(available_width: u16) -> usize {
    let max_chars = (available_width as usize / CELL_WIDTH)
        .min(MAX_BAR_CELLS)
        .max(MIN_BAR_CELLS);
    max_chars
}

/// Calculates filled cells for a progress bar
pub fn calculate_filled_cells(percentage: f64, total_cells: usize) -> usize {
    ((percentage / 100.0) * total_cells as f64) as usize
}
