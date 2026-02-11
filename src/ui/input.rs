//! Input bar widget and key handling.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Span;

use crate::app::App;
use super::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let prompt_prefix = format!("{} ({}) > ", app.status.agent_name, app.status.model);
    let display_text = format!("{}{}", prompt_prefix, app.input);

    let paragraph = Paragraph::new(Span::raw(&display_text))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style()));

    frame.render_widget(paragraph, area);

    // Place cursor
    let cursor_x = area.x + 1 + prompt_prefix.len() as u16 + app.cursor_pos as u16;
    let cursor_y = area.y + 1;
    if cursor_x < area.x + area.width - 1 {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
