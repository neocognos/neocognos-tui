# Neocognos TUI

Rich terminal interface for the [Neocognos](https://github.com/neocognos) agent kernel — inspired by Claude Code.

## Features

- 🎨 **Rich prompt** — colored input with agent name/model indicator
- 📡 **Streaming output** — tokens appear as they're generated
- 📝 **Markdown rendering** — headers, bold, code blocks with syntax highlighting
- ⚡ **Tool call display** — visual tool execution with spinners and result summaries
- 📜 **Multi-line input** — backslash continuation
- ⌨️ **Slash commands** — `/quit`, `/model`, `/clear`, `/help`, `/compact`
- 📊 **Session info** — token count and cost estimates
- 🔄 **History** — readline-style up/down navigation

## Usage

```bash
# With Ollama (default)
cargo run -- --model llama3.2:3b

# With Anthropic API
cargo run -- --provider anthropic --model claude-sonnet-4-20250514

# With Claude CLI (Max subscription)
cargo run -- --provider claude-cli

# With agent manifest
cargo run -- --manifest ../my-agent/agent.yaml

# Mock mode for testing
cargo run -- --mock
```

## Building

Requires Rust 1.75+:

```bash
cargo build --release
```

## Tool Call Display

```
⚡ exec ──────────────────────────────────
│ command: ls -la
├─ running...
│ total 48
│ drwxr-xr-x 5 reza reza 4096 ...
└─ done (0.3s)
```

## Architecture

```
src/
├── main.rs          # Entry point, arg parsing
├── ui/
│   ├── prompt.rs    # Input handling (rustyline)
│   ├── render.rs    # Markdown/output rendering
│   ├── spinner.rs   # Tool call spinners
│   └── theme.rs     # Colors and styling
├── session.rs       # Agent session management
└── commands.rs      # Slash commands
```

## License

MIT
