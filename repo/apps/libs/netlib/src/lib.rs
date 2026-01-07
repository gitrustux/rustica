// Copyright 2025 The Rustux Authors
//
// Rustica Networking Library
//
// HTTP client wrappers, DNS resolution, and network protocols
// for Rustica applications.

#![no_std]

extern crate alloc;

/// HTTP client utilities
pub mod http;

/// DNS resolution
pub mod dns;

/// Network protocol helpers
pub mod protocol;

// Re-exports
pub use http::*;
pub use dns::*;
pub use protocol::*;
