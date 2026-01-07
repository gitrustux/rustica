//! Build script for CLI applications

fn main() {
    println!("cargo:rerun-if-changed=../");
    println!("Building Rustica CLI applications...");
}
