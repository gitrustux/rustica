// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! cp - Copy files and directories

use anyhow::{Context, Result};
use clap::Parser;
use coreutils::file_utils::*;
use std::path::{Path, PathBuf};

/// Copy files and directories
#[derive(Parser, Debug)]
#[command(name = "cp")]
#[command(about = "Copy files and directories", long_about = None)]
struct Args {
    /// Recursive
    #[arg(short, long)]
    recursive: bool,

    /// Force
    #[arg(short, long)]
    force: bool,

    /// Preserve attributes
    #[arg(short, long)]
    preserve: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Source files
    #[arg(required = true)]
    sources: Vec<String>,

    /// Destination
    #[arg(required = true)]
    destination: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let dest_path = Path::new(&args.destination);

    // Check if destination is a directory
    let dest_is_dir = dest_path.exists() && dest_path.is_dir();

    // Handle multiple sources
    if args.sources.len() > 1 {
        if !dest_is_dir {
            anyhow::bail!("target '{}' is not a directory", args.destination);
        }

        for source in &args.sources {
            let src_path = Path::new(source);
            let dest_file = dest_path.join(
                src_path.file_name()
                    .ok_or_else(|| anyhow::anyhow!("invalid source path"))?
            );

            copy_path(src_path, &dest_file, args.recursive, args.verbose)?;
        }
    } else {
        // Single source
        let src_path = Path::new(&args.sources[0]);

        let final_dest = if dest_is_dir {
            dest_path.join(
                src_path.file_name()
                    .ok_or_else(|| anyhow::anyhow!("invalid source path"))?
            )
        } else {
            dest_path.to_path_buf()
        };

        copy_path(src_path, &final_dest, args.recursive, args.verbose)?;
    }

    Ok(())
}

/// Copy a path (file or directory)
fn copy_path(src: &Path, dst: &Path, recursive: bool, verbose: bool) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("cannot stat '{}': No such file or directory", src.display());
    }

    if src.is_dir() {
        if !recursive {
            anyhow::bail!("omitting directory '{}'", src.display());
        }

        if verbose {
            println!("cp: '{}' -> '{}'", src.display(), dst.display());
        }

        copy_dir(src, dst)?;
    } else {
        if verbose {
            println!("cp: '{}' -> '{}'", src.display(), dst.display());
        }

        copy_file(src, dst)?;
    }

    Ok(())
}
