// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Rustica Shell (sh)
//!
//! A minimal POSIX-compatible shell implementation in Rust.
//! Provides command execution, built-in commands, and job control.

use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Command, ExitCode};

/// Rustica Shell - Command Interpreter
#[derive(Parser, Debug)]
#[command(name = "sh")]
#[command(about = "Rustica Shell - POSIX-compatible command interpreter", long_about = None)]
struct Args {
    /// Script file to execute
    #[arg(short, long)]
    command: Option<String>,

    /// Path to script file
    #[arg(short, long)]
    file: Option<String>,

    /// Interactive mode (default)
    #[arg(short, long)]
    interactive: bool,
}

/// Built-in commands
const BUILTINS: &[&str] = &[
    "cd",    // Change directory
    "pwd",   // Print working directory
    "echo",  // Echo arguments
    "export", // Export environment variable
    "unset", // Unset environment variable
    "exit",  // Exit shell
    "help",  // Show help
];

fn main() -> ExitCode {
    let args = Args::parse();

    // Set up logging
    env_logger::init();

    // Run shell
    if let Err(e) = run_shell(args) {
        eprintln!("sh: error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// Main shell execution loop
fn run_shell(args: Args) -> Result<()> {
    // Display splash screen on first interactive start
    if args.interactive || (args.command.is_none() && args.file.is_none()) {
        display_splash();
    }

    // Execute file script
    if let Some(file) = args.file {
        return execute_script(&file);
    }

    // Execute single command
    if let Some(cmd) = args.command {
        return execute_command_line(&cmd);
    }

    // Interactive mode
    run_interactive()
}

/// Display shell splash screen
fn display_splash() {
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────");
    println!("─████████████████───██████──██████─██████████████─██████████████─██████████─██████████████─██████████████─");
    println!("─██░░░░░░░░░░░░██───██░░██──██░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─██░░░░░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─");
    println!("─██░░████████░░██───██░░██──██░░██─██░░██████████─██████░░██████─████░░████─██░░██████████─██░░██████░░██─");
    println!("─██░░██────██░░██───██░░██──██░░██─██░░██─────────────██░░██───────██░░██───██░░██─────────██░░██──██░░██─");
    println!("─██░░████████░░██───██░░██──██░░██─██░░██████████─────██░░██───────██░░██───██░░██─────────██░░██████░░██─");
    println!("─██░░░░░░░░░░░░██───██░░██──██░░██─██░░░░░░░░░░██─────██░░██───────██░░██───██░░██─────────██░░░░░░░░░░██─");
    println!("─██░░██████░░████───██░░██──██░░██─██████████░░██─────██░░██───────██░░██───██░░██─────────██░░██████░░██─");
    println!("─██░░██──██░░██─────██░░██──██░░██─────────██░░██─────██░░██───────██░░██───██░░██─────────██░░██──██░░██─");
    println!("─██░░██──██░░██████─██░░██████░░██─██████████░░██─────██░░██─────████░░████─██░░██████████─██░░██──██░░██─");
    println!("─██░░██──██░░░░░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─────██░░██─────██░░░░░░██─██░░░░░░░░░░██─██░░██──██░░██─");
    println!("─██████──██████████─██████████████─██████████████─────██████─────██████████─██████████████─██████──██████─");
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────");
    println!("Rustica Shell v0.1.0");
    println!("Type 'help' for available commands");
    println!();
}

/// Run interactive shell
fn run_interactive() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    // Set up environment
    let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    env::set_var("PATH", "/bin:/usr/bin:/usr/local/bin");
    env::set_var("HOME", &home);
    env::set_var("USER", "root");
    env::set_var("SHELL", "/bin/sh");

    // Change to home directory
    let _ = env::set_current_dir(&home);

    loop {
        // Display prompt
        let cwd = env::current_dir()
            .and_then(|p| p.canonicalize())
            .unwrap_or_else(|_| Path::new("/").to_path_buf());

        let prompt = format!(
            "\x1b[1;32mroot\x1b[0m@\x1b[1;34mrustica\x1b[0m:\x1b[1;36m{}\x1b[0m# ",
            cwd.display()
        );

        print!("{}", prompt);
        stdout.flush()?;

        // Read line
        let line = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(e)) => {
                eprintln!("sh: read error: {}", e);
                continue;
            }
            None => break, // EOF
        };

        // Skip empty lines
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Execute command
        if let Err(e) = execute_command_line(line) {
            eprintln!("sh: {}", e);
        }
    }

    println!(); // Newline after exit
    Ok(())
}

