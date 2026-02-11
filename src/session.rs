//! Agent session management — wraps kernel AgentLoop with TUI-specific callbacks.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use neocognos_kernel::events::{EventBus, EventListener, EventKind, KernelEvent, StderrListener};
use neocognos_kernel::llm::{AnthropicClient, ClaudeCliClient, LlmClient, MockLlmClient, MockStrategy, OllamaClient};
use neocognos_kernel::loop_runner::AgentLoop;
use neocognos_kernel::module_loader::ModuleRegistry;
use neocognos_kernel::policy::PolicyEngine;
use neocognos_kernel::workflow_router::CompiledRouter;
use neocognos_modules::about_me::AboutMeModule;
use neocognos_modules::exec_tool::ExecModule;

use crate::ui;

/// TUI event listener — displays workflow stage progress in the terminal.
struct TuiEventListener {
    /// Track current depth for indentation (shared via atomic)
    depth: std::sync::atomic::AtomicUsize,
}

impl TuiEventListener {
    fn new() -> Self {
        Self { depth: std::sync::atomic::AtomicUsize::new(0) }
    }

    fn indent(&self) -> String {
        let d = self.depth.load(std::sync::atomic::Ordering::Relaxed);
        "  ".repeat(d + 1) // base indent + depth
    }
}

impl EventListener for TuiEventListener {
    fn on_event(&self, event: &KernelEvent) {
        use crossterm::style::{self, Stylize};
        use std::io::Write;

        match &event.event {
            EventKind::StageStarted { stage_id, stage_kind, stage_path, .. } => {
                self.depth.store(stage_path.len(), std::sync::atomic::Ordering::Relaxed);
                let indent = self.indent();
                let connector = if stage_path.len() > 1 { "├─" } else { "" };
                eprint!("{}{}{} {} ",
                    indent,
                    style::style(connector).with(crossterm::style::Color::DarkGrey),
                    style::style("▶").with(crossterm::style::Color::Cyan),
                    style::style(format!("{} ({})", stage_id, stage_kind)).with(crossterm::style::Color::DarkGrey),
                );
                let _ = std::io::stderr().flush();
            }
            EventKind::StageCompleted { duration_ms, skipped, .. } => {
                if *skipped {
                    eprintln!("{}", style::style("skipped").with(crossterm::style::Color::DarkYellow));
                } else {
                    eprintln!("{}", style::style(format!("{}ms", duration_ms)).with(crossterm::style::Color::DarkGrey));
                }
            }
            EventKind::ToolCallStarted { tool_name, arguments, .. } => {
                let indent = self.indent();
                let args_short = if arguments.len() > 60 {
                    format!("{}...", &arguments[..57])
                } else {
                    arguments.clone()
                };
                eprintln!("{}  {} {} {}",
                    indent,
                    style::style("⚡").with(crossterm::style::Color::Yellow),
                    style::style(tool_name).with(crossterm::style::Color::Yellow).bold(),
                    style::style(args_short).with(crossterm::style::Color::DarkGrey),
                );
            }
            EventKind::ToolCallCompleted { tool_name, success, duration_ms, .. } => {
                let indent = self.indent();
                let icon = if *success { "✓" } else { "✗" };
                let color = if *success { crossterm::style::Color::Green } else { crossterm::style::Color::Red };
                eprintln!("{}  {} {} {}",
                    indent,
                    style::style(icon).with(color),
                    style::style(tool_name).with(crossterm::style::Color::DarkGrey),
                    style::style(format!("{}ms", duration_ms)).with(crossterm::style::Color::DarkGrey),
                );
            }
            EventKind::LlmCallStarted { model, .. } => {
                let indent = self.indent();
                eprint!("{}  {} {} ",
                    indent,
                    style::style("🧠").reset(),
                    style::style(format!("llm ({})", model)).with(crossterm::style::Color::DarkGrey),
                );
                let _ = std::io::stderr().flush();
            }
            EventKind::LlmCallCompleted { duration_ms, completion_tokens, .. } => {
                eprintln!("{}", style::style(format!("{}ms, {} tokens", duration_ms, completion_tokens)).with(crossterm::style::Color::DarkGrey));
            }
            _ => {}
        }
    }
}
use neocognos_modules::file_tools::FileToolsModule;
use neocognos_modules::history::HistoryModule;
use neocognos_modules::identity::IdentityModule;
use neocognos_modules::noop::NoopModule;
use neocognos_protocol::*;


