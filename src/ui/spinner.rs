//! Spinner for tool call execution.

use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use crossterm::style::{self, Stylize};

use super::theme;

/// Display a tool call header and return a spinner.
pub fn start_tool_call(tool_name: &str, args: &serde_json::Value) -> ProgressBar {
    // Print tool header
    let header = format!("{}{} {}", theme::TOOL_BORDER_TOP, tool_name,
        "─".repeat(40usize.saturating_sub(tool_name.len() + 2)));
    println!("{}", style::style(&header).with(theme::TOOL_COLOR));

    // Print args summary
    if let Some(obj) = args.as_object() {
        for (key, val) in obj {
            let val_str = match val {
                serde_json::Value::String(s) => {
                    if s.len() > 80 { format!("{}...", &s[..77]) } else { s.clone() }
                }
                other => {
                    let s = other.to_string();
                    if s.len() > 80 { format!("{}...", &s[..77]) } else { s }
                }
            };
            println!("{}{}: {}",
                style::style(theme::TOOL_BORDER_LINE).with(theme::TOOL_COLOR),
                style::style(key).with(theme::DIM_COLOR),
                val_str);
        }
    }

    // Create spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template(&format!("{} {{spinner}} {{msg}}", 
                style::style(theme::TOOL_BORDER_RUNNING).with(theme::TOOL_COLOR)))
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
    );
    pb.set_message("running...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Finish a tool call display with result summary.
pub fn finish_tool_call(pb: ProgressBar, output: &str, success: bool, elapsed: std::time::Duration) {
    pb.finish_and_clear();

    // Print truncated output
    let lines: Vec<&str> = output.lines().collect();
    let max_lines = 10;
    for line in lines.iter().take(max_lines) {
        println!("{}{}",
            style::style(theme::TOOL_BORDER_LINE).with(theme::TOOL_COLOR),
            line);
    }
    if lines.len() > max_lines {
        println!("{}... ({} more lines)",
            style::style(theme::TOOL_BORDER_LINE).with(theme::TOOL_COLOR),
            lines.len() - max_lines);
    }

    // Print done/error footer
    let status = if success { "done" } else { "error" };
    let elapsed_str = format!("{:.1}s", elapsed.as_secs_f64());
    let footer = format!("{}{} ({})", theme::TOOL_BORDER_DONE, status, elapsed_str);
    let color = if success { theme::TOOL_COLOR } else { theme::ERROR_COLOR };
    println!("{}", style::style(&footer).with(color));
}
