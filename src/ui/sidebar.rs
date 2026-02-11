//! Right sidebar — status panel + LLM call log.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

use crate::app::App;
use super::theme;

/// Render the status panel (upper sidebar).
pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(" Status ", theme::accent_style()));

    let mut lines: Vec<Line> = Vec::new();

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

    // Recent files
    if !app.recent_files.is_empty() {
        lines.push(Line::from(""));
        for f in app.recent_files.iter().rev().take(4) {
            let display = f.rsplit('/').next().unwrap_or(f);
            lines.push(Line::from(Span::styled(format!(" 📄 {display}"), theme::dim_style())));
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

/// Render the LLM call log (lower sidebar).
pub fn render_llm_log(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(" LLM Calls ", theme::accent_style()));

    let mut lines: Vec<Line> = Vec::new();

    if app.llm_calls.is_empty() {
        lines.push(Line::from(Span::styled(" No calls yet", theme::dim_style())));
    } else {
        // Show calls in reverse order (most recent first), with index
        let max_visible = (area.height as usize).saturating_sub(2); // account for borders
        for (i, call) in app.llm_calls.iter().enumerate().rev().take(max_visible) {
            let ctx_k = (call.prompt_tokens as f64 / 1000.0).round() as usize;
            let out_tokens = call.completion_tokens;
            let dur = if call.duration_ms >= 1000 {
                format!("{:.1}s", call.duration_ms as f64 / 1000.0)
            } else {
                format!("{}ms", call.duration_ms)
            };

            let model_short = if call.model.len() > 12 {
                &call.model[..12]
            } else {
                &call.model
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" #{:<2} ", i + 1), theme::dim_style()),
                Span::styled(model_short, theme::user_style()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(format!("{}k in", ctx_k), theme::dim_style()),
                Span::styled(" → ", theme::dim_style()),
                Span::styled(format!("{} out", out_tokens), theme::dim_style()),
                Span::styled(format!(" ({})", dur), theme::dim_style()),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
