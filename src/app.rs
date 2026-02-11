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

/// Which panel has focus for scrolling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelFocus {
    Chat,
    Trace,
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
    pub focus: PanelFocus,
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
            focus: PanelFocus::Chat,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = App::new("test-agent", "sonnet", "default");
        assert!(app.messages.is_empty());
        assert!(app.input.is_empty());
        assert_eq!(app.cursor_pos, 0);
        assert_eq!(app.scroll_offset, 0);
        assert_eq!(app.status.agent_name, "test-agent");
        assert_eq!(app.status.model, "sonnet");
        assert_eq!(app.status.workflow, "default");
        assert_eq!(app.status.total_tokens, 0);
        assert_eq!(app.status.cost, 0.0);
        assert_eq!(app.focus, PanelFocus::Chat);
        assert!(!app.agent_busy);
        assert!(!app.should_quit);
        assert!(app.input_history.is_empty());
        assert!(app.history_index.is_none());
    }

    #[test]
    fn test_add_message() {
        let mut app = App::new("a", "m", "w");
        app.add_message(ChatMessage::User("hello".into()));
        assert_eq!(app.messages.len(), 1);
        assert_eq!(app.scroll_offset, usize::MAX);
        app.add_message(ChatMessage::Assistant("hi".into()));
        assert_eq!(app.messages.len(), 2);
    }

    #[test]
    fn test_add_recent_tool() {
        let mut app = App::new("a", "m", "w");
        for i in 0..10 {
            app.add_recent_tool(format!("tool_{i}"), true);
        }
        assert_eq!(app.recent_tools.len(), 8); // max capacity
        assert_eq!(app.recent_tools[0].name, "tool_9"); // most recent first
    }

    #[test]
    fn test_add_recent_file() {
        let mut app = App::new("a", "m", "w");
        app.add_recent_file("a.rs".into());
        app.add_recent_file("b.rs".into());
        app.add_recent_file("a.rs".into()); // dedup
        assert_eq!(app.recent_files.len(), 2);
        assert_eq!(app.recent_files[0], "a.rs"); // moved to front

        for i in 0..15 {
            app.add_recent_file(format!("file_{i}.rs"));
        }
        assert_eq!(app.recent_files.len(), 10); // max capacity
    }

    #[test]
    fn test_input_editing() {
        let mut app = App::new("a", "m", "w");
        app.insert_char('h');
        app.insert_char('i');
        assert_eq!(app.input, "hi");
        assert_eq!(app.cursor_pos, 2);

        app.move_cursor_left();
        assert_eq!(app.cursor_pos, 1);
        app.insert_char('!');
        assert_eq!(app.input, "h!i");

        app.move_cursor_home();
        assert_eq!(app.cursor_pos, 0);
        app.move_cursor_end();
        assert_eq!(app.cursor_pos, 3);

        app.delete_char_before();
        assert_eq!(app.input, "h!");
        app.move_cursor_home();
        app.delete_char_after();
        assert_eq!(app.input, "!");
    }

    #[test]
    fn test_history_navigation() {
        let mut app = App::new("a", "m", "w");
        app.input = "first".into();
        app.submit_input();
        app.input = "second".into();
        app.submit_input();

        app.history_up();
        assert_eq!(app.input, "second");
        app.history_up();
        assert_eq!(app.input, "first");
        app.history_up(); // at beginning, stays
        assert_eq!(app.input, "first");

        app.history_down();
        assert_eq!(app.input, "second");
        app.history_down(); // past end, clears
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_panel_focus_toggle() {
        let mut app = App::new("a", "m", "w");
        assert_eq!(app.focus, PanelFocus::Chat);
        app.focus = PanelFocus::Trace;
        assert_eq!(app.focus, PanelFocus::Trace);
        app.focus = PanelFocus::Chat;
        assert_eq!(app.focus, PanelFocus::Chat);
    }

    #[test]
    fn test_status_display() {
        let mut info = StatusInfo::default();
        info.total_tokens = 500;
        assert_eq!(info.tokens_display(), "500");

        info.total_tokens = 1000;
        assert_eq!(info.tokens_display(), "1.0k");

        info.total_tokens = 15432;
        assert_eq!(info.tokens_display(), "15.4k");

        info.cost = 0.0123;
        assert_eq!(info.cost_display(), "~$0.0123");
    }

    #[test]
    fn test_clear_messages() {
        let mut app = App::new("a", "m", "w");
        app.add_message(ChatMessage::User("hi".into()));
        app.add_message(ChatMessage::System("ok".into()));
        assert_eq!(app.messages.len(), 2);
        app.clear_messages();
        assert!(app.messages.is_empty());
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_submit_input_empty() {
        let mut app = App::new("a", "m", "w");
        app.input = "   ".into();
        assert!(app.submit_input().is_none());
        assert!(app.input_history.is_empty());
    }

    #[test]
    fn test_chat_message_variants() {
        let _msgs = vec![
            ChatMessage::User("u".into()),
            ChatMessage::Assistant("a".into()),
            ChatMessage::Narration("n".into()),
            ChatMessage::ToolCall { name: "t".into(), args_short: "{}".into() },
            ChatMessage::ToolResult { name: "t".into(), success: true, duration_ms: 100 },
            ChatMessage::Error("e".into()),
            ChatMessage::System("s".into()),
        ];
    }

    #[test]
    fn test_trace_entry_variants() {
        let _entries = vec![
            TraceEntry::StageStart { id: "s1".into(), kind: "plan".into() },
            TraceEntry::StageEnd { id: "s1".into(), duration_ms: 50, skipped: false },
            TraceEntry::LlmCall { model: "m".into(), ctx_tokens: 100, out_tokens: 50, duration_ms: 200 },
            TraceEntry::ToolCall { name: "t".into(), args: "{}".into() },
            TraceEntry::ToolResult { name: "t".into(), success: true, duration_ms: 10 },
            TraceEntry::Narration("n".into()),
        ];
    }
}
