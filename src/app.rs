//! Application state.

use std::time::Instant;

/// A single chat message for display.
#[derive(Debug, Clone)]
pub enum ChatMessage {
    User(String),
    Assistant(String),
    Narration(String),
    ToolCall { name: String, args_short: String },
    ToolResult { name: String, success: bool, duration_ms: u64 },
    Error(String),
    System(String),
}

/// Tool status for the sidebar.
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub success: bool,
}

/// LLM call log entry for the sidebar.
#[derive(Debug, Clone)]
pub struct LlmCallEntry {
    pub model: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub duration_ms: u64,
}

/// A trace log entry for the workflow trace panel.
#[derive(Debug, Clone)]
pub enum TraceEntry {
    StageStart { id: String, kind: String },
    StageEnd { id: String, duration_ms: u64, skipped: bool },
    LlmCall { model: String, ctx_tokens: usize, out_tokens: usize, duration_ms: u64 },
    ToolCall { name: String, args: String },
    ToolResult { name: String, success: bool, duration_ms: u64 },
    Narration(String),
}

/// Status info for the sidebar.
#[derive(Debug, Clone, Default)]
pub struct StatusInfo {
    pub model: String,
    pub agent_name: String,
    pub workflow: String,
    pub total_tokens: usize,
    pub total_turns: usize,
    pub cost: f64,
}

impl StatusInfo {
    pub fn tokens_display(&self) -> String {
        if self.total_tokens >= 1000 {
            format!("{:.1}k", self.total_tokens as f64 / 1000.0)
        } else {
            format!("{}", self.total_tokens)
        }
    }

    pub fn cost_display(&self) -> String {
        format!("~${:.4}", self.cost)
    }
}

/// Main application state.
pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub cursor_pos: usize,
    pub scroll_offset: usize,
    pub status: StatusInfo,
    pub recent_files: Vec<String>,
    pub recent_tools: Vec<ToolStatus>,
    pub llm_calls: Vec<LlmCallEntry>,
    pub trace_log: Vec<TraceEntry>,
    pub trace_scroll: Option<usize>,  // None = auto-scroll (follow), Some(n) = pinned at offset n
    pub agent_busy: bool,
    pub should_quit: bool,
    pub input_history: Vec<String>,
    pub history_index: Option<usize>,
    pub thinking_since: Option<Instant>,
}

impl App {
    pub fn new(agent_name: &str, model: &str, workflow: &str) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
            status: StatusInfo {
                model: model.to_string(),
                agent_name: agent_name.to_string(),
                workflow: workflow.to_string(),
                ..Default::default()
            },
            recent_files: Vec::new(),
            recent_tools: Vec::new(),
            llm_calls: Vec::new(),
            trace_log: Vec::new(),
            trace_scroll: None,
            agent_busy: false,
            should_quit: false,
            input_history: Vec::new(),
            history_index: None,
            thinking_since: None,
        }
    }

    pub fn submit_input(&mut self) -> Option<String> {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return None;
        }
        self.input_history.push(text.clone());
        self.history_index = None;
        self.input.clear();
        self.cursor_pos = 0;
        Some(text)
    }

    pub fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            None => self.input_history.len() - 1,
            Some(0) => return,
            Some(i) => i - 1,
        };
        self.history_index = Some(idx);
        self.input = self.input_history[idx].clone();
        self.cursor_pos = self.input.len();
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            None => return,
            Some(i) => {
                if i + 1 >= self.input_history.len() {
                    self.history_index = None;
                    self.input.clear();
                    self.cursor_pos = 0;
                } else {
                    self.history_index = Some(i + 1);
                    self.input = self.input_history[i + 1].clone();
                    self.cursor_pos = self.input.len();
                }
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char_before(&mut self) {
        if self.cursor_pos > 0 {
            // Find the previous character boundary
            let prev = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input.remove(prev);
            self.cursor_pos = prev;
        }
    }

    pub fn delete_char_after(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.input.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.cursor_pos = self.input[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.input.len());
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.input.len();
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        // Auto-scroll to bottom
        self.scroll_offset = usize::MAX;
    }

    pub fn add_recent_file(&mut self, path: String) {
        // Remove if already present, then push to front
        self.recent_files.retain(|f| f != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    pub fn add_recent_tool(&mut self, name: String, success: bool) {
        self.recent_tools.insert(0, ToolStatus { name, success });
        if self.recent_tools.len() > 8 {
            self.recent_tools.truncate(8);
        }
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
    }
}
