// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! rm - Remove files or directories

use anyhow::{Context, Result};
use clap::Parser;
use coreutils::file_utils::remove_path;
use std::path::Path;

/// Remove files or directories
#[derive(Parser, Debug)]
#[command(name = "rm")]
#[command(about = "Remove files or directories", long_about = None)]
struct Args {
    /// Recursive
    #[arg(short, long)]
    recursive: bool,

    /// Force
    #[arg(short, long)]
    force: bool,

    /// Interactive
    #[arg(short, long)]
    interactive: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Files to remove
    #[arg(required = true)]
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    for file_path in &args.files {
        let path = Path::new(file_path);

        if !path.exists() {
            if args.force {
                continue;
            }
            anyhow::bail!("cannot remove '{}': No such file or directory", file_path);
        }

        // Interactive prompt
        if args.interactive {
            eprint!("rm: remove '{}'? ", file_path);
            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;
            if !response.trim().eq_ignore_ascii_case("y") {
                continue;
            }
        }

        // Check if directory
        if path.is_dir() && !args.recursive {
            anyhow::bail!("cannot remove '{}': Is a directory", file_path);
        }

        if args.verbose {
            println!("rm: removing '{}'", file_path);
        }

        remove_path(path)?;
    }

    Ok(())
}
