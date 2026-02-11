//! Split-pane layout: chat + sidebar on top, input bar on bottom.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// The three main areas of the UI.
pub struct AppLayout {
    pub chat: Rect,
    pub sidebar: Rect,
    pub input: Rect,
}

pub fn compute_layout(area: Rect) -> AppLayout {
    // Vertical: main area + input bar (3 lines)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Horizontal: chat (75%) + sidebar (25%)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(75),
            Constraint::Percentage(25),
        ])
        .split(vertical[0]);

    AppLayout {
        chat: horizontal[0],
        sidebar: horizontal[1],
        input: vertical[1],
    }
}
