// Copyright 2025 The Rustux Authors
//
// Aurora GUI Library
//
// Widget library, theme system, and rendering utilities
// for the Aurora Desktop Environment.

#![no_std]

extern crate alloc;

/// Widget types and components
pub mod widget;

/// Theme system
pub mod theme;

/// Rendering utilities
pub mod render;

// Re-exports
pub use widget::*;
pub use theme::*;
pub use render::*;
