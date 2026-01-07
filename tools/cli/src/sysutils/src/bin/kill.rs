// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! kill - Send signal to process

use anyhow::{Context, Result};
use clap::Parser;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

/// Send signal to process
#[derive(Parser, Debug)]
#[command(name = "kill")]
#[command(about = "Send signal to process", long_about = None)]
struct Args {
    /// Signal to send (name or number)
    #[arg(short = 's', long)]
    signal: Option<String>,

    /// List signal names
    #[arg(short = 'l', long)]
    list: bool,

    /// PIDs or processes to signal
    #[arg(required = false)]
    pids: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // List signals
    if args.list {
        if args.pids.is_empty() {
            // List all signals
            println!(" 1) SIGHUP       2) SIGINT       3) SIGQUIT      4) SIGILL");
            println!(" 5) SIGTRAP      6) SIGABRT      7) SIGBUS       8) SIGFPE");
            println!(" 9) SIGKILL     10) SIGUSR1     11) SIGSEGV     12) SIGUSR2");
            println!("13) SIGPIPE     14) SIGALRM     15) SIGTERM     17) SIGCHLD");
            println!("18) SIGCONT     19) SIGSTOP     20) SIGTSTP     21) SIGTTIN");
            println!("22) SIGTTOU     23) SIGURG      24) SIGXCPU     25) SIGXFSZ");
            println!("26) SIGVTALRM   27) SIGPROF     28) SIGWINCH    29) SIGIO");
            println!("30) SIGPWR      31) SIGSYS      34) SIGRTMIN    64) SIGRTMAX");
        } else {
            // List specific signal
            for sig_name in &args.pids {
                if let Some(sig) = parse_signal(sig_name) {
                    println!("{}", sig.as_str());
                }
            }
        }
        return Ok(());
    }

    // Default to SIGTERM if no signal specified
    let sig = if let Some(ref sig_str) = args.signal {
        parse_signal(sig_str).unwrap_or(Signal::SIGTERM)
    } else {
        Signal::SIGTERM
    };

    // Send signal to processes
    if args.pids.is_empty() {
        eprintln!("kill: missing operand");
        eprintln!("Try 'kill --help' for more information.");
        return Ok(());
    }

    for pid_str in &args.pids {
        let pid: i32 = pid_str.parse()
            .with_context(|| format!("invalid PID: {}", pid_str))?;

        signal::kill(Pid::from_raw(pid), sig)
            .with_context(|| format!("failed to send signal to process {}", pid))?;

        if sig == Signal::SIGTERM {
            log::info!("Sent SIGTERM to process {}", pid);
        }
    }

    Ok(())
}

/// Parse signal from name or number
fn parse_signal(s: &str) -> Option<Signal> {
    // Try as number first
    if let Ok(num) = s.parse::<i32>() {
        return Signal::try_from(num).ok();
    }

    // Try as name (without SIG prefix)
    let name = s.trim_start_matches("SIG").to_uppercase();
    match name.as_str() {
        "HUP" => Some(Signal::SIGHUP),
        "INT" => Some(Signal::SIGINT),
        "QUIT" => Some(Signal::SIGQUIT),
        "ILL" => Some(Signal::SIGILL),
        "TRAP" => Some(Signal::SIGTRAP),
        "ABRT" => Some(Signal::SIGABRT),
        "BUS" => Some(Signal::SIGBUS),
        "FPE" => Some(Signal::SIGFPE),
        "KILL" => Some(Signal::SIGKILL),
        "USR1" => Some(Signal::SIGUSR1),
        "SEGV" => Some(Signal::SIGSEGV),
        "USR2" => Some(Signal::SIGUSR2),
        "PIPE" => Some(Signal::SIGPIPE),
        "ALRM" => Some(Signal::SIGALRM),
        "TERM" => Some(Signal::SIGTERM),
        "CHLD" => Some(Signal::SIGCHLD),
        "CONT" => Some(Signal::SIGCONT),
        "STOP" => Some(Signal::SIGSTOP),
        "TSTP" => Some(Signal::SIGTSTP),
        "TTIN" => Some(Signal::SIGTTIN),
        "TTOU" => Some(Signal::SIGTTOU),
        _ => None,
    }
}
