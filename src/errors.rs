use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GodotError {
    pub file: String,
    pub line: u32,
    pub message: String,
}

/// Parse structured error lines from Godot output (--structured-errors mode).
/// Matches lines like: `ERROR res://file.gd:42: Some error message`
pub fn parse_errors(output: &str) -> Vec<GodotError> {
    let mut errors = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Match: ERROR res://path/file.gd:LINE: message
        if let Some(rest) = trimmed.strip_prefix("ERROR res://") {
            if let Some((location, message)) = split_location_message(rest) {
                if let Some((file, line_num)) = parse_file_line(location) {
                    if !is_noise_error(&file, message) {
                        errors.push(GodotError {
                            file,
                            line: line_num,
                            message: message.to_string(),
                        });
                    }
                }
            }
        }
    }

    errors
}

/// Parse SCRIPT ERROR blocks from Godot output (standard mode, no --structured-errors).
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
                        errors.push(GodotError {
                            file,
                            line: line_num,
                            message: message.to_string(),
                        });
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

/// Split "path/file.gd:42: message text" into ("path/file.gd:42", "message text")
fn split_location_message(s: &str) -> Option<(&str, &str)> {
    // Find the second colon (first is after filename, second is after line number)
    let first_colon = s.find(':')?;
    let after_first = first_colon + 1;
    let second_colon = s[after_first..].find(':')? + after_first;
    let location = &s[..second_colon];
    let message = s[second_colon + 1..].trim();
    Some((location, message))
}

/// Parse "path/file.gd:42" into ("res://path/file.gd", 42)
fn parse_file_line(location: &str) -> Option<(String, u32)> {
    let colon = location.rfind(':')?;
    let file = format!("res://{}", &location[..colon]);
    let line_num = location[colon + 1..].parse::<u32>().ok()?;
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
    fn test_parse_structured_error() {
        let output =
            "ERROR res://test_parse_error.gd:4: Parser Error: Unexpected \"Indent\" in class body.";
        let errors = parse_errors(output);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "res://test_parse_error.gd");
        assert_eq!(errors[0].line, 4);
        assert_eq!(
            errors[0].message,
            "Parser Error: Unexpected \"Indent\" in class body."
        );
    }

    #[test]
    fn test_ignores_noise() {
        let output = "ERROR res://.godot/cache:1: Could not load global script cache";
        let errors = parse_errors(output);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_parse_multiple_errors() {
        let output = "\
ERROR res://src/player.gd:15: Invalid get index 'velocity' on base 'Nil'.
Some other output line
ERROR res://src/enemy.gd:3: Identifier not found: speed";
        let errors = parse_errors(output);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].file, "res://src/player.gd");
        assert_eq!(errors[1].file, "res://src/enemy.gd");
    }

    #[test]
    fn test_no_errors() {
        let output = "Godot Engine v4.7.dev\nSome normal output\n";
        let errors = parse_errors(output);
        assert_eq!(errors.len(), 0);
    }

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
}
