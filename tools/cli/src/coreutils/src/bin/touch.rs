// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! touch - Change file timestamps

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::OpenOptions;
use std::path::Path;

/// Change file timestamps
#[derive(Parser, Debug)]
#[command(name = "touch")]
#[command(about = "Change file timestamps", long_about = None)]
struct Args {
    /// Do not create file if it doesn't exist
    #[arg(short, long)]
    no_create: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Files to touch
    #[arg(required = true)]
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    for file_path in &args.files {
        let path = Path::new(file_path);

        if path.exists() {
            // Update modification time
            if args.verbose {
                println!("touch: updating '{}'", file_path);
            }

            // Reopen and close to update timestamp
            let _ = OpenOptions::new()
                .write(true)
                .open(path)
                .with_context(|| format!("cannot touch '{}'", file_path))?;
        } else {
            // Create new file
            if args.no_create {
                continue;
            }

            if args.verbose {
                println!("touch: creating '{}'", file_path);
            }

            OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)
                .with_context(|| format!("cannot create '{}'", file_path))?;
        }
    }

    Ok(())
}
