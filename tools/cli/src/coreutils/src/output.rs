// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Output formatting utilities

use std::io::{self, Write};

/// Print columns with padding
pub fn print_columns(items: &[String], width: usize) {
    if items.is_empty() {
        return;
    }

    // Calculate terminal width
    let term_width = terminal_size();

    // Calculate column width
    let max_len = items.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_len + 2;

    // Calculate number of columns
    let num_cols = (term_width / col_width).max(1);

    // Print in columns
    for (i, item) in items.iter().enumerate() {
        print!("{:<width$}", item, width = col_width);
        if (i + 1) % num_cols == 0 {
            println!();
        }
    }

    // Print final newline if needed
    if items.len() % num_cols != 0 {
        println!();
    }
}

/// Get terminal width (default to 80)
fn terminal_size() -> usize {
    // For now, just return a default
    // In production, would use termion or similar
    80
}

/// Print error message
pub fn print_error(msg: &str) {
    eprintln!("\x1b[1;31merror:\x1b[0m {}", msg);
}

/// Print warning message
pub fn print_warning(msg: &str) {
    eprintln!("\x1b[1;33mwarning:\x1b[0m {}", msg);
}
