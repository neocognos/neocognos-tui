//! Split-pane layout: chat + sidebar (status + llm log) on top, input bar on bottom.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// The main areas of the UI.
pub struct AppLayout {
    pub chat: Rect,
    pub sidebar_status: Rect,
    pub sidebar_llm_log: Rect,
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

    // Sidebar vertical split: status (top 40%) + LLM log (bottom 60%)
    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(horizontal[1]);

    AppLayout {
        chat: horizontal[0],
        sidebar_status: sidebar[0],
        sidebar_llm_log: sidebar[1],
        input: vertical[1],
    }
}
