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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_command() {
        assert!(matches!(process_command("/quit"), CommandResult::Quit));
        assert!(matches!(process_command("/q"), CommandResult::Quit));
        assert!(matches!(process_command("/exit"), CommandResult::Quit));
    }

    #[test]
    fn test_help_command() {
        assert!(matches!(process_command("/help"), CommandResult::Continue));
        assert!(matches!(process_command("/?"), CommandResult::Continue));
    }

    #[test]
    fn test_clear_command() {
        assert!(matches!(process_command("/clear"), CommandResult::Clear));
    }

    #[test]
    fn test_compact_command() {
        assert!(matches!(process_command("/compact"), CommandResult::Compact));
    }

    #[test]
    fn test_cost_command() {
        assert!(matches!(process_command("/cost"), CommandResult::Cost));
    }

    #[test]
    fn test_model_command() {
        match process_command("/model sonnet") {
            CommandResult::SwitchModel(m) => assert_eq!(m, "sonnet"),
            _ => panic!("expected SwitchModel"),
        }
        // No arg returns Continue
        assert!(matches!(process_command("/model"), CommandResult::Continue));
    }

    #[test]
    fn test_shell_command() {
        match process_command("!ls -la") {
            CommandResult::ShellCommand(c) => assert_eq!(c, "ls -la"),
            _ => panic!("expected ShellCommand"),
        }
        // Empty shell command
        assert!(matches!(process_command("!"), CommandResult::Continue));
    }

    #[test]
    fn test_not_a_command() {
        assert!(matches!(process_command("hello"), CommandResult::NotACommand));
    }

    #[test]
    fn test_unknown_slash() {
        assert!(matches!(process_command("/unknown"), CommandResult::Continue));
    }
}
