// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! cat - Concatenate and print files

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;

/// Concatenate and print files
#[derive(Parser, Debug)]
#[command(name = "cat")]
#[command(about = "Concatenate and print files", long_about = None)]
struct Args {
    /// Show line numbers
    #[arg(short = 'n', long)]
    number: bool,

    /// Show non-printing characters
    #[arg(short = 'A', long)]
    show_all: bool,

    /// Squeeze blank lines
    #[arg(short = 's', long)]
    squeeze_blank: bool,

    /// Files to concatenate
    #[arg(default_value = "-")]
    files: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    for file_path in &args.files {
        if file_path == "-" {
            // Read from stdin
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            copy_to_output(&mut reader, &mut writer, args.number, args.show_all)?;
        } else {
            // Read from file
            let path = Path::new(file_path);
            let mut file = File::open(path)
                .with_context(|| format!("cannot open: {}", file_path))?;
            copy_to_output(&mut file, &mut writer, args.number, args.show_all)?;
        }
    }

    writer.flush()?;
    Ok(())
}

/// Copy from reader to writer
fn copy_to_output<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    show_numbers: bool,
    show_all: bool,
) -> Result<()> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut line_number = 1;

    if show_numbers {
        print!("     1  ");
    }

    for (i, byte) in buffer.iter().enumerate() {
        if show_all {
            // Show non-printing characters
            match byte {
                b'\n' => {
                    println!("$");
                    if show_numbers && i < buffer.len() - 1 {
                        line_number += 1;
                        print!("{:>7}  ", line_number);
                    }
                }
                b'\t' => print!("^I"),
                0..=8 | 11..=31 | 127 => print!("^{}", (byte + 64) as char),
                _ => print!("{}", *byte as char),
            }
        } else {
            if *byte == b'\n' {
                println!();
                if show_numbers && i < buffer.len() - 1 {
                    line_number += 1;
                    print!("{:>7}  ", line_number);
                }
            } else {
                print!("{}", *byte as char);
            }
        }
    }

    // Ensure final newline if needed
    if !buffer.is_empty() && *buffer.last().unwrap() != b'\n' {
        println!();
    }

    Ok(())
}
