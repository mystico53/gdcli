use colored::Colorize;
use serde::Serialize;
use std::cell::RefCell;
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

// --- MCP stdout capture ---

thread_local! {
    static CAPTURE_BUF: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Start capturing emit_json output into a buffer instead of stdout.
pub fn begin_capture() {
    CAPTURE_BUF.with(|buf| {
        *buf.borrow_mut() = Some(String::new());
    });
}

/// Stop capturing and return whatever was captured.
pub fn end_capture() -> String {
    CAPTURE_BUF.with(|buf| buf.borrow_mut().take().unwrap_or_default())
}

/// Emit a JSON envelope to stdout (or capture buffer if active).
pub fn emit_json<T: Serialize>(envelope: &JsonEnvelope<T>) {
    if let Ok(json) = serde_json::to_string_pretty(envelope) {
        CAPTURE_BUF.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut s) = *b {
                s.push_str(&json);
            } else {
                println!("{json}");
            }
        });
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
