// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ps - Report process status

use anyhow::Result;
use clap::Parser;

/// Report process status
#[derive(Parser, Debug)]
#[command(name = "ps")]
#[command(about = "Report process status", long_about = None)]
struct Args {
    /// Show all processes
    #[arg(short, long)]
    all: bool,

    /// Show full format
    #[arg(short = 'f', long)]
    full: bool,

    /// Show processes for all users
    #[arg(short = 'a', long)]
    all_users: bool,

    /// Show process tree
    #[arg(short = 'H', long)]
    forest: bool,

    /// Select by PID
    #[arg(short = 'p', long)]
    pid: Option Vec<u32>>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Print header
    println!("{:<8} {:<8} {:<8} {:<8} {:<8} {:<20} {}",
        "PID", "PPID", "USER", "%CPU", "%MEM", "TIME", "COMMAND");

    // Read /proc filesystem
    let proc_path = std::path::Path::new("/proc");

    if !proc_path.exists() {
        // Fallback: Just show our own process
        print_process(std::process::id(), None, &args);
        return Ok(());
    }

    // Iterate through /proc entries
    for entry in proc_path.read_dir()? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();

        // Skip non-numeric entries
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let pid: u32 = name.parse()?;
        let proc_entry = entry.path();

        // Read process info
        if let Ok(info) = read_process_info(&proc_entry) {
            print_process(pid, Some(info), &args);
        }
    }

    Ok(())
}

/// Process information
struct ProcessInfo {
    ppid: u32,
    state: String,
    utime: u64,
    stime: u64,
    rss: usize,
    comm: String,
}

/// Read process information from /proc
fn read_process_info(proc_path: &std::path::Path) -> Result<ProcessInfo> {
    // Read stat file
    let stat_path = proc_path.join("stat");
    let stat_content = std::fs::read_to_string(&stat_path)?;

    // Parse stat (format: pid (comm) state ppid ...)
    let parts: Vec<&str> = stat_content.split_whitespace().collect();

    let comm = parts[1].trim_matches('(').trim_matches(')').to_string();
    let state = parts[2].to_string();
    let ppid: u32 = parts[3].parse()?;
    let utime: u64 = parts[13].parse()?;
    let stime: u64 = parts[14].parse()?;

    // Read statm for memory info
    let statm_path = proc_path.join("statm");
    let statm_content = std::fs::read_to_string(&statm_path)?;
    let statm_parts: Vec<&str> = statm_content.split_whitespace().collect();
    let rss: usize = statm_parts[1].parse()?; // resident set size in pages

    Ok(ProcessInfo {
        ppid,
        state,
        utime,
        stime,
        rss,
        comm,
    })
}

/// Print process information
fn print_process(pid: u32, info: Option<ProcessInfo>, args: &Args) {
    let ppid = info.as_ref().map(|i| i.ppid as i32).unwrap_or(-1);
    let user = if info.is_some() { "root" } else { "root" };
    let cpu = "0.0";
    let mem = "0.0";
    let time = "00:00:00";
    let cmd = info.as_ref().map(|i| i.comm.as_str()).unwrap_or("[unknown]");

    println!("{:<8} {:<8} {:<8} {:<8} {:<8} {:<20} {}",
        pid,
        ppid,
        user,
        cpu,
        mem,
        time,
        cmd,
    );
}
