//! Markdown rendering for terminal output.

use crossterm::style::{self, Stylize};
use termimad::MadSkin;

use super::theme;

/// Create a configured terminal markdown skin.
pub fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(crossterm::style::Color::White);
    skin.italic.set_fg(crossterm::style::Color::Cyan);
    skin.headers[0].set_fg(theme::ACCENT_COLOR);
    skin.headers[1].set_fg(theme::ACCENT_COLOR);
    skin.code_block.set_fg(crossterm::style::Color::Green);
    skin.inline_code.set_fg(crossterm::style::Color::Yellow);
    skin
}

/// Render markdown text to the terminal.
pub fn render_markdown(text: &str) {
    let skin = make_skin();
    // termimad handles markdown → styled terminal output
    skin.print_text(text);
}

/// Print a streaming token (raw, no markdown processing).
pub fn print_token(token: &str) {
    use std::io::Write;
    print!("{}", style::style(token).with(theme::ASSISTANT_COLOR));
    std::io::stdout().flush().ok();
}

/// Re-render the full response as markdown after streaming completes.
pub fn render_final(text: &str) {
    // Move cursor up and clear streamed output, then render markdown
    // For simplicity in MVP: just print a newline and render markdown below
    // (The streamed raw text stays, markdown re-render is for future enhancement)
    if !text.is_empty() {
        println!();
    }
}
