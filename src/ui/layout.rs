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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_dimensions() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = compute_layout(area);

        // Input bar should be 3 lines tall at the bottom
        assert_eq!(layout.input.height, 3);
        assert_eq!(layout.input.y, 37);
        assert_eq!(layout.input.width, 120);

        // Chat should take ~75% width
        assert!(layout.chat.width >= 85 && layout.chat.width <= 95);
        // Chat height = total - input
        assert_eq!(layout.chat.height, 37);

        // Sidebar should take ~25%
        let sidebar_w = layout.sidebar_status.width;
        assert!(sidebar_w >= 25 && sidebar_w <= 35);
    }

    #[test]
    fn test_sidebar_split() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = compute_layout(area);

        // Status is top part, llm_log is bottom part of sidebar
        assert!(layout.sidebar_status.y < layout.sidebar_llm_log.y);
        // Combined heights should equal the main area height (37)
        let combined = layout.sidebar_status.height + layout.sidebar_llm_log.height;
        assert_eq!(combined, 37);
        // Status ~40%, log ~60%
        assert!(layout.sidebar_status.height < layout.sidebar_llm_log.height);
    }
}
