use colored::Colorize;
use serde::Serialize;
use std::io::IsTerminal;

#[derive(Serialize)]
pub struct JsonEnvelope<T: Serialize> {
    pub ok: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Returns true if output should be JSON (either --json flag or non-TTY stdout).
pub fn use_json(flag: bool) -> bool {
    flag || !std::io::stdout().is_terminal()
}

/// Emit a JSON envelope to stdout.
pub fn emit_json<T: Serialize>(envelope: &JsonEnvelope<T>) {
    if let Ok(json) = serde_json::to_string_pretty(envelope) {
        println!("{json}");
    }
}

/// Print a check result line with colored checkmark/cross.
pub fn print_check(passed: bool, message: &str) {
    if passed {
        println!("  {} {}", "✓".green(), message);
    } else {
        println!("  {} {}", "✗".red(), message);
    }
}

/// Print a section header.
pub fn print_header(title: &str) {
    println!("{}", title.bold());
}

/// Print an error message to stderr.
pub fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".red().bold(), message);
}
