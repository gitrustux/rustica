// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Rustica Shell
//!
//! This is the main shell for Rustux OS.
//! It runs as a userspace process and provides an interactive command-line interface.

#![no_std]
#![no_main]

extern crate alloc;

mod parser;
mod theme;
mod builtins;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use parser::parse_command;
use theme::{print_prompt, print_error, print_success, print_info};
use builtins::{exec_builtin, read_line};

// =============================================================
// SYSCALL DECLARATIONS
// =============================================================

extern "C" {
    /// Write to a file descriptor
    fn sys_write(fd: u32, buf: *const u8, len: usize) -> isize;

    /// Read from a file descriptor
    fn sys_read(fd: u32, buf: *mut u8, len: usize) -> isize;

    /// Spawn a process from ramdisk path
    fn sys_spawn(path: *const u8) -> isize;

    /// Get current process ID
    fn sys_getpid() -> u32;

    /// Get parent process ID
    fn sys_getppid() -> u32;

    /// Exit the current process
    fn sys_exit(code: u32) -> !;
}

// =============================================================
// ENTRY POINT
// =============================================================

#[no_mangle]
pub extern "C" fn main() -> u32 {
    // Clear screen on startup
    print("\x1b[2J\x1b[H");

    // Print welcome banner
    print_welcome();

    // Main shell loop
    shell_loop();

    // Should not reach here
    0
}

// =============================================================
// SHELL MAIN LOOP
// =============================================================

fn shell_loop() -> ! {
    let mut input_buffer = [0u8; 512];

    loop {
        // Print prompt
        print_prompt();

        // Read input line
        let count = read_line(&mut input_buffer);

        if count == 0 {
            // Empty line - continue
            continue;
        }

        // Parse command
        let line = unsafe {
            core::str::from_utf8_unchecked(&input_buffer[..count])
        };

        match parse_command(line) {
            Ok(cmd) => {
                execute_command(&cmd);
            }
            Err(parser::ParseError::Empty) => {
                // Empty command - do nothing
                continue;
            }
            Err(e) => {
                print_error(&format!("parse error: {}\n", e));
            }
        }
    }
}

// =============================================================
// COMMAND EXECUTION
// =============================================================

fn execute_command(cmd: &parser::Command) {
    // Check if it's a built-in command
    if exec_builtin(cmd) {
        return;
    }

    // Try to spawn an external program
    spawn_program(&cmd.name, &cmd.args);
}

fn spawn_program(name: &str, _args: &[String]) {
    // Build the path to the program
    let path = format!("/bin/{}", name);

    // Try to spawn the program
    let result = unsafe {
        sys_spawn(path.as_ptr() as *const u8)
    };

    if result < 0 {
        // Failed to spawn - show error
        print_error(&format!("command not found: {}\n", name));
    } else {
        // Successfully spawned
        print_success(&format!("started process with PID {}\n", result));
    }
}

// =============================================================
// UTILITY FUNCTIONS
// =============================================================

fn print(s: &str) {
    unsafe {
        for &b in s.as_bytes() {
            sys_write(1, &b as *const u8, 1);
        }
    }
}

fn print_welcome() {
    print("\n");
    theme::set_color(theme::DRACULA_PURPLE);
    print("╔════════════════════════════════════════════════════════════════╗\n");
    print("║                                                                ║\n");
    print("║                    Welcome to Rustux OS                        ║\n");
    print("║                    Dracula Theme v1.0                          ║\n");
    print("║                                                                ║\n");
    print("║  Type 'help' for available commands                           ║\n");
    print("║                                                                ║\n");
    print("╚════════════════════════════════════════════════════════════════╝\n");
    theme::reset_color();
    print("\n");
}

// =============================================================
// PANIC HANDLER
// =============================================================

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    print("\n");
    theme::set_color(theme::DRACULA_RED);
    print("PANIC: ");
    theme::reset_color();
    print("Shell panic!\n");
    unsafe {
        sys_exit(1);
    }
}