/// Execute a single command line
fn execute_command_line(line: &str) -> Result<()> {
    let parts = parse_command_line(line)?;

    if parts.is_empty() {
        return Ok(());
    }

    let cmd = &parts[0];
    let args = &parts[1..];

    // Check for built-in commands
    if is_builtin(cmd) {
        return execute_builtin(cmd, args);
    }

    // Execute external command
    execute_external(cmd, args)
}

/// Parse command line into parts
fn parse_command_line(line: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escape = false;

    for ch in line.chars() {
        if escape {
            current.push(ch);
            escape = false;
        } else if ch == '\\' {
            escape = true;
        } else if ch == '"' {
            in_quote = !in_quote;
        } else if ch.is_whitespace() && !in_quote {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    Ok(parts)
}

/// Check if command is a built-in
fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

/// Execute built-in command
fn execute_builtin(cmd: &str, args: &[String]) -> Result<()> {
    match cmd {
        "cd" => builtin_cd(args),
        "pwd" => builtin_pwd(),
        "echo" => builtin_echo(args),
        "export" => builtin_export(args),
        "unset" => builtin_unset(args),
        "exit" => builtin_exit(args),
        "help" => builtin_help(),
        _ => anyhow::bail!("unknown built-in: {}", cmd),
    }
}

/// Built-in: cd - Change directory
fn builtin_cd(args: &[String]) -> Result<()> {
    let target = if args.is_empty() {
        // Default to home directory
        env::var("HOME").unwrap_or_else(|_| "/root".to_string())
    } else {
        args[0].clone()
    };

    let path = Path::new(&target);

    // Handle ~ expansion
    let expanded = if target.starts_with('~') {
        let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        Path::new(&home).join(target.trim_start_matches('~').trim_start_matches('/'))
    } else {
        path.to_path_buf()
    };

    env::set_current_dir(&expanded)
        .with_context(|| format!("cd: {}", target))?;

    Ok(())
}

/// Built-in: pwd - Print working directory
fn builtin_pwd() -> Result<()> {
    let cwd = env::current_dir()
        .context("pwd: cannot get current directory")?;

    println!("{}", cwd.display());
    Ok(())
}

/// Built-in: echo - Echo arguments
fn builtin_echo(args: &[String]) -> Result<()> {
    println!("{}", args.join(" "));
    Ok(())
}

/// Built-in: export - Set environment variable
fn builtin_export(args: &[String]) -> Result<()> {
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            env::set_var(key, value);
        } else {
            // Print environment variable
            if let Ok(value) = env::var(arg) {
                println!("{}={}", arg, value);
            }
        }
    }
    Ok(())
}

/// Built-in: unset - Unset environment variable
fn builtin_unset(args: &[String]) -> Result<()> {
    for arg in args {
        env::remove_var(arg);
    }
    Ok(())
}

/// Built-in: exit - Exit shell
fn builtin_exit(_args: &[String]) -> Result<()> {
    std::process::exit(0);
}

/// Built-in: help - Show help
fn builtin_help() -> Result<()> {
    println!("Rustica Shell v0.1.0");
    println!();
    println!("Built-in commands:");
    for builtin in BUILTINS {
        println!("  {}", builtin);
    }
    println!();
    println!("External commands:");
    println!("  ls, cat, cp, mv, rm, mkdir, touch, ps, kill, dmesg, uname, date");
    println!("  ip, ping, hostname, nslookup");
    println!("  pkg, fwctl, svc, system-check");
    println!();
    println!("For more information on a command, see its documentation.");
    Ok(())
}

/// Execute external command
fn execute_external(cmd: &str, args: &[String]) -> Result<()> {
    let result = Command::new(cmd)
        .args(args)
        .spawn();

    match result {
        Ok(mut child) => {
            child.wait()
                .context(format!("{}: execution failed", cmd))?;
            Ok(())
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                anyhow::bail!("{}: command not found", cmd);
            } else {
                anyhow::bail!("{}: {}", cmd, e);
            }
        }
    }
}

/// Execute script file
fn execute_script(path: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read script: {}", path))?;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        execute_command_line(line)?;
    }

    Ok(())
}
