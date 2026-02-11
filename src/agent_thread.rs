//! Agent thread — bridges the blocking AgentLoop with the UI event loop via channels.

use std::sync::mpsc;

use crate::session::Session;
use crate::commands::{self, CommandResult};

/// Events sent from the agent thread to the UI.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    Narration(String),
    ToolCallStarted { name: String, args: String },
    ToolCallCompleted { name: String, success: bool, duration_ms: u64 },
    Response(String),
    TokenUpdate { total: usize, turns: usize, cost: f64 },
    Error(String),
    SystemMessage(String),
    Done,
    Quit,
}

/// Spawn the agent thread. Returns a sender for user input.
pub fn spawn(
    session: Session,
    event_tx: mpsc::Sender<AgentEvent>,
) -> mpsc::Sender<String> {
    let (input_tx, input_rx) = mpsc::channel::<String>();

    std::thread::Builder::new()
        .name("agent".into())
        .spawn(move || {
            agent_loop(session, input_rx, event_tx);
        })
        .expect("Failed to spawn agent thread");

    input_tx
}

fn agent_loop(
    mut session: Session,
    input_rx: mpsc::Receiver<String>,
    event_tx: mpsc::Sender<AgentEvent>,
) {
    while let Ok(input) = input_rx.recv() {
        let input = input.trim().to_string();
        if input.is_empty() {
            let _ = event_tx.send(AgentEvent::Done);
            continue;
        }

        // Process slash commands
        match commands::process_command(&input) {
            CommandResult::NotACommand => {}
            CommandResult::Quit => {
                let _ = event_tx.send(AgentEvent::Quit);
                break;
            }
            CommandResult::Continue => {
                // Check if it was /help
                if input.trim().starts_with("/help") || input.trim() == "/?" {
                    let help = "\
Commands: /quit /clear /model <m> /compact /help\n\
Shell: !<command>\n\
Keys: Ctrl+C quit | Ctrl+L clear | PgUp/PgDn scroll | Up/Down history";
                    let _ = event_tx.send(AgentEvent::SystemMessage(help.to_string()));
                }
                let _ = event_tx.send(AgentEvent::Done);
                continue;
            }
            CommandResult::Clear => {
                let _ = event_tx.send(AgentEvent::SystemMessage("__clear__".into()));
                let _ = event_tx.send(AgentEvent::Done);
                continue;
            }
            CommandResult::SwitchModel(model) => {
                let _ = event_tx.send(AgentEvent::SystemMessage(
                    format!("⚠ Model switching not yet implemented. Restart with --model {model}")
                ));
                let _ = event_tx.send(AgentEvent::Done);
                continue;
            }
            CommandResult::Compact => {
                session.compact_with_callback(|msg| {
                    let _ = event_tx.send(AgentEvent::SystemMessage(msg));
                });
                let _ = event_tx.send(AgentEvent::Done);
                continue;
            }
            CommandResult::ShellCommand(cmd) => {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output();
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let combined = if stderr.is_empty() { stdout } else { format!("{stdout}{stderr}") };
                        let _ = event_tx.send(AgentEvent::SystemMessage(combined));
                    }
                    Err(e) => {
                        let _ = event_tx.send(AgentEvent::Error(format!("Shell error: {e}")));
                    }
                }
                let _ = event_tx.send(AgentEvent::Done);
                continue;
            }
        }

        // Run agent turn
        match session.run_turn_with_events(&input, &event_tx) {
            Ok(_) => {
                // Send updated stats
                let stats = &session.stats;
                let _ = event_tx.send(AgentEvent::TokenUpdate {
                    total: stats.total_tokens(),
                    turns: stats.total_turns,
                    cost: stats.estimated_cost(),
                });
            }
            Err(e) => {
                let _ = event_tx.send(AgentEvent::Error(format!("{e}")));
            }
        }
        let _ = event_tx.send(AgentEvent::Done);
    }

    let _ = session.shutdown();
}
