// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ls - List directory contents

use anyhow::{Context, Result};
use clap::Parser;
use coreutils::{file_utils, print_columns};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// List directory contents
#[derive(Parser, Debug)]
#[command(name = "ls")]
#[command(about = "List directory contents", long_about = None)]
struct Args {
    /// Show hidden files (starting with .)
    #[arg(short, long)]
    all: bool,

    /// Long format
    #[arg(short, long)]
    long: bool,

    /// Human-readable sizes
    #[arg(short = 'h', long)]
    human: bool,

    /// List entries by columns
    #[arg(short = 'C', long)]
    columns: bool,

    /// One per line
    #[arg(short = '1', long)]
    single: bool,

    /// Reverse order
    #[arg(short = 'r', long)]
    reverse: bool,

    /// Sort by time
    #[arg(short = 't', long)]
    sort_time: bool,

    /// Recursive
    #[arg(short = 'R', long)]
    recursive: bool,

    /// Paths to list
    #[arg(default_value = ".")]
    paths: Vec<String>,
}

#[derive(Debug)]
struct FileInfo {
    name: String,
    path: PathBuf,
    is_dir: bool,
    is_link: bool,
    size: u64,
    modified: SystemTime,
    permissions: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut all_entries = Vec::new();

    for path_str in &args.paths {
        let path = Path::new(path_str);

        if !path.exists() {
            eprintln!("ls: cannot access '{}': No such file or directory", path_str);
            continue;
        }

        if path.is_dir() {
            // List directory contents
            let entries = list_directory(path, args.all)?;
            all_entries.extend(entries);
        } else {
            // Single file
            let metadata = fs::metadata(path)
                .with_context(|| format!("cannot stat: {}", path_str))?;

            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            all_entries.push(FileInfo {
                name,
                path: path.to_path_buf(),
                is_dir: metadata.is_dir(),
                is_link: metadata.is_symlink(),
                size: metadata.len(),
                modified: metadata.modified()?,
                permissions: format_permissions(metadata),
            });
        }
    }

    // Sort entries
    if args.sort_time {
        all_entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    } else {
        all_entries.sort_by(|a, b| a.name.cmp(&b.name));
    }

    if args.reverse {
        all_entries.reverse();
    }

    // Display entries
    if args.long {
        print_long_format(&all_entries, args.human);
    } else if args.single {
        for entry in &all_entries {
            println!("{}", entry.name);
        }
    } else {
        let names: Vec<String> = all_entries.iter()
            .map(|e| {
                let mut name = e.name.clone();
                if e.is_dir {
                    name.push('/');
                }
                name
            })
            .collect();
        print_columns(&names, 80);
    }

    Ok(())
}

/// List directory contents
fn list_directory(path: &Path, show_all: bool) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)
        .with_context(|| format!("cannot read directory: {}", path.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_else(|_| "???".to_string());

        // Skip hidden files unless -a
        if !show_all && name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()
            .unwrap_or_else(|_| {
                // Default metadata if stat fails
                fs::metadata(path).unwrap_or_else(|_| {
                    // Create minimal fake metadata
                    // In production, would handle this better
                    panic!("cannot stat file: {}", name)
                })
            });

        let file_type = metadata.file_type();
        let is_dir = file_type.is_dir();
        let is_link = file_type.is_symlink();

        entries.push(FileInfo {
            name,
            path: entry.path(),
            is_dir,
            is_link,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(UNIX_EPOCH),
            permissions: format_permissions(metadata),
        });
    }

    Ok(entries)
}

/// Format permissions string
fn format_permissions(metadata: fs::Metadata) -> String {
    let file_type = if metadata.is_dir() {
        'd'
    } else if metadata.is_symlink() {
        'l'
    } else {
        '-'
    };

    let mode = metadata.permissions().mode();
    let user = format_mode_bits((mode >> 6) & 0x7);
    let group = format_mode_bits((mode >> 3) & 0x7);
    let other = format_mode_bits(mode & 0x7);

    format!("{}{}{}{}", file_type, user, group, other)
}

/// Format mode bits (rwx)
fn format_mode_bits(bits: u32) -> String {
    format!(
        "{}{}{}",
        if bits & 4 != 0 { 'r' } else { '-' },
        if bits & 2 != 0 { 'w' } else { '-' },
        if bits & 1 != 0 { 'x' } else { '-' },
    )
}

/// Print long format
fn print_long_format(entries: &[FileInfo], human: bool) {
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    println!("total {}", total_size / 1024);

    for entry in entries {
        let size_str = if human {
            format_size(entry.size)
        } else {
            entry.size.to_string()
        };

        let modified_str = format_timestamp(entry.modified);

        println!("{} {} {} {} {} {}",
            entry.permissions,
            1, // owner ID (placeholder)
            1, // group ID (placeholder)
            size_str,
            modified_str,
            entry.name,
        );
    }
}

/// Format size in human-readable format
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

/// Format timestamp
fn format_timestamp(time: SystemTime) -> String {
    use chrono::{DateTime, Local};

    let datetime: DateTime<Local> = time.into();
    datetime.format("%b %d %H:%M").to_string()
}
