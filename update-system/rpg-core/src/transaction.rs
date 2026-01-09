// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Transaction management for atomic package operations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::package::{Package, PackageKind};
use crate::symlink::atomic_symlink_swap_with_rollback;
use crate::version::Version;

/// Transaction kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionKind {
    /// Install a package
    Install,
    /// Remove a package
    Remove,
    /// Upgrade a package
    Upgrade,
    /// Rollback a package
    Rollback,
    /// Switch system version
    SwitchSystem,
}

/// Transaction state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionState {
    /// Transaction is prepared but not started
    Prepared,
    /// Transaction is in progress
    InProgress,
    /// Transaction completed successfully
    Completed,
    /// Transaction failed
    Failed,
    /// Transaction was rolled back
    RolledBack,
}

/// A transaction for package operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: String,

    /// Transaction kind
    pub kind: TransactionKind,

    /// Transaction state
    pub state: TransactionState,

    /// Packages affected by this transaction
    pub packages: Vec<Package>,

    /// Rollback information
    #[serde(default)]
    pub rollback_info: RollbackInfo,

    /// Transaction timestamp
    pub created_at: i64,

    /// Error message if transaction failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Rollback information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollbackInfo {
    /// Previous system version (for system switches)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_system_version: Option<Version>,

    /// Previous app versions (name -> version)
    #[serde(default)]
    pub previous_app_versions: Vec<(String, Version)>,

    /// Symlink targets before transaction
    #[serde(default)]
    pub previous_symlinks: Vec<(PathBuf, PathBuf)>,
}

/// Result of a transaction operation
#[derive(Debug, Clone)]
pub enum TransactionResult {
    /// Transaction succeeded
    Success {
        /// Activated packages
        activated: Vec<String>,
        /// Packages requiring reboot
        requires_reboot: Vec<String>,
    },
    /// Transaction failed
    Failed {
        /// Error message
        error: String,
        /// Packages that were partially installed
        partial: Vec<String>,
    },
    /// Transaction was rolled back
    RolledBack {
        /// Reason for rollback
        reason: String,
    },
}

