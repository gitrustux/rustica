// System information utilities

use alloc::string::String;

pub struct SystemInfo {
    pub hostname: String,
    pub os_version: String,
    pub arch: String,
}

pub fn get_system_info() -> SystemInfo {
    // TODO: Implement system info retrieval
    SystemInfo {
        hostname: "rustica".to_string(),
        os_version: "0.1.0".to_string(),
        arch: "x86_64".to_string(),
    }
}