/// Session statistics displayed in the status bar.
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub total_prompt_tokens: usize,
    pub total_completion_tokens: usize,
    pub total_turns: usize,
}

impl SessionStats {
    pub fn total_tokens(&self) -> usize {
        self.total_prompt_tokens + self.total_completion_tokens
    }

    /// Rough cost estimate (Anthropic Sonnet pricing as default).
    pub fn estimated_cost(&self) -> f64 {
        let input_cost = self.total_prompt_tokens as f64 * 3.0 / 1_000_000.0;
        let output_cost = self.total_completion_tokens as f64 * 15.0 / 1_000_000.0;
        input_cost + output_cost
    }
}

/// Configuration parsed from CLI args.
pub struct SessionConfig {
    pub manifest_path: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub ollama_url: String,
    pub use_mock: bool,
    pub verbose: bool,
    pub workflow: Option<String>,
    pub autonomy_override: Option<String>,
    pub checkpoint_dir: Option<String>,
    pub event_log_path: Option<String>,
    pub trace_path: Option<String>,
}

/// A TUI session wrapping the agent kernel.
pub struct Session {
    pub agent: AgentLoop,
    pub stats: SessionStats,
    pub model_name: String,
    pub agent_name: String,
    pub agent_version: String,
    pub workflow_name: String,
    pub compiled_router: Option<CompiledRouter>,
}

fn build_module_registry() -> ModuleRegistry {
    let mut registry = ModuleRegistry::new();
    registry.register("noop", || Box::new(NoopModule));
    registry.register("identity", || Box::new(IdentityModule::new()));
    registry.register("history", || Box::new(HistoryModule::new()));
    registry.register("exec", || Box::new(ExecModule::new()));
    registry.register("file_tools", || Box::new(FileToolsModule::new()));
    registry.register("about_me", || Box::new(AboutMeModule::new()));
    registry
}

