// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! dmesg - Print kernel ring buffer messages

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// Print or control the kernel ring buffer
#[derive(Parser, Debug)]
#[command(name = "dmesg")]
#[command(about = "Print or control the kernel ring buffer", long_about = None)]
struct Args {
    /// Clear the ring buffer
    #[arg(short, long)]
    clear: bool,

    /// Read all messages
    #[arg(short = 'a', long)]
    all: bool,

    /// Show timestamp
    #[arg(short = 'T', long)]
    show_time: bool,

    /// Show human-readable timestamps
    #[arg(short = 'H', long)]
    human: bool,

    /// Follow output
    #[arg(short = 'f', long)]
    follow: bool,

    /// Level filter
    #[arg(short = 'n', long)]
    level: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Try to read from /proc/kmsg (follow mode) or /var/log/kern.log
    let kmsg_path = "/proc/kmsg";
    let kern_log_path = "/var/log/kern.log";

    if args.clear {
        // Clear kernel ring buffer
        // This would require syslog syscall
        eprintln!("dmesg: clear not yet implemented");
        return Ok(());
    }

    if args.follow {
        // Follow mode
        if std::path::Path::new(kmsg_path).exists() {
            follow_dmesg(kmsg_path, args)?;
        } else {
            eprintln!("dmesg: {} does not exist", kmsg_path);
        }
    } else {
        // Read all messages
        if std::path::Path::new(kern_log_path).exists() {
            print_dmesg(kern_log_path, args)?;
        } else if std::path::Path::new(kmsg_path).exists() {
            print_dmesg(kmsg_path, args)?;
        } else {
            eprintln!("dmesg: cannot find kernel log file");
            eprintln!(" Tried: {}, {}", kmsg_path, kern_log_path);
            return Ok(());
        }
    }

    Ok(())
}

/// Print dmesg from file
fn print_dmesg(path: &str, args: Args) -> Result<()> {
    let file = File::open(path)
        .with_context(|| format!("cannot open: {}", path))?;

    let reader = BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

    // Filter by level if specified
    if let Some(level) = args.level {
        // Simple filtering based on log level
        // In production, would parse actual level from message
    }

    // Print lines
    for line in lines {
        if args.show_time || args.human {
            // Timestamp is usually included in the log format
        }
        println!("{}", line);
    }

    Ok(())
}

/// Follow dmesg output
fn follow_dmesg(path: &str, args: Args) -> Result<()> {
    let file = File::open(path)
        .with_context(|| format!("cannot open: {}", path))?;

    let reader = BufReader::new(file);

    println!("Following kernel messages (Ctrl+C to stop)...");
    println!();

    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }

    Ok(())
}
