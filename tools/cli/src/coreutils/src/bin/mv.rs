// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! mv - Move (rename) files

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

/// Move (rename) files
#[derive(Parser, Debug)]
#[command(name = "mv")]
#[command(about = "Move (rename) files", long_about = None)]
struct Args {
    /// Force
    #[arg(short, long)]
    force: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Do not overwrite
    #[arg(short = 'n', long)]
    no_clobber: bool,

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

            move_path(src_path, &dest_file, args.force, args.no_clobber, args.verbose)?;
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

        move_path(src_path, &final_dest, args.force, args.no_clobber, args.verbose)?;
    }

    Ok(())
}

/// Move a path
fn move_path(src: &Path, dst: &Path, force: bool, no_clobber: bool, verbose: bool) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("cannot stat '{}': No such file or directory", src.display());
    }

    // Check if destination exists
    if dst.exists() {
        if no_clobber {
            return Ok(());
        }

        if !force {
            eprint!("mv: overwrite '{}'? ", dst.display());
            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;
            if !response.trim().eq_ignore_ascii_case("y") {
                return Ok(());
            }
        }

        // Remove destination
        if dst.is_dir() {
            fs::remove_dir_all(dst)?;
        } else {
            fs::remove_file(dst)?;
        }
    }

    if verbose {
        println!("mv: '{}' -> '{}'", src.display(), dst.display());
    }

    fs::rename(src, dst)
        .with_context(|| format!("cannot move '{}' to '{}'", src.display(), dst.display()))?;

    Ok(())
}
