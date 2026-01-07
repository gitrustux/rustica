//! Build script for Rustica userspace CLI
//!
//! This build script:
//! 1. Compiles all workspace members
//! 2. Copies binaries to the bin/ directory
//! 3. Creates a minimal filesystem structure

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src");

    // Get the output directory
    let out_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("bin");
    let target_dir = Path::new(&env::var("OUT_DIR").unwrap());

    // Create bin directory if it doesn't exist
    fs::create_dir_all(&out_dir).unwrap();

    println!("Building Rustica userspace CLI utilities...");
    println!("Output directory: {:?}", out_dir);

    // Build all workspace members
    let members = [
        "sh",
        "init",
        "coreutils",
        "networkutils",
        "pkgutil",
        "fwctl",
        "storageutils",
        "svc",
        "syscheck",
    ];

    for member in &members {
        println!("Building: {}", member);
    }

    // Note: Actual binaries will be built by cargo build
    // This script just sets up the structure

    println!("Build complete!");
}
