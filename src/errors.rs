use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GodotError {
    pub file: String,
    pub line: u32,
    pub message: String,
}

/// Parse SCRIPT ERROR blocks from Godot output.
/// Matches blocks like:
/// ```
/// SCRIPT ERROR: Parse Error: Unexpected "Indent" in class body.
///    at: GDScript::reload (res://test_parse_error.gd:4)
/// ```
pub fn parse_script_errors(output: &str) -> Vec<GodotError> {
    let mut errors = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(message) = trimmed.strip_prefix("SCRIPT ERROR: ") {
            // Look at the next line for the "at:" location
            if i + 1 < lines.len() {
                let at_line = lines[i + 1].trim();
                if let Some(rest) = at_line.strip_prefix("at: ") {
                    if let Some((file, line_num)) = parse_at_location(rest) {
                        if !is_noise_error(&file, message) {
                            errors.push(GodotError {
                                file,
                                line: line_num,
                                message: message.to_string(),
                            });
                        }
                        i += 2;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }

    errors
}

/// Parse "GDScript::reload (res://file.gd:42)" into ("res://file.gd", 42)
fn parse_at_location(at_text: &str) -> Option<(String, u32)> {
    // Find the part in parentheses: (res://file.gd:42)
    let open = at_text.find('(')?;
    let close = at_text.find(')')?;
    let inner = &at_text[open + 1..close];

    // inner is "res://file.gd:42"
    let colon = inner.rfind(':')?;
    let file = inner[..colon].to_string();
    let line_num = inner[colon + 1..].parse::<u32>().ok()?;

    Some((file, line_num))
}

/// Filter out non-actionable noise errors from Godot output.
fn is_noise_error(file: &str, message: &str) -> bool {
    // Global class cache error appears on every headless run
    if message.contains("Could not load global script cache") {
        return true;
    }
    // Ignore errors in Godot's own internal files
    if file.starts_with("res://.godot/") {
        return true;
    }
    false
}

/// Format errors for TTY display.
pub fn format_error_tty(error: &GodotError) -> String {
    format!("{}:{}: {}", error.file, error.line, error.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_script_error_block() {
        let output = r#"SCRIPT ERROR: Parse Error: Unexpected "Indent" in class body.
   at: GDScript::reload (res://test_parse_error.gd:4)"#;
        let errors = parse_script_errors(output);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "res://test_parse_error.gd");
        assert_eq!(errors[0].line, 4);
        assert_eq!(
            errors[0].message,
            "Parse Error: Unexpected \"Indent\" in class body."
        );
    }

    #[test]
    fn test_parse_script_error_no_match() {
        let output = "Some normal output\nNo errors here\n";
        let errors = parse_script_errors(output);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_parse_multiple_script_errors() {
        let output = "\
SCRIPT ERROR: Invalid get index 'velocity' on base 'Nil'.
   at: Player.gd::_process (res://src/player.gd:15)
Some other output line
SCRIPT ERROR: Identifier not found: speed
   at: Enemy.gd::_ready (res://src/enemy.gd:3)";
        let errors = parse_script_errors(output);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].file, "res://src/player.gd");
        assert_eq!(errors[0].line, 15);
        assert_eq!(errors[1].file, "res://src/enemy.gd");
        assert_eq!(errors[1].line, 3);
    }

    #[test]
    fn test_script_errors_ignores_noise() {
        let output = "\
SCRIPT ERROR: Could not load global script cache
   at: (res://.godot/cache:1)
SCRIPT ERROR: Real error here
   at: GDScript::reload (res://player.gd:10)";
        let errors = parse_script_errors(output);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "res://player.gd");
        assert_eq!(errors[0].message, "Real error here");
    }
}
