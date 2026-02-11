//! Input prompt handling with rustyline + file path and command autocomplete.

use std::borrow::Cow;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Config, Context, Editor, Helper, Result as RlResult};
use crossterm::style::{self, Stylize};

use super::theme;

/// Slash commands available for completion.
const SLASH_COMMANDS: &[&str] = &[
    "/quit", "/exit", "/q", "/clear", "/model", "/compact", "/help", "/?",
];

/// Custom helper that provides file path and slash command completion.
struct NeocognosHelper;

impl Helper for NeocognosHelper {}
impl Highlighter for NeocognosHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _default: bool) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }
}
impl Hinter for NeocognosHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}
impl Validator for NeocognosHelper {}

impl Completer for NeocognosHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RlResult<(usize, Vec<Pair>)> {
        let text = &line[..pos];

        // Find the start of the current "word" (token being completed)
        let word_start = text.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
        let word = &text[word_start..];

        // Slash command completion (only at start of line)
        if word_start == 0 && word.starts_with('/') {
            let candidates: Vec<Pair> = SLASH_COMMANDS
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();
            return Ok((word_start, candidates));
        }

        // File path completion
        if word.starts_with('/') || word.starts_with("./") || word.starts_with("../") || word.starts_with('~') {
            let expanded = if word.starts_with('~') {
                if let Ok(home) = std::env::var("HOME") {
                    format!("{}{}", home, &word[1..])
                } else {
                    word.to_string()
                }
            } else {
                word.to_string()
            };

            let (dir_path, prefix) = if expanded.ends_with('/') {
                (expanded.as_str(), "")
            } else {
                match expanded.rfind('/') {
                    Some(idx) => (&expanded[..=idx], &expanded[idx + 1..])  ,
                    None => return Ok((pos, vec![])),
                }
            };

            if let Ok(entries) = std::fs::read_dir(dir_path) {
                let candidates: Vec<Pair> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .map(|n| n.starts_with(prefix))
                            .unwrap_or(false)
                    })
                    .filter_map(|e| {
                        let name = e.file_name().to_str()?.to_string();
                        let is_dir = e.file_type().ok()?.is_dir();
                        let suffix = if is_dir { "/" } else { " " };
                        // Build the full replacement from the original word
                        let base = if word.ends_with('/') {
                            word.to_string()
                        } else {
                            match word.rfind('/') {
                                Some(idx) => word[..=idx].to_string(),
                                None => word.to_string(),
                            }
                        };
                        let replacement = format!("{}{}{}", base, name, suffix);
                        Some(Pair {
                            display: if is_dir { format!("{}/", name) } else { name },
                            replacement,
                        })
                    })
                    .collect();
                return Ok((word_start, candidates));
            }
        }

        Ok((pos, vec![]))
    }
}

/// The interactive prompt handler.
pub struct InputPrompt {
    editor: Editor<NeocognosHelper, rustyline::history::DefaultHistory>,
    history_path: Option<String>,
    model_name: String,
    agent_name: String,
}

impl InputPrompt {
    pub fn new(agent_name: &str, model_name: &str) -> Self {
        let config = Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut editor = Editor::with_config(config).expect("Failed to create editor");
        editor.set_helper(Some(NeocognosHelper));

        let history_path = dirs_path().map(|p| {
            format!("{}/.neocognos_history", p)
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
