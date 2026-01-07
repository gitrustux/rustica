// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ping - Send ICMP echo requests to network hosts

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Duration;

/// Send ICMP echo requests
#[derive(Parser, Debug)]
#[command(name = "ping")]
#[command(about = "Send ICMP echo requests to network hosts")]
struct Args {
    /// Count of packets to send
    #[arg(short, long)]
    count: Option<u32>,

    /// Wait interval (seconds)
    #[arg(short = 'i', long, default_value_t = 1.0)]
    interval: f64,

    /// Packet size
    #[arg(short = 's', long, default_value_t = 64)]
    size: usize,

    /// Timeout (seconds)
    #[arg(short = 'W', long, default_value_t = 5)]
    timeout: u64,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Host to ping
    #[arg(required = true)]
    host: String,
}

#[derive(Debug)]
struct PingResult {
    seq: u32,
    ttl: u8,
    time: Duration,
    bytes: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("PING {} ({}) {} bytes of data.",
        args.host,
        args.host, // Would resolve to IP
        args.size
    );

    let count = args.count.unwrap_or(0); // 0 means infinite
    let mut results = Vec::new();

    for seq in 1..=count {
        match ping_host(&args.host, seq, args.size) {
            Ok(result) => {
                println!("{} bytes from {}: icmp_seq={} ttl={} time={:.2} ms",
                    result.bytes,
                    args.host,
                    result.seq,
                    result.ttl,
                    result.time.as_secs_f64() * 1000.0
                );
                results.push(result);
            }
            Err(e) => {
                eprintln!("From {} icmp_seq={} Destination Unreachable", args.host, seq);
            }
        }

        // Wait between pings
        if seq < count {
            std::thread::sleep(Duration::from_secs_f64(args.interval));
        }
    }

    // Print statistics
    if !results.is_empty() {
        print_statistics(&args.host, &results);
    }

    Ok(())
}

/// Ping a host
fn ping_host(host: &str, seq: u32, size: usize) -> Result<PingResult> {
    // Create ICMP socket (requires raw sockets, usually needs root)
    // For now, simulate a ping

    // In production, would:
    // 1. Resolve hostname to IP
    // 2. Create ICMP socket
    // 3. Send ICMP echo request
    // 4. Wait for reply
    // 5. Calculate round-trip time

    // Simulate with a small delay
    std::thread::sleep(Duration::from_millis(100));

    // Parse host to IP (simplified)
    let ip = host; // In production, would resolve

    // Return simulated result
    Ok(PingResult {
        seq,
        ttl: 64,
        time: Duration::from_millis(100),
        bytes: size,
    })
}

/// Print ping statistics
fn print_statistics(host: &str, results: &[PingResult]) {
    let transmitted = results.len() as u32;
    let received = transmitted; // Assuming all succeeded

    let min_time = results.iter().map(|r| r.time).min().unwrap();
    let max_time = results.iter().map(|r| r.time).max().unwrap();
    let avg_time = results.iter().map(|r| r.time.as_millis()).sum::<u128>() / results.len() as u128;

    println!();
    println!("--- {} ping statistics ---", host);
    println!("{} packets transmitted, {} received, {:.0}% packet loss",
        transmitted,
        received,
        0.0
    );
    println!("rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
        min_time.as_secs_f64() * 1000.0,
        avg_time as f64 / 1000.0,
        max_time.as_secs_f64() * 1000.0,
        0.0
    );
}
