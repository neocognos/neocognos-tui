//! Chat area widget — renders scrollable message list.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::text::{Line, Span};

use crate::app::{App, ChatMessage};
use super::theme;

/// Render the chat area.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(" Chat ", theme::accent_style()));

    let inner = block.inner(area);

    // Build lines from messages
    let mut lines: Vec<Line> = Vec::new();

    if app.messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Type a message to begin...",
            theme::dim_style(),
        )));
    }

    for msg in &app.messages {
        match msg {
            ChatMessage::User(text) => {
                lines.push(Line::from(vec![
                    Span::styled("> ", theme::user_style()),
                    Span::styled(text.as_str(), theme::user_style()),
                ]));
            }
            ChatMessage::Assistant(text) => {
                // Split into lines for multi-line responses
                for line in text.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {line}"),
                        theme::assistant_style(),
                    )));
                }
            }
            ChatMessage::Narration(text) => {
                lines.push(Line::from(vec![
                    Span::styled("  💬 ", Style::default()),
                    Span::styled(text.as_str(), theme::narration_style()),
                ]));
            }
            ChatMessage::ToolCall { name, args_short } => {
                lines.push(Line::from(vec![
                    Span::styled("  ⚡ ", Style::default()),
                    Span::styled(name.as_str(), theme::tool_style()),
                    Span::raw(" "),
                    Span::styled(args_short.as_str(), theme::dim_style()),
                ]));
            }
            ChatMessage::ToolResult { name, success, duration_ms } => {
                let icon = if *success { "  ✓ " } else { "  ✗ " };
                let style = if *success { theme::success_style() } else { theme::error_style() };
                lines.push(Line::from(vec![
                    Span::styled(icon, style),
                    Span::styled(name.as_str(), theme::dim_style()),
                    Span::raw(" "),
                    Span::styled(format!("{duration_ms}ms"), theme::dim_style()),
                ]));
            }
            ChatMessage::Error(text) => {
                lines.push(Line::from(vec![
                    Span::styled("  ✗ ", theme::error_style()),
                    Span::styled(text.as_str(), theme::error_style()),
                ]));
            }
            ChatMessage::System(text) => {
                lines.push(Line::from(Span::styled(
                    format!("  {text}"),
                    theme::system_style(),
                )));
            }
        }
        // Add blank line between messages for readability
        lines.push(Line::from(""));
    }

    // Show thinking indicator
    if app.agent_busy {
        let dots = if let Some(since) = app.thinking_since {
            let elapsed = since.elapsed().as_secs();
            let dot_count = (elapsed % 4) as usize;
            ".".repeat(dot_count + 1)
        } else {
            "...".to_string()
        };
        lines.push(Line::from(Span::styled(
            format!("  🧠 Thinking{dots}"),
            theme::dim_style(),
        )));
    }

    let total_lines = lines.len();
    let visible_height = inner.height as usize;

    // Calculate scroll: auto-scroll if at bottom
    let scroll = if app.scroll_offset == usize::MAX || app.scroll_offset + visible_height >= total_lines {
        total_lines.saturating_sub(visible_height)
    } else {
        app.scroll_offset
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));

    frame.render_widget(paragraph, area);
}
