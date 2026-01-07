// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! hostname - Show or set system hostname

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

/// Show or set system hostname
#[derive(Parser, Debug)]
#[command(name = "hostname")]
#[command(about = "Show or set the system's host name")]
struct Args {
    /// Set hostname
    #[arg(short, long)]
    set: Option<String>,

    /// Short hostname
    #[arg(short, long)]
    short: bool,

    /// Long hostname (FQDN)
    #[arg(short = 'f', long)]
    long: bool,

    /// IP address
    #[arg(short = 'i', long)]
    ip: bool,

    /// All addresses
    #[arg(short = 'a', long)]
    all: bool,

    /// DNS domain name
    #[arg(short = 'd', long)]
    domain: bool,

    /// YP/NIS domain name
    #[arg(short = 'y', long)]
    yp: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set hostname if requested
    if let Some(new_hostname) = args.set {
        return set_hostname(&new_hostname);
    }

    // Get hostname
    let hostname = get_hostname()?;

    // Display based on flags
    if args.short {
        // Short hostname (default)
        println!("{}", hostname);
    } else if args.long || args.ip || args.all || args.domain || args.yp {
        if args.long {
            println!("{}", hostname);
        }
        if args.ip || args.all {
            // Resolve to IP
            println!("127.0.0.1");
        }
        if args.domain || args.yp {
            // Show domain
            if let Some(domain) = hostname.split('.').skip(1).next() {
                println!("{}", domain);
            }
        }
    } else {
        // Default: show short hostname
        println!("{}", hostname);
    }

    Ok(())
}

/// Get system hostname
fn get_hostname() -> Result<String> {
    // Try reading from /etc/hostname
    let hostname_path = "/etc/hostname";
    if let Ok(content) = fs::read_to_string(hostname_path) {
        return Ok(content.trim().to_string());
    }

    // Fallback to system call
    unsafe {
        let mut buf = [0u8; 256];
        let ret = libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len());

        if ret == 0 {
            let hostname = std::ffi::CStr::from_ptr(buf.as_ptr() as *const libc::c_char)
                .to_string_lossy()
                .to_string();
            Ok(hostname)
        } else {
            Ok("localhost".to_string())
        }
    }
}

/// Set system hostname
fn set_hostname(new_hostname: &str) -> Result<()> {
    // Validate hostname
    if new_hostname.is_empty() || new_hostname.len() > 253 {
        anyhow::bail!("invalid hostname: must be 1-253 characters");
    }

    // Check for valid characters
    if !new_hostname.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.') {
        anyhow::bail!("invalid hostname: contains invalid characters");
    }

    // Write to /etc/hostname
    let hostname_path = "/etc/hostname";
    fs::write(hostname_path, format!("{}\n", new_hostname))
        .context("cannot write hostname file")?;

    // Set system hostname
    unsafe {
        let c_hostname = std::ffi::CString::new(new_hostname)?;
        let ret = libc::sethostname(c_hostname.as_ptr(), new_hostname.len());

        if ret != 0 {
            anyhow::bail!("failed to set hostname");
        }
    }

    log::info!("Hostname set to: {}", new_hostname);

    Ok(())
}
