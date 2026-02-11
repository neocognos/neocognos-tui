//! Integration tests for App + command flow.

use neocognos_tui::app::{App, ChatMessage, PanelFocus, TraceEntry};
use neocognos_tui::commands::{process_command, CommandResult};

#[test]
fn test_clear_resets_messages() {
    let mut app = App::new("agent", "model", "workflow");
    app.add_message(ChatMessage::User("hello".into()));
    app.add_message(ChatMessage::Assistant("hi".into()));
    assert_eq!(app.messages.len(), 2);

    // Simulate /clear
    let result = process_command("/clear");
    assert!(matches!(result, CommandResult::Clear));
    app.clear_messages();
    assert!(app.messages.is_empty());
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_trace_log_accumulation() {
    let mut app = App::new("agent", "model", "workflow");
    app.trace_log.push(TraceEntry::StageStart { id: "s1".into(), kind: "plan".into() });
    app.trace_log.push(TraceEntry::LlmCall {
        model: "sonnet".into(), ctx_tokens: 100, out_tokens: 50, duration_ms: 200,
    });
    app.trace_log.push(TraceEntry::StageEnd { id: "s1".into(), duration_ms: 300, skipped: false });
    assert_eq!(app.trace_log.len(), 3);
}

#[test]
fn test_submit_and_command_flow() {
    let mut app = App::new("agent", "model", "workflow");
    app.input = "/quit".into();
    app.cursor_pos = 5;
    let text = app.submit_input().unwrap();
    assert_eq!(text, "/quit");
    assert!(app.input.is_empty());
    assert_eq!(app.input_history.len(), 1);

    let result = process_command(&text);
    assert!(matches!(result, CommandResult::Quit));
}

#[test]
fn test_scroll_with_focus() {
    let mut app = App::new("agent", "model", "workflow");
    // Chat focus - scroll_offset controls chat
    assert_eq!(app.focus, PanelFocus::Chat);
    app.scroll_offset = 5;
    assert_eq!(app.scroll_offset, 5);

    // Trace focus - trace_scroll controls trace
    app.focus = PanelFocus::Trace;
    assert!(app.trace_scroll.is_none()); // auto-follow
    app.trace_scroll = Some(3);
    assert_eq!(app.trace_scroll, Some(3));
}

#[test]
fn test_full_input_submit_cycle() {
    let mut app = App::new("agent", "model", "workflow");
    // Type "hello"
    for c in "hello".chars() {
        app.insert_char(c);
    }
    assert_eq!(app.input, "hello");

    // Submit
    let text = app.submit_input().unwrap();
    assert_eq!(text, "hello");
    assert!(matches!(process_command(&text), CommandResult::NotACommand));

    // Add as user message
    app.add_message(ChatMessage::User(text));
    assert_eq!(app.messages.len(), 1);

    // History recall
    app.history_up();
    assert_eq!(app.input, "hello");
}
