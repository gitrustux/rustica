// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! date - Print or set system date and time

use anyhow::{Context, Result};
use clap::Parser;
use chrono::{Local, DateTime};

/// Print or set system date and time
#[derive(Parser, Debug)]
#[command(name = "date")]
#[command(about = "Print or set system date and time", long_about = None)]
struct Args {
    /// Format string
    #[arg(short, long)]
    format: Option<String>,

    /// Set date
    #[arg(short = 's', long)]
    set: Option<String>,

    /// Universal time (UTC)
    #[arg(short = 'u', long)]
    universal: bool,

    /// RFC 3339 format
    #[arg(short = 'I', long)]
    iso_8601: bool,

    /// RFC 5322 date
    #[arg(short = 'R', long)]
    rfc_email: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Get current time
    let now = if args.universal {
        chrono::Utc::now().naive_utc()
    } else {
        Local::now().naive_local()
    };

    // Set date if requested
    if let Some(ref date_str) = args.set {
        return set_date(date_str);
    }

    // Format output
    let output = if let Some(ref format) = args.format {
        // Custom format
        format_date(&now, format)
    } else if args.iso_8601 {
        // ISO 8601 format
        now.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
    } else if args.rfc_email {
        // RFC 5322 format
        now.format("%a, %d %b %Y %H:%M:%S %z").to_string()
    } else {
        // Default format
        now.format("%a %b %d %H:%M:%S %Z %Y").to_string()
    };

    println!("{}", output);

    Ok(())
}

/// Format date with custom format string
fn format_date(date: &chrono::NaiveDateTime, format: &str) -> String {
    // Support common format specifiers
    // %Y - year, %m - month, %d - day
    // %H - hour, %M - minute, %S - second
    // %a - abbreviated weekday, %b - abbreviated month
    // %Z - timezone, %z - timezone offset

    // For now, use chrono's format
    // In production, would support more format options
    date.format(format).to_string()
}

/// Set system date
fn set_date(date_str: &str) -> Result<()> {
    // Parse date string
    // This is simplified - would need more robust parsing

    // Try to parse as ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        log::info!("Setting system time to: {}", dt);

        // Set system time (requires root privileges)
        // This would use clock_settime syscall
        eprintln!("date: setting time not yet implemented");
        return Ok(());
    }

    // Try to parse as Unix timestamp
    if let Ok(timestamp) = date_str.parse::<i64>() {
        let dt = DateTime::from_timestamp(timestamp, 0);
        if let Some(dt) = dt {
            log::info!("Setting system time to: {}", dt);
            eprintln!("date: setting time not yet implemented");
            return Ok(());
        }
    }

    anyhow::bail!("invalid date format: {}", date_str)
}
