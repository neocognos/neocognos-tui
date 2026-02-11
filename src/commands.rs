//! Slash command handling.

/// Result of processing a slash command.
pub enum CommandResult {
    NotACommand,
    Continue,
    Quit,
    SwitchModel(String),
    Clear,
    ShellCommand(String),
    Compact,
    Cost,
}

/// Process a potential slash command or shell command.
pub fn process_command(input: &str) -> CommandResult {
    let trimmed = input.trim();

    if trimmed.starts_with('!') {
        let command = trimmed[1..].trim();
        if command.is_empty() {
            return CommandResult::Continue;
        }
        return CommandResult::ShellCommand(command.to_string());
    }

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
                CommandResult::Continue
            } else {
                CommandResult::SwitchModel(arg.to_string())
            }
        }
        "/help" | "/?" => CommandResult::Continue,
        "/compact" => CommandResult::Compact,
        "/cost" => CommandResult::Cost,
        _ => CommandResult::Continue,
    }
}
