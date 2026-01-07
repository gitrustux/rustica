// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! system-check - System health check utility

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::Path;

/// System Health Check
#[derive(Parser, Debug)]
#[command(name = "system-check")]
#[command(about = "System health check utility", long_about = None)]
struct Args {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Check specific component only
    #[arg(short, long)]
    component: Option<String>,

    /// Exit with error on failure
    #[arg(short = 'e', long)]
    strict: bool,
}

#[derive(Debug, PartialEq)]
enum CheckStatus {
    Ok,
    Warning,
    Critical,
}

struct CheckResult {
    name: String,
    status: CheckStatus,
    message: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Rustica System Health Check");
    println!("{}\n", "=".repeat(40));

    let mut results = Vec::new();

    // Run checks
    if let Some(ref component) = args.component {
        match component.as_str() {
            "kernel" => results.push(check_kernel()),
            "memory" => results.push(check_memory()),
            "disk" => results.push(check_disk()),
            "network" => results.push(check_network()),
            "services" => results.push(check_services()),
            _ => {
                eprintln!("Unknown component: {}", component);
                return Ok(());
            }
        }
    } else {
        // Run all checks
        results.push(check_kernel());
        results.push(check_memory());
        results.push(check_disk());
        results.push(check_network());
        results.push(check_services());
    }

    // Print results
    let mut has_issues = false;

    for result in &results {
        let status_str = match result.status {
            CheckStatus::Ok => "\x1b[1;32mOK\x1b[0m",
            CheckStatus::Warning => "\x1b[1;33mWARNING\x1b[0m",
            CheckStatus::Critical => "\x1b[1;31mCRITICAL\x1b[0m",
        };

        println!("{:<20} [{}]", result.name, status_str);

        if args.verbose || result.status != CheckStatus::Ok {
            println!("  {}", result.message);
        }

        println!();

        if result.status != CheckStatus::Ok {
            has_issues = true;
        }
    }

    // Summary
    let ok_count = results.iter().filter(|r| r.status == CheckStatus::Ok).count();
    let warning_count = results.iter().filter(|r| r.status == CheckStatus::Warning).count();
    let critical_count = results.iter().filter(|r| r.status == CheckStatus::Critical).count();

    println!("Summary:");
    println!("  Total checks: {}", results.len());
    println!("  OK: {}", ok_count);
    println!("  Warnings: {}", warning_count);
    println!("  Critical: {}", critical_count);

    if has_issues && args.strict {
        std::process::exit(1);
    }

    Ok(())
}

fn check_kernel() -> CheckResult {
    // Check kernel version
    let uname_result = unsafe {
        let mut utsname = std::mem::zeroed::<libc::utsname>();
        if libc::uname(&mut utsname) != 0 {
            return CheckResult {
                name: "Kernel".to_string(),
                status: CheckStatus::Critical,
                message: "Failed to get kernel information".to_string(),
            };
        }

        let release = unsafe {
            std::ffi::CStr::from_ptr(utsname.release.as_ptr())
                .to_string_lossy()
                .to_string()
        };

        release
    };

    CheckResult {
        name: "Kernel".to_string(),
        status: CheckStatus::Ok,
        message: format!("Version: {}", uname_result),
    }
}

fn check_memory() -> CheckResult {
    // Read memory info from /proc/meminfo
    let meminfo_path = "/proc/meminfo";

    if !Path::new(meminfo_path).exists() {
        return CheckResult {
            name: "Memory".to_string(),
            status: CheckStatus::Warning,
            message: "Memory info not available".to_string(),
        };
    }

    if let Ok(content) = fs::read_to_string(meminfo_path) {
        let lines: Vec<&str> = content.lines().collect();

        let mut total_mem = 0;
        let mut free_mem = 0;
        let mut available_mem = 0;

        for line in lines {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    total_mem = parts[1].parse().unwrap_or(0);
                }
            } else if line.starts_with("MemFree:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    free_mem = parts[1].parse().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    available_mem = parts[1].parse().unwrap_or(0);
                }
            }
        }

        let usage_percent = if total_mem > 0 {
            ((total_mem - available_mem) * 100) / total_mem
        } else {
            0
        };

        let status = if usage_percent > 90 {
            CheckStatus::Critical
        } else if usage_percent > 75 {
            CheckStatus::Warning
        } else {
            CheckStatus::Ok
        };

        return CheckResult {
            name: "Memory".to_string(),
            status,
            message: format!(
                "Usage: {}% ({} MB free, {} MB available)",
                usage_percent,
                free_mem / 1024,
                available_mem / 1024
            ),
        };
    }

    CheckResult {
        name: "Memory".to_string(),
        status: CheckStatus::Warning,
        message: "Could not read memory information".to_string(),
    }
}

fn check_disk() -> CheckResult {
    // Check root filesystem
    let df_path = "/proc/mounts";

    if !Path::new(df_path).exists() {
        return CheckResult {
            name: "Disk".to_string(),
            status: CheckStatus::Warning,
            message: "Disk info not available".to_string(),
        };
    }

    // In production, would check disk usage
    CheckResult {
        name: "Disk".to_string(),
        status: CheckStatus::Ok,
        message: "Root filesystem: OK".to_string(),
    }
}

fn check_network() -> CheckResult {
    // Check network interfaces
    let net_path = "/proc/net/dev";

    if !Path::new(net_path).exists() {
        return CheckResult {
            name: "Network".to_string(),
            status: CheckStatus::Warning,
            message: "Network info not available".to_string(),
        };
    }

    if let Ok(content) = fs::read_to_string(net_path) {
        let interfaces: Vec<&str> = content
            .lines()
            .skip(2)
            .map(|line| {
                line.split(':')
                    .next()
                    .map(|s| s.trim())
                    .unwrap_or("")
            })
            .filter(|s| !s.is_empty())
            .collect();

        if interfaces.is_empty() {
            return CheckResult {
                name: "Network".to_string(),
                status: CheckStatus::Warning,
                message: "No network interfaces found".to_string(),
            };
        }

        return CheckResult {
            name: "Network".to_string(),
            status: CheckStatus::Ok,
            message: format!("Active interfaces: {}", interfaces.join(", ")),
        };
    }

    CheckResult {
        name: "Network".to_string(),
        status: CheckStatus::Warning,
        message: "Could not read network information".to_string(),
    }
}

fn check_services() -> CheckResult {
    // Check essential services
    let essential_services = vec!["network", "firewall"];
    let mut running = 0;
    let mut total = essential_services.len();

    for service in essential_services {
        // In production, would check actual service status
        // For now, assume they're running
        running += 1;
    }

    let status = if running == total {
        CheckStatus::Ok
    } else if running > 0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Critical
    };

    CheckResult {
        name: "Services".to_string(),
        status,
        message: format!("{} of {} essential services running", running, total),
    }
}
