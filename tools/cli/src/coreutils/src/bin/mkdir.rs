// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! mkdir - Make directories

use anyhow::{Context, Result};
use clap::Parser;
use coreutils::file_utils::create_dir;
use std::path::Path;

/// Make directories
#[derive(Parser, Debug)]
#[command(name = "mkdir")]
#[command(about = "Make directories", long_about = None)]
struct Args {
    /// Create parent directories
    #[arg(short, long)]
    parents: bool,

    /// Set permission mode (octal)
    #[arg(short = 'm', long)]
    mode: Option<String>,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Directories to create
    #[arg(required = true)]
    directories: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    for dir_path in &args.directories {
        let path = Path::new(dir_path);

        // Check if already exists
        if path.exists() {
            anyhow::bail!("cannot create directory '{}': File exists", dir_path);
        }

        if args.verbose {
            println!("mkdir: created directory '{}'", dir_path);
        }

        if args.parents {
            // Create with parents
            create_dir(path)?;
        } else {
            // Create only the final directory
            std::fs::create_dir(path)
                .with_context(|| format!("cannot create directory '{}'", dir_path))?;
        }

        // Set mode if specified
        if let Some(ref mode_str) = args.mode {
            // Parse octal mode
            let mode = u32::from_str_radix(mode_str, 8)
                .with_context(|| format!("invalid mode: {}", mode_str))?;

            // Set permissions (would need to call chmod)
            // For now, just note it
            log::warn!("mode setting not yet implemented: {:o}", mode);
        }
    }

    Ok(())
}
