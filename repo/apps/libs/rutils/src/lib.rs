// Copyright 2025 The Rustux Authors
//
// Rustica Utilities Library
//
// Common data structures, helper functions, and system integration
// for Rustica applications.

#![no_std]

extern crate alloc;

/// File path utilities
pub mod path;

/// System information utilities
pub mod sys;

/// String utilities
pub mod string;

/// Configuration utilities
pub mod config;

// Re-exports
pub use path::*;
pub use sys::*;
pub use string::*;
pub use config::*;
