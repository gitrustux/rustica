// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Command Parser
//!
//! This module provides parsing for shell commands.

/// Parsed command structure
#[derive(Debug, Clone)]
pub struct Command {
    /// Command name (e.g., "ls", "cat")
    pub name: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Original input line (for error messages)
    pub raw: String,
}

/// Parse error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Empty input
    Empty,
    /// Unterminated quote
    UnterminatedQuote(char),
    /// Invalid escape sequence
    InvalidEscape(char),
    /// Command too long
    TooLong,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty command"),
            ParseError::UnterminatedQuote(ch) => write!(f, "unterminated '{}'", ch),
            ParseError::InvalidEscape(ch) => write!(f, "invalid escape sequence '\\{}'", ch),
            ParseError::TooLong => write!(f, "command too long"),
        }
    }
}

/// Parse a command line into a Command structure
///
/// # Arguments
/// * `line` - Input line from user
///
/// # Returns
/// * `Ok(Command)` - Successfully parsed command
/// * `Err(ParseError)` - Parse error
///
/// # Rules
/// - Space-separated arguments
/// - No pipes or redirection (future)
/// - No quoting (future)
/// - Leading/trailing whitespace is ignored
pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    let line = line.trim();

    if line.is_empty() {
        return Err(ParseError::Empty);
    }

    // Limit command length
    if line.len() > 1024 {
        return Err(ParseError::TooLong);
    }

    // Simple space-separated parsing
    // Future: Add support for quotes, escaping, etc.
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return Err(ParseError::Empty);
    }

    let name = parts[0].to_string();
    let args = parts[1..].iter().map(|s| s.to_string()).collect();

    Ok(Command {
        name,
        args,
        raw: line.to_string(),
    })
}

/// Parse a command line with basic quoting support
///
/// This is an enhanced parser that supports:
/// - Single quotes (')
/// - Double quotes (")
/// - Basic escape sequences (\n, \t, \\, \", \')
///
/// # Arguments
/// * `line` - Input line from user
///
/// # Returns
/// * `Ok(Command)` - Successfully parsed command
/// * `Err(ParseError)` - Parse error
pub fn parse_command_quoted(line: &str) -> Result<Command, ParseError> {
    let line = line.trim();

    if line.is_empty() {
        return Err(ParseError::Empty);
    }

    // Limit command length
    if line.len() > 1024 {
        return Err(ParseError::TooLong);
    }

    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut chars = line.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quote => {
                // Toggle single quote mode
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                // Toggle double quote mode
                in_double_quote = !in_double_quote;
            }
            '\\' if !in_single_quote && !in_double_quote => {
                // Escape sequence
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        'n' => current_arg.push('\n'),
                        't' => current_arg.push('\t'),
                        'r' => current_arg.push('\r'),
                        '\\' => current_arg.push('\\'),
                        '"' => current_arg.push('"'),
                        '\'' => current_arg.push('\''),
                        ' ' => current_arg.push(' '),
                        _ => return Err(ParseError::InvalidEscape(next_ch)),
                    }
                }
            }
            ' ' if !in_single_quote && !in_double_quote => {
                // Space outside quotes - argument separator
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }

    // Check for unterminated quotes
    if in_single_quote {
        return Err(ParseError::UnterminatedQuote('\''));
    }
    if in_double_quote {
        return Err(ParseError::UnterminatedQuote('"'));
    }

    // Add the last argument
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    if args.is_empty() {
        return Err(ParseError::Empty);
    }

    let name = args[0].clone();
    let args_rest = args[1..].to_vec();

    Ok(Command {
        name,
        args: args_rest,
        raw: line.to_string(),
    })
}

/// Check if a command name is a built-in command
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "help" | "clear" | "ls" | "cat" | "echo" | "ps" | "exit" | "cd" | "pwd"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse_command(""), Err(ParseError::Empty));
        assert_eq!(parse_command("   "), Err(ParseError::Empty));
    }

    #[test]
    fn test_parse_simple() {
        let cmd = parse_command("ls").unwrap();
        assert_eq!(cmd.name, "ls");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_parse_with_args() {
        let cmd = parse_command("echo hello world").unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_quoted() {
        let cmd = parse_command_quoted("echo 'hello world'").unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_double_quoted() {
        let cmd = parse_command_quoted("echo \"hello world\"").unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_unterminated_quote() {
        assert_eq!(
            parse_command_quoted("echo 'hello"),
            Err(ParseError::UnterminatedQuote('\''))
        );
    }

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("help"));
        assert!(is_builtin("ls"));
        assert!(is_builtin("exit"));
        assert!(!is_builtin("nonexistent"));
    }
}
