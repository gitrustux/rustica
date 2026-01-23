// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Built-in Commands
//!
//! This module provides the built-in shell commands.

use crate::parser::Command;
use crate::theme;

/// Syscall declarations
extern "C" {
    fn sys_write(fd: u32, buf: *const u8, len: usize) -> isize;
    fn sys_read(fd: u32, buf: *mut u8, len: usize) -> isize;
    fn sys_spawn(path: *const u8) -> isize;
    fn sys_exit(code: u32) -> !;
}

/// Write a string to stdout
fn print(s: &str) {
    unsafe {
        for &b in s.as_bytes() {
            sys_write(1, &b as *const u8, 1);
        }
    }
}

/// Write a single character to stdout
fn print_char(c: u8) {
    unsafe {
        sys_write(1, &c as *const u8, 1);
    }
}

/// Execute a built-in command
///
/// # Returns
/// * `true` - Command was a built-in and was executed
/// * `false` - Command is not a built-in
pub fn exec_builtin(cmd: &Command) -> bool {
    match cmd.name.as_str() {
        "help" => {
            cmd_help();
            true
        }
        "clear" => {
            cmd_clear();
            true
        }
        "ls" => {
            cmd_ls(&cmd.args);
            true
        }
        "cat" => {
            cmd_cat(&cmd.args);
            true
        }
        "echo" => {
            cmd_echo(&cmd.args);
            true
        }
        "ps" => {
            cmd_ps();
            true
        }
        "exit" => {
            cmd_exit(&cmd.args);
            true
        }
        _ => false,
    }
}

// =============================================================
// BUILT-IN COMMAND IMPLEMENTATIONS
// =============================================================

/// help - Show list of available commands
fn cmd_help() {
    print("\n");
    theme::print_info("Available Commands:\n");
    print("\n");
    print("  Built-in Commands:\n");
    print("    help     - Show this help message\n");
    print("    clear    - Clear the screen\n");
    print("    ls       - List files in ramdisk\n");
    print("    cat      - Print file contents\n");
    print("    echo     - Print arguments\n");
    print("    ps       - List running processes\n");
    print("    exit     - Exit the shell\n");
    print("\n");
    print("  External Programs:\n");
    print("    (Any program in the ramdisk can be executed)\n");
    print("\n");
}

/// clear - Clear the screen
fn cmd_clear() {
    // ANSI escape code to clear screen
    print("\x1b[2J");
    // Move cursor to home
    print("\x1b[H");
}

/// ls - List files in ramdisk
fn cmd_ls(args: &[String]) {
    // For now, we have a simple implementation
    // In the future, this would read from the ramdisk

    // Check if we're listing a specific directory
    let path = if args.is_empty() {
        "/bin"
    } else {
        args[0].as_str()
    };

    // Simple file list for demo
    print("\n");
    theme::print_info(&format!("Directory listing for: {}\n", path));
    print("\n");

    // Known files in ramdisk
    if path == "/bin" || path == "." {
        print("  init   - Init process (PID 1)\n");
        print("  hello  - Hello world program\n");
    }

    print("\n");
}

/// cat - Print file contents
fn cmd_cat(args: &[String]) {
    if args.is_empty() {
        theme::print_error("cat: missing file operand\n");
        print("Usage: cat <file>\n");
        return;
    }

    for filename in args {
        // Try to spawn the program as a simple test
        // In a real implementation, this would read the file contents
        print("\n");
        theme::print_info(&format!("Contents of: {}\n", filename));

        // For demo purposes, show a placeholder
        print("(File content would be displayed here)\n");
        print("\n");
    }
}

/// echo - Print arguments
fn cmd_echo(args: &[String]) {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print(" ");
        }
        print(arg);
    }
    print("\n");
}

/// ps - List running processes
fn cmd_ps() {
    print("\n");
    theme::print_info("Running Processes:\n");
    print("\n");
    print("  PID  PPID  NAME\n");
    print("  ---  ----  ----\n");
    print("    1     0  init\n");
    print("    2     1  shell\n");
    print("\n");
}

/// exit - Exit the shell
fn cmd_exit(args: &[String]) {
    let exit_code = if args.is_empty() {
        0
    } else {
        // Try to parse exit code
        match args[0].parse::<u32>() {
            Ok(code) => code,
            Err(_) => {
                theme::print_error("exit: invalid exit code\n");
                return;
            }
        }
    };

    theme::print_info(&format!("Exiting with code {}\n", exit_code));

    // Exit the shell process
    unsafe {
        sys_exit(exit_code);
    }
}

// =============================================================
// UTILITY FUNCTIONS
// =============================================================

/// Read a line from stdin
pub fn read_line(buffer: &mut [u8]) -> usize {
    let mut count = 0;

    loop {
        let ch: u8;
        unsafe {
            // Block until character available
            loop {
                let result = sys_read(0, &mut ch as *mut u8, 1);
                if result == 1 {
                    break;
                }
                // Yield CPU while waiting
                core::hint::spin_loop();
            }
        }

        match ch {
            b'\n' => {
                // Enter - end of line
                print_char(b'\n');
                break;
            }
            0x08 => {
                // Backspace
                if count > 0 {
                    count -= 1;
                    // Move cursor back and clear character
                    print("\x08 \x08");
                }
            }
            0x20..=0x7E => {
                // Printable ASCII
                if count < buffer.len() - 1 {
                    buffer[count] = ch;
                    count += 1;
                    print_char(ch);
                }
            }
            _ => {
                // Ignore other characters
            }
        }
    }

    // Null-terminate the string
    buffer[count] = 0;
    count
}

/// Check if a string represents a valid path
pub fn is_valid_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    // Check for valid characters
    for ch in path.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '.' | '_' | '-' => {}
            _ => return false,
        }
    }

    true
}

/// Get the prompt string
pub fn get_prompt() -> &'static str {
    "rustux> "
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_path() {
        assert!(is_valid_path("/bin/init"));
        assert!(is_valid_path("hello"));
        assert!(!is_valid_path(""));
        assert!(!is_valid_path("/bin/init@"));
    }

    #[test]
    fn test_get_prompt() {
        assert_eq!(get_prompt(), "rustux> ");
    }
}
