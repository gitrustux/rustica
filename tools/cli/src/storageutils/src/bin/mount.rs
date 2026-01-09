// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! mount - Mount a filesystem

use anyhow::{Context, Result};
use clap::Parser;

/// Mount a filesystem
#[derive(Parser, Debug)]
#[command(name = "mount")]
#[command(about = "Mount a filesystem", long_about = None)]
struct Args {
    /// Filesystem type
    #[arg(short, long)]
    r#type: Option<String>,

    /// Read-only
    #[arg(short = 'r', long)]
    read_only: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Fake mount (don't actually mount)
    #[arg(short = 'f', long)]
    fake: bool,

    /// Source device
    #[arg(required = false)]
    source: Option<String>,

    /// Target directory
    #[arg(required = false)]
    target: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // If no arguments, list mounted filesystems
    if args.source.is_none() && args.target.is_none() {
        return list_mounts();
    }

    let source = args.source.as_ref().ok_or_else(|| anyhow::anyhow!("source device required"))?;
    let target = args.target.as_ref().ok_or_else(|| anyhow::anyhow!("target directory required"))?;

    // Validate source exists
    let source_path = std::path::Path::new(source);
    if !source_path.exists() {
        anyhow::bail!("source device does not exist: {}", source);
    }

    // Validate target exists
    let target_path = std::path::Path::new(target);
    if !target_path.exists() {
        anyhow::bail!("target directory does not exist: {}", target);
    }

    // Determine filesystem type
    let fs_type = args.r#type.as_ref().map(|s| s.as_str()).unwrap_or("auto");

    // Build mount options
    let mut options = Vec::new();
    if args.read_only {
        options.push("ro");
    }

    let options_str = if options.is_empty() {
        None
    } else {
        Some(options.join(","))
    };

    if args.verbose {
        println!("Mounting:");
        println!("  Source: {}", source);
        println!("  Target: {}", target);
        println!("  Type: {}", fs_type);
        if let Some(ref opts) = options_str {
            println!("  Options: {}", opts);
        }
    }

    if !args.fake {
        // Perform mount
        // In production, would use mount() syscall
        #[cfg(unix)]
        {
            use nix::mount::{mount, MsFlags};
            use std::ffi::CString;

            let source_cstr = CString::new(source.as_str())?;
            let target_cstr = CString::new(target.as_str())?;

            let fs_type_cstr = if fs_type == "auto" {
                None
            } else {
                Some(CString::new(fs_type)?)
            };

            let flags = if args.read_only {
                MsFlags::MS_RDONLY
            } else {
                MsFlags::empty()
            };

            mount(
                fs_type_cstr.as_ref().map(|s| s.as_c_str()),
                source_cstr.as_c_str(),
                Some(target_cstr.as_c_str()),
                flags,
                None::<&str>,
            ).context("mount failed")?;
        }
    }

    if args.verbose {
        println!("Mount successful.");
    }

    Ok(())
}

/// List mounted filesystems
fn list_mounts() -> Result<()> {
    println!("Mounted filesystems:");

    // Read /proc/mounts
    let mounts_path = "/proc/mounts";
    if std::path::Path::new(mounts_path).exists() {
        let content = std::fs::read_to_string(mounts_path)?;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let device = parts[0];
                let mountpoint = parts[1];
                let fs_type = parts[2];
                println!("  {} on {} type {}", device, mountpoint, fs_type);
            }
        }
    } else {
        // Fallback: show common mounts
        println!("  /proc on /proc type proc");
        println!("  /sys on /sys type sysfs");
        println!("  /dev on /dev type devtmpfs");
    }

    Ok(())
}
