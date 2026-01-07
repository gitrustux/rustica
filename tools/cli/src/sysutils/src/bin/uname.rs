// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! uname - Print system information

use anyhow::Result;
use clap::Parser;

/// Print system information
#[derive(Parser, Debug)]
#[command(name = "uname")]
#[command(about = "Print system information", long_about = None)]
struct Args {
    /// Print all information
    #[arg(short, long)]
    all: bool,

    /// Print kernel name
    #[arg(short = 's', long)]
    kernel_name: bool,

    /// Print nodename (network node hostname)
    #[arg(short = 'n', long)]
    nodename: bool,

    /// Print kernel release
    #[arg(short = 'r', long)]
    kernel_release: bool,

    /// Print kernel version
    #[arg(short = 'v', long)]
    kernel_version: bool,

    /// Print machine hardware name
    #[arg(short = 'm', long)]
    machine: bool,

    /// Print processor type
    #[arg(short = 'p', long)]
    processor: bool,

    /// Print hardware platform
    #[arg(short = 'i', long)]
    hardware_platform: bool,

    /// Print operating system
    #[arg(short = 'o', long)]
    operating_system: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Default is -s (kernel name)
    let print_all = args.all;
    let print_kernel_name = args.kernel_name || args.all;
    let print_nodename = args.nodename || args.all;
    let print_release = args.kernel_release || args.all;
    let print_version = args.kernel_version || args.all;
    let print_machine = args.machine || args.all;
    let print_processor = args.processor || args.all;
    let print_hardware = args.hardware_platform || args.all;
    let print_os = args.operating_system || args.all;

    // If no flags specified, default to kernel name
    let default = !args.all
        && !args.kernel_name
        && !args.nodename
        && !args.kernel_release
        && !args.kernel_version
        && !args.machine
        && !args.processor
        && !args.hardware_platform
        && !args.operating_system;

    // Get system information
    let kernel_name = "Rustux";
    let nodename = get_hostname();
    let kernel_release = "0.1.0";
    let kernel_version = "0.0.1";
    let machine = get_machine();
    let processor = "unknown";
    let hardware_platform = "unknown";
    let operating_system = "Rustica OS";

    // Build output
    let mut parts = Vec::new();

    if default || print_kernel_name {
        parts.push(kernel_name.to_string());
    }

    if print_nodename {
        parts.push(nodename);
    }

    if print_release {
        parts.push(kernel_release.to_string());
    }

    if print_version {
        parts.push(kernel_version.to_string());
    }

    if print_machine {
        parts.push(machine.to_string());
    }

    if print_processor {
        parts.push(processor.to_string());
    }

    if print_hardware {
        parts.push(hardware_platform.to_string());
    }

    if print_os {
        parts.push(operating_system.to_string());
    }

    println!("{}", parts.join(" "));

    Ok(())
}

/// Get system hostname
fn get_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|h| h.trim().to_string())
        .unwrap_or_else(|| {
            // Fallback to system hostname
            "rustica".to_string()
        })
}

/// Get machine architecture
fn get_machine() -> String {
    // Read from /proc/cpuinfo or use compile-time target
    #[cfg(target_arch = "x86_64")]
    return "x86_64".to_string();

    #[cfg(target_arch = "aarch64")]
    return "aarch64".to_string();

    #[cfg(target_arch = "riscv64")]
    return "riscv64".to_string();

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    return "unknown".to_string();
}
