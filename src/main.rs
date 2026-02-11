//! Neocognos TUI — Rich terminal interface for the Neocognos agent kernel.
//! Ratatui-based split-pane layout with thread-based architecture.

mod agent_thread;
mod app;
mod commands;
mod session;
mod ui;

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::prelude::*;
use ratatui::backend::CrosstermBackend;

use agent_thread::AgentEvent;
use app::{App, ChatMessage};
use session::SessionConfig;

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if has_flag(&args, "--help") || has_flag(&args, "-h") {
        println!("neocognos-tui — Rich terminal interface for Neocognos agents");
        println!();
        println!("USAGE:");
        println!("  neocognos-tui [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("  --manifest <path>     Agent manifest YAML file");
        println!("  --model <model>       LLM model (e.g. anthropic:claude-sonnet-4-20250514)");
        println!("  --provider <name>     LLM provider (anthropic, ollama, claude-cli)");
        println!("  --api-key <key>       API key for the provider");
        println!("  --ollama-url <url>    Ollama base URL (default: http://localhost:11434)");
        println!("  --workflow <path>     Custom workflow YAML file");
        println!("  --autonomy <level>    Autonomy level (manual, supervised, semi, full)");
        println!("  --mock                Use mock LLM for testing");
        println!("  --verbose             Enable verbose event logging");
        println!("  --checkpoint-dir <d>  Enable checkpointing");
        println!("  --event-log <path>    Write events to JSONL file");
        println!("  --trace <path>        Write trace to file");
        println!("  -h, --help            Show this help");
        return Ok(());
    }

    let config = SessionConfig {
        manifest_path: get_arg(&args, "--manifest"),
        model: get_arg(&args, "--model"),
        provider: get_arg(&args, "--provider"),
        api_key: get_arg(&args, "--api-key"),
        ollama_url: get_arg(&args, "--ollama-url")
            .unwrap_or_else(|| "http://localhost:11434".to_string()),
        use_mock: has_flag(&args, "--mock"),
        verbose: has_flag(&args, "--verbose"),
        workflow: get_arg(&args, "--workflow"),
        autonomy_override: get_arg(&args, "--autonomy"),
        checkpoint_dir: get_arg(&args, "--checkpoint-dir"),
        event_log_path: get_arg(&args, "--event-log"),
        trace_path: get_arg(&args, "--trace"),
    };

    // Create event channel
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>();

    // Create session (before entering raw mode, so errors print normally)
    let session = session::Session::from_config(config, event_tx.clone())?;

    let agent_name = session.agent_name.clone();
    let model_name = session.model_name.clone();
    let workflow_name = session.workflow_name.clone();

    // Spawn agent thread
    let input_tx = agent_thread::spawn(session, event_tx);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(&agent_name, &model_name, &workflow_name);
    app.add_message(ChatMessage::System(format!(
        "🧬 Neocognos TUI — Agent: {} | Model: {} | Workflow: {}",
        agent_name, model_name, workflow_name
    )));
    app.add_message(ChatMessage::System(
        "Type /help for commands, /quit to exit".into()
    ));

    // Main event loop
    let tick_rate = Duration::from_millis(100);

    loop {
        // Draw
        terminal.draw(|frame| {
            let layout = ui::layout::compute_layout(frame.area());
            ui::chat::render(frame, layout.chat, &app);
            ui::sidebar::render_status(frame, layout.sidebar_status, &app);
            ui::sidebar::render_llm_log(frame, layout.sidebar_llm_log, &app);
            ui::input::render(frame, layout.input, &app);
        })?;

        // Process agent events (non-blocking)
        while let Ok(evt) = event_rx.try_recv() {
            match evt {
                AgentEvent::Narration(text) => {
                    app.add_message(ChatMessage::Narration(text));
                }
                AgentEvent::ToolCallStarted { name, args } => {
                    app.add_message(ChatMessage::ToolCall {
                        name: name.clone(),
                        args_short: args,
                    });
                    // Extract file path from tool args for sidebar
                    if name == "read_file" || name == "write_file" {
                        // Try to extract path from the args string
                        if let Some(path) = extract_file_path(&app.messages.last()) {
                            app.add_recent_file(path);
                        }
                    }
                }
                AgentEvent::LlmCall { model, prompt_tokens, completion_tokens, duration_ms } => {
                    app.llm_calls.push(app::LlmCallEntry {
                        model,
                        prompt_tokens,
                        completion_tokens,
                        duration_ms,
                    });
                }
                AgentEvent::ToolCallCompleted { name, success, duration_ms } => {
                    app.add_message(ChatMessage::ToolResult {
                        name: name.clone(),
                        success,
                        duration_ms,
                    });
                    app.add_recent_tool(name, success);
                }
                AgentEvent::Response(text) => {
                    app.add_message(ChatMessage::Assistant(text));
                }
                AgentEvent::TokenUpdate { total, turns, cost } => {
                    app.status.total_tokens = total;
                    app.status.total_turns = turns;
                    app.status.cost = cost;
                }
                AgentEvent::Error(text) => {
                    app.add_message(ChatMessage::Error(text));
                }
                AgentEvent::SystemMessage(text) => {
                    if text == "__clear__" {
                        app.clear_messages();
                    } else {
                        app.add_message(ChatMessage::System(text));
                    }
                }
                AgentEvent::Done => {
                    app.agent_busy = false;
                    app.thinking_since = None;
                }
                AgentEvent::Quit => {
                    app.should_quit = true;
                }
            }
        }

        if app.should_quit {
            break;
        }

        // Handle terminal input events
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, key, &input_tx);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("Goodbye! 👋");
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent, input_tx: &mpsc::Sender<String>) {
    match (key.modifiers, key.code) {
        // Ctrl+C: quit if idle, ignore if busy (agent thread handles cancellation)
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            if !app.agent_busy {
                app.should_quit = true;
            }
        }
        // Ctrl+D: quit
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
            app.should_quit = true;
        }
        // Ctrl+L: clear chat
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            app.clear_messages();
        }
        // Enter: submit input
        (_, KeyCode::Enter) => {
            if app.agent_busy {
                return;
            }
            if let Some(text) = app.submit_input() {
                app.add_message(ChatMessage::User(text.clone()));
                app.agent_busy = true;
                app.thinking_since = Some(Instant::now());
                let _ = input_tx.send(text);
            }
        }
        // Backspace
        (_, KeyCode::Backspace) => {
            app.delete_char_before();
        }
        // Delete
        (_, KeyCode::Delete) => {
            app.delete_char_after();
        }
        // Arrow keys
        (_, KeyCode::Left) => app.move_cursor_left(),
        (_, KeyCode::Right) => app.move_cursor_right(),
        (_, KeyCode::Up) => app.history_up(),
        (_, KeyCode::Down) => app.history_down(),
        (_, KeyCode::Home) => app.move_cursor_home(),
        (_, KeyCode::End) => app.move_cursor_end(),
        // Page Up/Down for scrolling
        (_, KeyCode::PageUp) => {
            if app.scroll_offset == usize::MAX {
                // Calculate current position first
                let total = app.messages.len();
                app.scroll_offset = total.saturating_sub(10);
            }
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        (_, KeyCode::PageDown) => {
            app.scroll_offset = if app.scroll_offset == usize::MAX {
                usize::MAX
            } else {
                app.scroll_offset + 10
            };
        }
        // Regular character input
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
            app.insert_char(c);
        }
        _ => {}
    }
}

/// Try to extract a file path from a tool call message.
fn extract_file_path(msg: &Option<&ChatMessage>) -> Option<String> {
    if let Some(ChatMessage::ToolCall { args_short, .. }) = msg {
        // Simple heuristic: look for path-like strings
        let args = args_short.trim();
        if args.contains('/') || args.contains('.') {
            // Take first token that looks like a path
            for token in args.split_whitespace() {
                let clean = token.trim_matches(|c: char| c == '"' || c == '\'' || c == '{' || c == '}' || c == ',');
                if clean.contains('/') || clean.contains('.') {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}
