//! Neocognos TUI — Rich terminal interface for the Neocognos agent kernel.

mod commands;
mod session;
mod ui;

use anyhow::Result;
use crossterm::style::{self, Stylize};

use session::{Session, SessionConfig};
use ui::theme;

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn print_welcome(session: &Session) {
    let width = crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    let bar = "─".repeat(width.min(60));

    println!();
    println!("{}", style::style(&bar).with(theme::DIM_COLOR));
    println!("  {} {}",
        style::style("🧬 Neocognos TUI").with(theme::ACCENT_COLOR).bold(),
        style::style(format!("v{}", session.agent_version)).with(theme::DIM_COLOR));
    println!("  {} {}  {} {}  {} {}",
        style::style("Agent:").with(theme::DIM_COLOR),
        style::style(&session.agent_name).with(theme::ASSISTANT_COLOR),
        style::style("Model:").with(theme::DIM_COLOR),
        style::style(&session.model_name).with(theme::USER_COLOR),
        style::style("Workflow:").with(theme::DIM_COLOR),
        style::style(&session.workflow_name).with(theme::TOOL_COLOR));
    println!("{}", style::style(&bar).with(theme::DIM_COLOR));
    println!("  Type {} for commands, {} to exit",
        style::style("/help").with(theme::USER_COLOR),
        style::style("Ctrl+D").with(theme::USER_COLOR));
    println!("{}", style::style(&bar).with(theme::DIM_COLOR));
    println!();
}

fn print_status_bar(session: &Session) {
    let stats = &session.stats;
    if stats.total_turns > 0 {
        let status = format!("[tokens: {} | turns: {} | cost: ~${:.4}]",
            stats.total_tokens(), stats.total_turns, stats.estimated_cost());
        eprintln!("{}", style::style(&status).with(theme::DIM_COLOR));
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if has_flag(&args, "--help") || has_flag(&args, "-h") {
        println!("neocognos-tui — Rich terminal interface for Neocognos agents");
        println!();
        println!("USAGE:");
        println!("  neocognos-tui [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("  --manifest <path>     Agent manifest YAML file");
        println!("  --model <model>       LLM model (e.g. anthropic:claude-sonnet-4-20250514)");
        println!("  --provider <name>     LLM provider (anthropic, ollama, claude-cli)");
        println!("  --api-key <key>       API key for the provider");
        println!("  --ollama-url <url>    Ollama base URL (default: http://localhost:11434)");
        println!("  --workflow <path>     Custom workflow YAML file");
        println!("  --autonomy <level>    Autonomy level (manual, supervised, semi, full)");
        println!("  --mock                Use mock LLM for testing");
        println!("  --verbose             Enable verbose event logging");
        println!("  --checkpoint-dir <d>  Enable checkpointing");
        println!("  --event-log <path>    Write events to JSONL file");
        println!("  --trace <path>        Write trace to file");
        println!("  -h, --help            Show this help");
        return Ok(());
    }

    let config = SessionConfig {
        manifest_path: get_arg(&args, "--manifest"),
        model: get_arg(&args, "--model"),
        provider: get_arg(&args, "--provider"),
        api_key: get_arg(&args, "--api-key"),
        ollama_url: get_arg(&args, "--ollama-url")
            .unwrap_or_else(|| "http://localhost:11434".to_string()),
        use_mock: has_flag(&args, "--mock"),
        verbose: has_flag(&args, "--verbose"),
        workflow: get_arg(&args, "--workflow"),
        autonomy_override: get_arg(&args, "--autonomy"),
        checkpoint_dir: get_arg(&args, "--checkpoint-dir"),
        event_log_path: get_arg(&args, "--event-log"),
        trace_path: get_arg(&args, "--trace"),
    };

    let mut session = Session::from_config(config)?;
    print_welcome(&session);

    let mut prompt = ui::prompt::InputPrompt::new(&session.agent_name, &session.model_name);

    loop {
        print_status_bar(&session);

        let input = match prompt.read_input() {
            Some(s) => s,
            None => {
                println!();
                break;
            }
        };

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Check for slash commands
        match commands::process_command(input) {
            commands::CommandResult::NotACommand => {}
            commands::CommandResult::Continue => continue,
            commands::CommandResult::Quit => break,
            commands::CommandResult::Clear => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                print_welcome(&session);
                continue;
            }
            commands::CommandResult::SwitchModel(model) => {
                println!("⚠ Model switching at runtime not yet implemented. Restart with --model {model}");
                continue;
            }
        }

        match session.run_turn(input) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{} {}",
                    style::style("Error:").with(theme::ERROR_COLOR).bold(),
                    e);
            }
        }
    }

    prompt.save_history();
    session.shutdown()?;

    println!("{}", style::style("Goodbye! 👋").with(theme::DIM_COLOR));
    Ok(())
}