impl Transaction {
    /// Create a new transaction
    pub fn new(kind: TransactionKind, packages: Vec<Package>) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            kind,
            state: TransactionState::Prepared,
            packages,
            rollback_info: RollbackInfo::default(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            error: None,
        }
    }

    /// Execute the transaction
    pub async fn execute(&mut self) -> TransactionResult {
        self.state = TransactionState::InProgress;

        match self.kind {
            TransactionKind::Install => self.install(),
            TransactionKind::Remove => self.remove(),
            TransactionKind::Upgrade => self.upgrade(),
            TransactionKind::Rollback => self.rollback(),
            TransactionKind::SwitchSystem => self.switch_system(),
        }
    }

    /// Install packages
    fn install(&mut self) -> TransactionResult {
        let mut activated = Vec::new();
        let mut requires_reboot = Vec::new();
        let mut partial = Vec::new();

        // Collect package names first to avoid borrow issues
        let package_names: Vec<String> = self.packages.iter().map(|p| p.name().to_string()).collect();

        for (idx, name) in package_names.iter().enumerate() {
            match self.install_package(idx) {
                Ok(Some(reboot)) => {
                    if reboot {
                        requires_reboot.push(name.clone());
                    } else {
                        activated.push(name.clone());
                    }
                }
                Ok(None) => {
                    // Package already installed
                }
                Err(e) => {
                    partial.push(name.clone());
                    self.state = TransactionState::Failed;
                    self.error = Some(e.to_string());
                }
            }
        }

        if partial.is_empty() {
            self.state = TransactionState::Completed;
            TransactionResult::Success {
                activated,
                requires_reboot,
            }
        } else {
            TransactionResult::Failed {
                error: self.error.clone().unwrap_or_default(),
                partial,
            }
        }
    }

    /// Install a single package
    fn install_package(&mut self, idx: usize) -> crate::Result<Option<bool>> {
        use crate::layout::{AppLayout, SystemLayout};

        let requires_reboot = self.packages[idx].kind().requires_reboot();

        match self.packages[idx].kind() {
            PackageKind::App => {
                let layout = AppLayout::new();
                let version_str = self.packages[idx].version().as_str();
                let app_path = layout.version_path(self.packages[idx].name(), &version_str);

                // Create version directory
                std::fs::create_dir_all(&app_path)?;

                // Extract package (stub for now)
                // In production, would extract from archive

                // Update metadata
                let metadata_path = layout.metadata_path(self.packages[idx].name(), &version_str);
                let metadata_json = serde_json::to_string_pretty(&self.packages[idx].metadata)
                    .map_err(|e| crate::Error::Serialization(e.to_string()))?;
                std::fs::write(&metadata_path, metadata_json)?;

                // Activate if not requiring reboot
                if !requires_reboot {
                    let current_path = layout.current_path(self.packages[idx].name());
                    let old_target = atomic_symlink_swap_with_rollback(&current_path, &app_path)?;

                    if let Some(old) = old_target {
                        if let Some(old_version) = old.file_name().and_then(|s| s.to_str()) {
                            self.rollback_info.previous_app_versions.push((
                                self.packages[idx].name().to_string(),
                                Version::parse(old_version)?,
                            ));
                        }
                    }
                }

                self.packages[idx].set_state(crate::package::PackageState::Active);
                Ok(Some(requires_reboot))
            }
            PackageKind::Kernel | PackageKind::System => {
                let layout = SystemLayout::new();
                let version_str = format!("v{}", self.packages[idx].version().as_str());
                let version_path = layout.version_path(&version_str);

                // Create version directory
                std::fs::create_dir_all(&version_path)?;

                // Update metadata
                let metadata_path = layout.metadata_path(&version_str);
                let metadata_json = serde_json::to_string_pretty(&self.packages[idx].metadata)
                    .map_err(|e| crate::Error::Serialization(e.to_string()))?;
                std::fs::write(&metadata_path, metadata_json)?;

                // Mark as pending (requires reboot)
                self.packages[idx].set_state(crate::package::PackageState::Pending);

                Ok(Some(true))
            }
            _ => Ok(Some(false)),
        }
    }

    /// Remove packages
    fn remove(&mut self) -> TransactionResult {
        let mut activated = Vec::new();
        let packages_to_remove: Vec<_> = self.packages.iter().map(|p| p.clone()).collect();

        for package in &packages_to_remove {
            match self.remove_package(package) {
                Ok(_) => {
                    activated.push(package.name().to_string());
                }
                Err(e) => {
                    self.state = TransactionState::Failed;
                    self.error = Some(e.to_string());
                    return TransactionResult::Failed {
                        error: e.to_string(),
                        partial: activated,
                    };
                }
            }
        }

        self.state = TransactionState::Completed;
        TransactionResult::Success {
            activated,
            requires_reboot: Vec::new(),
        }
    }

    /// Remove a single package
    fn remove_package(&mut self, package: &Package) -> crate::Result<()> {
        use crate::layout::AppLayout;

        match package.kind() {
            PackageKind::App => {
                let layout = AppLayout::new();

                // Don't remove the active version
                if let Some(current) = layout.current_version(package.name())? {
                    if current == package.version().as_str() {
                        // Switch to another version first
                        let versions = layout.list_versions(package.name())?;
                        if versions.len() <= 1 {
                            return Err(crate::Error::Other(
                                "Cannot remove only version of app".into(),
                            ));
                        }
                    }
                }

                // Remove the version directory
                let version_path = layout.version_path(package.name(), &package.version().as_str());
                if version_path.exists() {
                    std::fs::remove_dir_all(&version_path)?;
                }

                Ok(())
            }
            _ => Err(crate::Error::Other(
                "Cannot remove system packages".into(),
            )),
        }
    }

    /// Upgrade packages
    fn upgrade(&mut self) -> TransactionResult {
        // For now, upgrade is implemented as install + switch
        self.install()
    }

    /// Rollback to previous version
    fn rollback(&mut self) -> TransactionResult {
        let mut activated = Vec::new();
        let mut requires_reboot = Vec::new();

        // Rollback each package
        for (name, version) in &self.rollback_info.previous_app_versions {
            use crate::layout::AppLayout;

            let layout = AppLayout::new();
            let version_path = layout.version_path(name, &version.as_str());
            let current_path = layout.current_path(name);

            if let Err(e) = atomic_symlink_swap_with_rollback(&current_path, &version_path) {
                self.state = TransactionState::Failed;
                self.error = Some(e.to_string());
                return TransactionResult::Failed {
                    error: e.to_string(),
                    partial: activated,
                };
            }

            activated.push(name.clone());
        }

        // Rollback system version if needed
        if let Some(ref version) = self.rollback_info.previous_system_version {
            use crate::layout::SystemLayout;

            let layout = SystemLayout::new();
            let version_path = layout.version_path(&format!("v{}", version.as_str()));
            let current_path = layout.current_path();

            if let Err(e) = atomic_symlink_swap_with_rollback(&current_path, &version_path) {
                self.state = TransactionState::Failed;
                self.error = Some(e.to_string());
                return TransactionResult::Failed {
                    error: e.to_string(),
                    partial: activated,
                };
            }

            requires_reboot.push("system".to_string());
        }

        self.state = TransactionState::Completed;
        TransactionResult::Success {
            activated,
            requires_reboot,
        }
    }

    /// Switch to a new system version
    fn switch_system(&mut self) -> TransactionResult {
        use crate::layout::SystemLayout;

        if self.packages.is_empty() {
            self.state = TransactionState::Failed;
            self.error = Some("No system version specified".into());
            return TransactionResult::Failed {
                error: "No system version specified".into(),
                partial: Vec::new(),
            };
        }

        let package = &self.packages[0];
        let layout = SystemLayout::new();
        let version_str = format!("v{}", package.version().as_str());
        let version_path = layout.version_path(&version_str);
        let current_path = layout.current_path();

        // Store current version for rollback
        if let Ok(Some(current)) = layout.current_version() {
            match Version::parse(&current) {
                Ok(version) => {
                    self.rollback_info.previous_system_version = Some(version);
                }
                Err(_) => {
                    // Continue anyway if we can't parse the version
                }
            }
        }

        match atomic_symlink_swap_with_rollback(&current_path, &version_path) {
            Ok(_) => {
                self.state = TransactionState::Completed;
                TransactionResult::Success {
                    activated: vec!["system".to_string()],
                    requires_reboot: vec!["system".to_string()],
                }
            }
            Err(e) => {
                self.state = TransactionState::Failed;
                self.error = Some(e.to_string());
                TransactionResult::Failed {
                    error: e.to_string(),
                    partial: Vec::new(),
                }
            }
        }
    }

    /// Check if the transaction is reversible
    pub fn can_rollback(&self) -> bool {
        !self.rollback_info.previous_app_versions.is_empty() ||
            self.rollback_info.previous_system_version.is_some()
    }

    /// Get the transaction ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{Package, PackageKind};
    use crate::signature::SigningKey;
    use crate::version::Version;

    #[test]
    fn test_transaction_creation() {
        let key = SigningKey::generate();
        let signature = key.sign(b"test");
        let version = Version::new(1, 0, 0);

        let metadata = crate::package::PackageMetadata::new(
            "test".to_string(),
            version.clone(),
            PackageKind::App,
            1024,
            "0".repeat(64),
            signature,
            "https://example.com/test.rpg".to_string(),
        );

        let package = Package::new(metadata);
        let tx = Transaction::new(TransactionKind::Install, vec![package]);

        assert_eq!(tx.state, TransactionState::Prepared);
        assert!(!tx.id.is_empty());
    }
}
