//! Input prompt handling with rustyline.

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};
use crossterm::style::{self, Stylize};

use super::theme;

/// The interactive prompt handler.
pub struct InputPrompt {
    editor: DefaultEditor,
    history_path: Option<String>,
    model_name: String,
    agent_name: String,
}

impl InputPrompt {
    pub fn new(agent_name: &str, model_name: &str) -> Self {
        let editor = DefaultEditor::new().expect("Failed to create editor");
        let history_path = dirs_path().map(|p| {
            let path = format!("{}/.neocognos_history", p);
            path
        });

        let mut prompt = Self {
            editor,
            history_path,
            model_name: model_name.to_string(),
            agent_name: agent_name.to_string(),
        };

        // Load history
        if let Some(ref path) = prompt.history_path {
            let _ = prompt.editor.load_history(path);
        }

        prompt
    }

    pub fn set_model(&mut self, model: &str) {
        self.model_name = model.to_string();
    }

    /// Read a line of input. Supports backslash continuation for multi-line.
    /// Returns None on EOF/quit.
    pub fn read_input(&mut self) -> Option<String> {
        let prompt_str = format!("{} ({}) > ",
            style::style(&self.agent_name).with(theme::ACCENT_COLOR),
            style::style(&self.model_name).with(theme::DIM_COLOR));

        let mut full_input = String::new();
        let mut continuation = false;

        loop {
            let p = if continuation { "... " } else { &prompt_str };
            match self.editor.readline(p) {
                Ok(line) => {
                    if line.ends_with('\\') {
                        full_input.push_str(&line[..line.len() - 1]);
                        full_input.push('\n');
                        continuation = true;
                        continue;
                    }
                    full_input.push_str(&line);
                    if !full_input.is_empty() {
                        let _ = self.editor.add_history_entry(&full_input);
                    }
                    return Some(full_input);
                }
                Err(ReadlineError::Interrupted) => {
                    if continuation {
                        full_input.clear();
                        continuation = false;
                        continue;
                    }
                    return Some(String::new()); // Ctrl+C clears line
                }
                Err(ReadlineError::Eof) => {
                    return None; // Ctrl+D exits
                }
                Err(_) => return None,
            }
        }
    }

    /// Save history on exit.
    pub fn save_history(&mut self) {
        if let Some(ref path) = self.history_path {
            let _ = self.editor.save_history(path);
        }
    }
}

fn dirs_path() -> Option<String> {
    std::env::var("HOME").ok()
}
