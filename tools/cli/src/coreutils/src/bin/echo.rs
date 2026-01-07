// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! echo - Display a line of text

use clap::Parser;

/// Display a line of text
#[derive(Parser, Debug)]
#[command(name = "echo")]
#[command(about = "Display a line of text", long_about = None)]
struct Args {
    /// Do not output trailing newline
    #[arg(short = 'n', long)]
    no_newline: bool,

    /// Enable escape sequences
    #[arg(short = 'e', long)]
    enable_escape: bool,

    /// Text to display
    #[arg(required = true)]
    text: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let output = args.text.join(" ");

    let result = if args.enable_escape {
        // Process escape sequences
        process_escape_sequences(&output)
    } else {
        output
    };

    if args.no_newline {
        print!("{}", result);
    } else {
        println!("{}", result);
    }
}

/// Process escape sequences
fn process_escape_sequences(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('0') => result.push('\0'),
                Some('x') => {
                    // Hex escape (e.g., \x1B)
                    let mut hex = String::new();
                    for _ in 0..2 {
                        if let Some(h) = chars.next() {
                            hex.push(h);
                        }
                    }
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    }
                }
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
