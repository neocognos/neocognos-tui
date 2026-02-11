//! Color theme and styling constants.

use crossterm::style::Color;

pub const USER_COLOR: Color = Color::Rgb { r: 100, g: 149, b: 237 }; // Cornflower blue
pub const ASSISTANT_COLOR: Color = Color::Rgb { r: 120, g: 200, b: 120 }; // Green
pub const TOOL_COLOR: Color = Color::Rgb { r: 230, g: 190, b: 60 }; // Yellow
pub const ERROR_COLOR: Color = Color::Rgb { r: 220, g: 80, b: 80 }; // Red
pub const DIM_COLOR: Color = Color::DarkGrey;
pub const ACCENT_COLOR: Color = Color::Rgb { r: 160, g: 120, b: 230 }; // Purple

pub const TOOL_BORDER_TOP: &str = "⚡ ";
pub const TOOL_BORDER_LINE: &str = "│ ";
pub const TOOL_BORDER_RUNNING: &str = "├─ ";
pub const TOOL_BORDER_DONE: &str = "└─ ";
