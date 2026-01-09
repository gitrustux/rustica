// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! umount - Unmount a filesystem

use anyhow::{Context, Result};
use clap::Parser;

/// Unmount a filesystem
#[derive(Parser, Debug)]
#[command(name = "umount")]
#[command(about = "Unmount a filesystem", long_about = None)]
struct Args {
    /// Lazy unmount (detach immediately)
    #[arg(short, long)]
    lazy: bool,

    /// Force unmount
    #[arg(short = 'f', long)]
    force: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Mountpoint to unmount
    #[arg(required = true)]
    target: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let target_path = std::path::Path::new(&args.target);

    if !target_path.exists() {
        anyhow::bail!("mountpoint does not exist: {}", args.target);
    }

    if args.verbose {
        println!("Unmounting: {}", args.target);
    }

    #[cfg(unix)]
    {
        use nix::mount::umount;

        // Note: The nix crate's umount() doesn't support flags directly
        // For lazy unmount (MNT_DETACH), we would need to use the raw syscall
        if args.lazy {
            // Stub: In production, would use libc::umount2 with MNT_DETACH
            // For now, just warn and proceed with normal unmount
            eprintln!("Warning: Lazy unmount not fully implemented, attempting normal unmount");
        }

        umount(target_path)
            .context("umount failed")?;
    }

    if args.verbose {
        println!("Unmount successful.");
    }

    Ok(())
}
