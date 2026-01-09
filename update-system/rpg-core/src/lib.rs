// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! RPG (Rust Package Manager) Core Library
//!
//! This library provides the core functionality for package management
//! in the Rustica Operating System, including:
//!
//! - Versioned filesystem layout
//! - Atomic package activation
//! - Rollback support
//! - Cryptographic package signing
//! - Background updates
//!
//! # Architecture
//!
//! The package manager is designed around the following principles:
//!
//! 1. **Never overwrite active files**: All packages are installed to
//!    versioned directories and activated via atomic symlink switches.
//!
//! 2. **Full rollback support**: Previous versions are retained until
//!    explicitly removed.
//!
//! 3. **Live updates**: Application and userland updates can occur
//!    while the system is running.
//!
//! 4. **Safe kernel updates**: Kernel updates are installed alongside
//!    the existing kernel and activated on next reboot.

pub mod config;
pub mod layout;
pub mod package;
pub mod signature;
pub mod symlink;
pub mod transaction;
pub mod version;
pub mod registry;
pub mod sources;
pub mod fetch;
pub mod ops;
pub mod archive;

// Re-exports
pub use config::{Config, UpdateConfig};
pub use layout::{SystemLayout, AppLayout, LayoutManager};
pub use package::{Package, PackageKind, PackageMetadata, PackageState};
pub use signature::{Signature, SignatureVerifier, SigningKey};
pub use symlink::{Symlink, atomic_symlink_swap};
pub use transaction::{Transaction, TransactionKind, TransactionResult};
pub use version::{Version, VersionConstraint};
pub use sources::{Source, SourcesConfig, SourcesStats};
pub use fetch::{FetchError, FetchOptions, fetch_file, fetch_index};
pub use ops::{PackageManager, UpdateInfo, PackageUpdate, UpdateResult, SystemStatus, InstalledPackage};
pub use archive::{PackageArchive, PackageManifest, create_package};

/// Result type for RPG operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during package operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Fetch error
    #[error("Fetch error: {0}")]
    Fetch(#[from] fetch::FetchError),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureVerification(String),

    /// Package not found
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    /// Version not found
    #[error("Version not found: {0}")]
    VersionNotFound(String),

    /// Invalid version format
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Rollback failed
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    /// Layout error
    #[error("Layout error: {0}")]
    Layout(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}
