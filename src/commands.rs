//! Slash command handling.

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
}

/// Process a potential slash command. Returns CommandResult.
pub fn process_command(input: &str) -> CommandResult {
    let trimmed = input.trim();
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
        "/compact" => {
            println!("⚠ Compact mode not yet implemented");
            CommandResult::Continue
        }
        _ => {
            println!("Unknown command: {cmd}. Type /help for available commands.");
            CommandResult::Continue
        }
    }
}

fn print_help() {
    use crossterm::style::{self, Stylize};
    use crate::ui::theme;

    println!();
    println!("{}", style::style("Available Commands").with(theme::ACCENT_COLOR).bold());
    println!("  {}  — Exit the application", style::style("/quit").with(theme::USER_COLOR));
    println!("  {}  — Clear the screen", style::style("/clear").with(theme::USER_COLOR));
    println!("  {} <model> — Switch LLM model", style::style("/model").with(theme::USER_COLOR));
    println!("  {} — Compact conversation history", style::style("/compact").with(theme::USER_COLOR));
    println!("  {}  — Show this help", style::style("/help").with(theme::USER_COLOR));
    println!();
    println!("{}", style::style("Input").with(theme::ACCENT_COLOR).bold());
    println!("  Use \\ at end of line for multi-line input");
    println!("  Ctrl+D to exit, Ctrl+C to cancel current line");
    println!();
}