impl Session {
    /// Create a new session from CLI configuration.
    pub fn from_config(cfg: SessionConfig) -> Result<Self> {
        // Load manifest or defaults
        let (config, system_prompt, module_configs, manifest_model, behavior_config,
             workflow_path, workflow_router_config, manifest_name, manifest_version) =
            if let Some(ref path) = cfg.manifest_path {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read manifest {path}: {e}"))?;
                let manifest: AgentManifest = serde_yaml::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {e}"))?;
                let model = if manifest.model != "mock" { Some(manifest.model.clone()) } else { None };
                let behavior = manifest.behavior.clone();
                let manifest_dir = std::path::Path::new(path).parent()
                    .unwrap_or(std::path::Path::new(".")).to_path_buf();

                if let Some(ref workdir) = manifest.workdir {
                    if workdir != "." {
                        let resolved = if std::path::Path::new(workdir).is_absolute() {
                            std::path::PathBuf::from(workdir)
                        } else {
                            std::env::current_dir()?.join(workdir)
                        };
                        if !resolved.exists() {
                            std::fs::create_dir_all(&resolved)?;
                        }
                        std::env::set_current_dir(&resolved)?;
                    }
                }

                let wf_path = manifest.workflow.map(|wf| {
                    manifest_dir.join(&wf).to_string_lossy().to_string()
                });
                let wf_router = manifest.workflow_router.map(|mut router| {
                    router.default = manifest_dir.join(&router.default).to_string_lossy().to_string();
                    for route in &mut router.routes {
                        route.workflow = manifest_dir.join(&route.workflow).to_string_lossy().to_string();
                    }
                    router
                });
                (manifest.kernel, manifest.system_prompt, manifest.modules, model,
                 behavior, wf_path, wf_router, manifest.name, manifest.version)
            } else {
                (KernelConfig::default(),
                 "You are Neocognos Core, a helpful assistant.".to_string(),
                 vec![], None, BehaviorConfig::default(), None, None,
                 "neocognos".to_string(), "0.1.0".to_string())
            };

        let workflow_path = cfg.workflow.or(workflow_path);

        // Resolve provider/model
        let (resolved_provider, resolved_model) = {
            let raw_model = cfg.model.or(manifest_model);
            let provider_from_cli = cfg.provider;
            match (provider_from_cli, raw_model) {
                (Some(p), Some(m)) => {
                    let model = if let Some((_pfx, rest)) = m.split_once(':') {
                        if m.starts_with(&format!("{p}:")) { rest.to_string() } else { m }
                    } else { m };
                    (p, model)
                }
                (None, Some(m)) => {
                    if let Some((pfx, rest)) = m.split_once(':') {
                        if pfx == "anthropic" || pfx == "ollama" || pfx == "claude-cli" {
                            (pfx.to_string(), rest.to_string())
                        } else {
                            ("ollama".to_string(), m)
                        }
                    } else {
                        ("ollama".to_string(), m)
                    }
                }
                (Some(p), None) => {
                    let default = if p == "anthropic" || p == "claude-cli" {
                        "sonnet".to_string()
                    } else {
                        "llama3.2:3b".to_string()
                    };
                    (p, default)
                }
                (None, None) => ("ollama".to_string(), "llama3.2:3b".to_string()),
            }
        };

        // Build LLM client
        let active_model;
        let llm: Arc<dyn LlmClient> = if cfg.use_mock {
            active_model = "mock".to_string();
            Arc::new(MockLlmClient::new(MockStrategy::Echo))
        } else if resolved_provider == "anthropic" {
            active_model = resolved_model;
            let api_key = cfg.api_key
                .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                .or_else(|| {
                    let env_path = std::path::Path::new(".env");
                    if env_path.exists() {
                        std::fs::read_to_string(env_path).ok().and_then(|content| {
                            content.lines().find_map(|line| {
                                let line = line.trim();
                                line.strip_prefix("ANTHROPIC_API_KEY=")
                                    .map(|val| val.trim_matches('"').trim_matches('\'').to_string())
                            })
                        })
                    } else { None }
                })
                .ok_or_else(|| anyhow::anyhow!("Anthropic API key not found"))?;
            Arc::new(AnthropicClient::new(&active_model, &api_key))
        } else if resolved_provider == "claude-cli" {
            active_model = resolved_model;
            Arc::new(ClaudeCliClient::new(&active_model))
        } else {
            active_model = resolved_model;
            Arc::new(OllamaClient::new(&active_model, &cfg.ollama_url))
        };

        // Create agent loop
        let about_me_system_prompt = system_prompt.clone();
        let about_me_max_turns = config.max_turns;
        let about_me_timeout = config.turn_timeout_secs;
        let about_me_budget = config.token_budget;

        let mut agent = AgentLoop::new(llm, config, system_prompt);
        agent.set_model_name(&active_model);
        agent.set_manifest_path(cfg.manifest_path.clone());

        // Compile workflow router
        let compiled_router = if let Some(ref router_config) = workflow_router_config {
            Some(CompiledRouter::from_config(router_config)?)
        } else { None };

        // Load workflow
        let mut workflow_yaml_text = String::new();
        let mut workflow_name_str = "default-agentic".to_string();
        if compiled_router.is_none() {
            if let Some(ref wf_path) = workflow_path {
                let wf_content = std::fs::read_to_string(wf_path)?;
                workflow_yaml_text = wf_content.clone();
                let wf = neocognos_kernel::workflow::parse_workflow(&wf_content)?;
                workflow_name_str = wf.name.clone();
                agent.set_workflow(wf);
            }
        }

        // Policy
        let mut behavior = behavior_config;
        let about_me_autonomy = format!("{:?}", behavior.autonomy.level);
        if let Some(level_str) = &cfg.autonomy_override {
            if let Ok(level) = level_str.parse::<AutonomyLevel>() {
                behavior.autonomy.level = level;
            }
        }
        agent.set_policy(PolicyEngine::new(behavior));

        // Modules
        let registry = build_module_registry();
        let (modules, errors) = registry.load_from_configs(&module_configs);
        for err in &errors { eprintln!("Warning: {err}"); }

        let mut module_config_map: HashMap<String, serde_json::Value> = HashMap::new();
        for mc in &module_configs {
            module_config_map.insert(mc.name.clone(), mc.config.clone());
        }
        for module in modules {
            agent.add_module(module);
        }

        // Register tool executors
        {
            let mut exec_for_init = ExecModule::new();
            if let Some(cfg) = module_config_map.get("exec") {
                exec_for_init.init(cfg).ok();
            }
            let exec_arc = Arc::new(exec_for_init);
            let exec_clone = exec_arc.clone();
            agent.register_tool_executor("exec", Arc::new(move |call| {
                let command = call.arguments.get("command")
                    .and_then(|v| v.as_str()).unwrap_or("echo");
                let args: Vec<String> = call.arguments.get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let output = exec_clone.execute(command, &args)?;
                Ok(ToolResult { call_id: call.id.clone(), success: true, output })
            }));
        }
        {
            let mut ft = FileToolsModule::new();
            if let Some(cfg) = module_config_map.get("file_tools") {
                ft.init(cfg).ok();
            }
            let ft = Arc::new(ft);
            for tool_name in &["read_file", "write_file", "list_directory"] {
                let ft_clone = ft.clone();
                agent.register_tool_executor(*tool_name, Arc::new(move |call| {
                    ft_clone.execute_tool(call)
                }));
            }
        }
        {
            let mut about_me = AboutMeModule::new();
            let workdir = std::env::current_dir()
                .map(|p| p.display().to_string()).unwrap_or_else(|_| ".".to_string());
            let about_me_config = serde_json::json!({
                "agent_name": manifest_name,
                "agent_version": manifest_version,
                "model": active_model,
                "system_prompt": about_me_system_prompt,
                "workflow_yaml": workflow_yaml_text,
                "workflow_name": workflow_name_str,
                "workdir": workdir,
                "max_turns": about_me_max_turns,
                "turn_timeout_secs": about_me_timeout,
                "token_budget": about_me_budget,
                "autonomy_level": about_me_autonomy,
                "tools": serde_json::json!([
                    {"name": "exec", "description": "Execute shell commands"},
                    {"name": "read_file", "description": "Read file contents"},
                    {"name": "write_file", "description": "Write content to a file"},
                    {"name": "list_directory", "description": "List files in a directory"},
                    {"name": "about_me", "description": "Learn about yourself"}
                ]),
            });
            about_me.init(&about_me_config).ok();
            let about_me = Arc::new(about_me);
            let about_me_clone = about_me.clone();
            agent.register_tool_executor("about_me", Arc::new(move |call| {
                about_me_clone.execute_tool(call)
            }));
        }

        // Event bus — always attach TUI listener for stage progress
        {
            let mut bus = EventBus::new(&format!("tui-{}", std::process::id()));
            bus.add_listener(Box::new(TuiEventListener::new()));
            if cfg.verbose {
                bus.add_listener(Box::new(StderrListener::new(true)));
            }
            agent.set_event_bus(bus);
        }

        agent.init(&module_config_map)?;

        Ok(Session {
            agent,
            stats: SessionStats::default(),
            model_name: active_model,
            agent_name: manifest_name,
            agent_version: manifest_version,
            workflow_name: workflow_name_str,
            compiled_router,
        })
    }

    /// Run a single user turn with streaming output.
    pub fn run_turn(&mut self, input: &str) -> Result<String> {
        // Route workflow if needed
        if let Some(ref router) = self.compiled_router {
            let selected_path = router.select(input);
            if let Ok(wf_content) = std::fs::read_to_string(selected_path) {
                if let Ok(wf) = neocognos_kernel::workflow::parse_workflow(&wf_content) {
                    self.agent.set_workflow(wf);
                }
            }
        }

        // Run the agent in a background thread so we can show a spinner
        let spinner = ui::spinner::create_thinking_spinner();

        let result = self.agent.run_streaming(input, &|_token| {
            // Claude CLI doesn't give us real token streaming,
            // but if another provider does, at least don't block
        })?;

        spinner.finish_and_clear();

        self.stats.total_turns += result.turns;
        self.stats.total_prompt_tokens += result.total_tokens;

        if !result.output.text.is_empty() {
            ui::render::render_markdown(&result.output.text);
        }

        Ok(result.output.text)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.agent.shutdown()
    }
}
