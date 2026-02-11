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

/// Render the workflow trace (lower sidebar).
pub fn render_trace(frame: &mut Frame, area: Rect, app: &App) {
    use crate::app::TraceEntry;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(" Trace ", theme::accent_style()));

    let mut lines: Vec<Line> = Vec::new();

    if app.trace_log.is_empty() {
        lines.push(Line::from(Span::styled(" Waiting...", theme::dim_style())));
    } else {
        for entry in &app.trace_log {
            match entry {
                TraceEntry::StageStart { id, kind } => {
                    lines.push(Line::from(vec![
                        Span::styled(" ▶ ", Style::default().fg(Color::Cyan)),
                        Span::styled(id, theme::dim_style()),
                        Span::styled(format!(" ({})", kind), Style::default().fg(Color::DarkGray)),
                    ]));
                }
                TraceEntry::StageEnd { id: _, duration_ms, skipped } => {
                    if *skipped {
                        lines.push(Line::from(Span::styled("   ⏭ skipped", Style::default().fg(Color::Yellow))));
                    } else if *duration_ms > 100 {
                        lines.push(Line::from(Span::styled(
                            format!("   ✓ {}ms", duration_ms),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                    // Don't show completion for fast stages (< 100ms) to reduce noise
                }
                TraceEntry::LlmCall { model, ctx_tokens, out_tokens, duration_ms } => {
                    let ctx_k = (*ctx_tokens as f64 / 1000.0).round() as usize;
                    let dur = if *duration_ms >= 1000 {
                        format!("{:.1}s", *duration_ms as f64 / 1000.0)
                    } else {
                        format!("{}ms", duration_ms)
                    };
                    let model_short = if model.len() > 10 { &model[..10] } else { model.as_str() };
                    lines.push(Line::from(vec![
                        Span::styled("   🧠 ", Style::default()),
                        Span::styled(model_short, theme::user_style()),
                        Span::styled(format!(" {}k→{} {}", ctx_k, out_tokens, dur), theme::dim_style()),
                    ]));
                }
                TraceEntry::ToolCall { name, args } => {
                    let args_short = if args.len() > 20 {
                        format!("{}...", &args[..17])
                    } else {
                        args.clone()
                    };
                    lines.push(Line::from(vec![
                        Span::styled("   ⚡ ", Style::default().fg(Color::Yellow)),
                        Span::styled(name, Style::default().fg(Color::Yellow)),
                        Span::styled(format!(" {}", args_short), theme::dim_style()),
                    ]));
                }
                TraceEntry::ToolResult { name: _, success, duration_ms } => {
                    let (icon, color) = if *success {
                        ("✓", Color::Green)
                    } else {
                        ("✗", Color::Red)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("   {} {}ms", icon, duration_ms),
                        Style::default().fg(color),
                    )));
                }
                TraceEntry::Narration(text) => {
                    let short = if text.len() > 25 {
                        format!("{}...", &text[..22])
                    } else {
                        text.clone()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("   💬 {}", short),
                        theme::dim_style(),
                    )));
                }
            }
        }
    }

    // Auto-scroll: only show the last N lines that fit
    let max_visible = (area.height as usize).saturating_sub(2);
    let total = lines.len();
    if total > max_visible {
        lines = lines.into_iter().skip(total - max_visible).collect();
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
