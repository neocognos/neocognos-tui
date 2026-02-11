//! Slash command handling.

use crossterm::style::{self, Stylize};
use crate::ui::theme;

/// Result of processing a slash command.
pub enum CommandResult {
    /// Not a command, treat as regular input.
    NotACommand,
    /// Command handled, continue the REPL.
    Continue,
    /// Quit the application.
    Quit,
    /// Switch model.
    SwitchModel(String),
    /// Clear the screen.
    Clear,
    /// Execute shell command.
    ShellCommand(String),
    /// Compact conversation history.
    Compact,
}

/// Process a potential slash command or shell command. Returns CommandResult.
pub fn process_command(input: &str) -> CommandResult {
    let trimmed = input.trim();
    
    // Handle ! prefix for direct shell commands
    if trimmed.starts_with('!') {
        let command = trimmed[1..].trim();
        if command.is_empty() {
            println!("Usage: !<command>");
            return CommandResult::Continue;
        }
        return CommandResult::ShellCommand(command.to_string());
    }
    
    // Handle / prefix for TUI commands
    if !trimmed.starts_with('/') {
        return CommandResult::NotACommand;
    }

    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd {
        "/quit" | "/exit" | "/q" => CommandResult::Quit,
        "/clear" => CommandResult::Clear,
        "/model" => {
            if arg.is_empty() {
                println!("Usage: /model <provider:model>");
                CommandResult::Continue
            } else {
                CommandResult::SwitchModel(arg.to_string())
            }
        }
        "/help" | "/?" => {
            print_help();
            CommandResult::Continue
        }
        "/compact" => CommandResult::Compact,
        _ => {
            println!("Unknown command: {cmd}. Type /help for available commands.");
            CommandResult::Continue
        }
    }
}

fn print_help() {
    println!();
    println!("{}", style::style("Available Commands").with(theme::ACCENT_COLOR).bold());
    println!("  {}  — Exit the application", style::style("/quit").with(theme::USER_COLOR));
    println!("  {}  — Clear the screen", style::style("/clear").with(theme::USER_COLOR));
    println!("  {} <model> — Switch LLM model", style::style("/model").with(theme::USER_COLOR));
    println!("  {} — Compact conversation history", style::style("/compact").with(theme::USER_COLOR));
    println!("  {}  — Show this help", style::style("/help").with(theme::USER_COLOR));
    println!();
    println!("{}", style::style("Shell Commands").with(theme::ACCENT_COLOR).bold());
    println!("  {} — Execute shell command directly", style::style("!<command>").with(theme::USER_COLOR));
    println!("  Example: {} or {}", 
        style::style("!ls -la").with(theme::DIM_COLOR),
        style::style("!pwd").with(theme::DIM_COLOR));
    println!();
    println!("{}", style::style("Keyboard Shortcuts").with(theme::ACCENT_COLOR).bold());
    println!("  {} — Clear the screen", style::style("Ctrl+L").with(theme::USER_COLOR));
    println!("  {} — Exit", style::style("Ctrl+D").with(theme::USER_COLOR));
    println!("  {} — Cancel current line", style::style("Ctrl+C").with(theme::USER_COLOR));
    println!();
    println!("{}", style::style("Input").with(theme::ACCENT_COLOR).bold());
    println!("  Use \\ at end of line for multi-line input");
    println!();
}
