// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Rustica Core Utilities Library
//!
//! Common utilities and types for coreutils commands.

pub mod file_utils;
pub mod output;

pub use file_utils::*;
pub use output::*;
