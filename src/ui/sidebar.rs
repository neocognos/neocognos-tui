//! Right sidebar — status, recent files, recent tools.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use crate::app::App;
use super::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(" Info ", theme::accent_style()));

    let mut lines: Vec<Line> = Vec::new();

    // Status section
    lines.push(Line::from(Span::styled("📊 Status", theme::accent_style())));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Model: ", theme::dim_style()),
        Span::styled(&app.status.model, theme::user_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Tokens: ", theme::dim_style()),
        Span::raw(app.status.tokens_display()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Turns: ", theme::dim_style()),
        Span::raw(format!("{}", app.status.total_turns)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" Cost: ", theme::dim_style()),
        Span::raw(app.status.cost_display()),
    ]));

    // Recent files section
    if !app.recent_files.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("📁 Recent Files", theme::accent_style())));
        lines.push(Line::from(""));
        for f in app.recent_files.iter().take(8) {
            // Show just filename, not full path
            let display = f.rsplit('/').next().unwrap_or(f);
            lines.push(Line::from(Span::styled(
                format!(" {display}"),
                theme::dim_style(),
            )));
        }
    }

    // Recent tools section
    if !app.recent_tools.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("🔧 Last Tools", theme::accent_style())));
        lines.push(Line::from(""));
        for t in app.recent_tools.iter().take(6) {
            let icon = if t.success { "✓" } else { "✗" };
            let style = if t.success { theme::success_style() } else { theme::error_style() };
            lines.push(Line::from(vec![
                Span::styled(format!(" {icon} "), style),
                Span::styled(&t.name, theme::dim_style()),
            ]));
        }
    }

    // Busy indicator
    if app.agent_busy {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(" ⏳ Working...", theme::tool_style())));
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
