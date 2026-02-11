//! Color theme and styling constants.

use ratatui::style::{Color, Modifier, Style};

pub const USER_COLOR: Color = Color::Rgb(100, 149, 237);       // Cornflower blue
pub const ASSISTANT_COLOR: Color = Color::Rgb(120, 200, 120);  // Green
pub const TOOL_COLOR: Color = Color::Rgb(230, 190, 60);        // Yellow
pub const ERROR_COLOR: Color = Color::Rgb(220, 80, 80);        // Red
pub const DIM_COLOR: Color = Color::DarkGray;
pub const ACCENT_COLOR: Color = Color::Rgb(160, 120, 230);     // Purple
pub const NARRATION_COLOR: Color = Color::Rgb(180, 180, 180);  // Light grey
pub const SYSTEM_COLOR: Color = Color::Rgb(100, 100, 100);     // Dark grey
pub const SUCCESS_COLOR: Color = Color::Rgb(80, 200, 80);      // Green
pub const BORDER_COLOR: Color = Color::Rgb(60, 60, 80);        // Dim border

pub fn user_style() -> Style {
    Style::default().fg(USER_COLOR)
}

pub fn assistant_style() -> Style {
    Style::default().fg(ASSISTANT_COLOR)
}

pub fn tool_style() -> Style {
    Style::default().fg(TOOL_COLOR)
}

pub fn error_style() -> Style {
    Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD)
}

pub fn dim_style() -> Style {
    Style::default().fg(DIM_COLOR)
}

pub fn accent_style() -> Style {
    Style::default().fg(ACCENT_COLOR).add_modifier(Modifier::BOLD)
}

pub fn narration_style() -> Style {
    Style::default().fg(NARRATION_COLOR)
}

pub fn system_style() -> Style {
    Style::default().fg(SYSTEM_COLOR).add_modifier(Modifier::ITALIC)
}

pub fn success_style() -> Style {
    Style::default().fg(SUCCESS_COLOR)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER_COLOR)
}
