// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Package registry for tracking installed packages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::package::{PackageMetadata, PackageRef};
use crate::transaction::Transaction;
use crate::version::Version;

/// Package registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRegistry {
    /// Installed packages (name -> versions)
    #[serde(default)]
    pub packages: HashMap<String, Vec<Version>>,

    /// Active versions (name -> version)
    #[serde(default)]
    pub active: HashMap<String, Version>,

    /// Pending updates (packages downloaded but not activated)
    #[serde(default)]
    pub pending: Vec<PackageRef>,

    /// Transaction history
    #[serde(default)]
    pub transactions: Vec<Transaction>,
}

impl PackageRegistry {
    /// Create a new package registry
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            active: HashMap::new(),
            pending: Vec::new(),
            transactions: Vec::new(),
        }
    }

    /// Load the registry from disk
    pub fn load() -> crate::Result<Self> {
        let path = Self::registry_path();

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content)
                .map_err(|e| crate::Error::Serialization(e.to_string()))
        } else {
            Ok(Self::new())
        }
    }

    /// Save the registry to disk
    pub fn save(&self) -> crate::Result<()> {
        let path = Self::registry_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get the registry file path
    fn registry_path() -> PathBuf {
        PathBuf::from("/var/lib/rpg/registry.json")
    }

    /// Register a package
    pub fn register_package(&mut self, name: String, version: Version) {
        self.packages
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(version.clone());

        // Sort versions
        if let Some(versions) = self.packages.get_mut(&name) {
            versions.sort();
            versions.dedup();
        }
    }

    /// Unregister a package
    pub fn unregister_package(&mut self, name: &str, version: &Version) {
        if let Some(versions) = self.packages.get_mut(name) {
            versions.retain(|v| v != version);
        }

        // Remove from active if it was the active version
        if let Some(active) = self.active.get(name) {
            if active == version {
                self.active.remove(name);
            }
        }
    }

    /// Set the active version of a package
    pub fn set_active(&mut self, name: String, version: Version) {
        self.active.insert(name, version);
    }

    /// Get the active version of a package
    pub fn get_active(&self, name: &str) -> Option<&Version> {
        self.active.get(name)
    }

    /// Get all versions of a package
    pub fn get_versions(&self, name: &str) -> Option<&[Version]> {
        self.packages.get(name).map(|v| v.as_slice())
    }

    /// Check if a package is installed
    pub fn is_installed(&self, name: &str, version: &Version) -> bool {
        self.packages
            .get(name)
            .map(|versions| versions.contains(version))
            .unwrap_or(false)
    }

    /// Get all installed packages
    pub fn list_packages(&self) -> Vec<String> {
        self.packages.keys().cloned().collect()
    }

    /// Get system version
    pub fn get_system_version(&self) -> Option<&Version> {
        self.get_active("system")
    }

    /// Set system version
    pub fn set_system_version(&mut self, version: Version) {
        self.set_active("system".to_string(), version);
    }

    /// Add a pending update
    pub fn add_pending(&mut self, package_ref: PackageRef) {
        if !self.pending.contains(&package_ref) {
            self.pending.push(package_ref);
        }
    }

    /// Remove a pending update
    pub fn remove_pending(&mut self, package_ref: &PackageRef) {
        self.pending.retain(|p| p != package_ref);
    }

    /// Get all pending updates
    pub fn get_pending(&self) -> &[PackageRef] {
        &self.pending
    }

    /// Clear all pending updates
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Add a transaction to history
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);

        // Keep only last 100 transactions
        if self.transactions.len() > 100 {
            self.transactions = self.transactions
                .split_off(self.transactions.len() - 100);
        }
    }

    /// Get transaction history
    pub fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    /// Get recent transactions
    pub fn get_recent_transactions(&self, count: usize) -> &[Transaction] {
        let start = if self.transactions.len() > count {
            self.transactions.len() - count
        } else {
            0
        };
        &self.transactions[start..]
    }

    /// Get the latest transaction
    pub fn get_latest_transaction(&self) -> Option<&Transaction> {
        self.transactions.last()
    }

    /// Find a transaction by ID
    pub fn find_transaction(&self, id: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|t| t.id() == id)
    }

    /// Check if a kernel update is pending
    pub fn has_pending_kernel_update(&self) -> bool {
        self.pending.iter().any(|p| {
            self.packages.get(&p.name)
                .map(|versions| {
                    versions.iter().any(|v| {
                        // Check if this is a newer kernel version
                        p.name == "kernel" && v > &p.version
                    })
                })
                .unwrap_or(false)
        })
    }

    /// Get packages that need updates
    pub fn get_available_updates(&self, repo_metadata: &HashMap<String, Vec<PackageMetadata>>) -> Vec<PackageRef> {
        let mut updates = Vec::new();

        for (name, _versions) in &self.packages {
            if let Some(repo_versions) = repo_metadata.get(name) {
                let current = self.get_active(name);

                for metadata in repo_versions {
                    if let Some(current_version) = current {
                        if metadata.version > *current_version {
                            updates.push(PackageRef::new(
                                name.clone(),
                                metadata.version.clone(),
                            ));
                        }
                    }
                }
            }
        }

        updates
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        let total_packages = self.packages.len();
        let total_versions: usize = self.packages.values().map(|v| v.len()).sum();
        let active_count = self.active.len();
        let pending_count = self.pending.len();
        let transaction_count = self.transactions.len();

        RegistryStats {
            total_packages,
            total_versions,
            active_count,
            pending_count,
            transaction_count,
        }
    }

    /// Record a transaction (alias for add_transaction)
    pub fn record_transaction(&mut self, transaction: Transaction) {
        self.add_transaction(transaction);
    }

    /// Add a package version to the registry
    pub fn add_package(&mut self, name: &str, version: &Version) {
        self.packages
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(version.clone());

        // Sort and dedupe versions
        if let Some(versions) = self.packages.get_mut(name) {
            versions.sort();
            versions.dedup();
        }
    }

    /// List all versions of a package
    pub fn list_versions(&self, name: &str) -> Vec<Version> {
        self.packages
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    /// Remove the active version of a package
    pub fn remove_active(&mut self, name: &str) {
        self.active.remove(name);
    }
}

impl Default for PackageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of unique packages
    pub total_packages: usize,

    /// Total number of installed package versions
    pub total_versions: usize,

    /// Number of active packages
    pub active_count: usize,

    /// Number of pending updates
    pub pending_count: usize,

    /// Number of transactions in history
    pub transaction_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_basic() {
        let mut registry = PackageRegistry::new();

        registry.register_package("test".to_string(), Version::new(1, 0, 0));
        registry.set_active("test".to_string(), Version::new(1, 0, 0));

        assert!(registry.is_installed("test", &Version::new(1, 0, 0)));
        assert_eq!(
            registry.get_active("test"),
            Some(&Version::new(1, 0, 0))
        );
    }

    #[test]
    fn test_registry_versions() {
        let mut registry = PackageRegistry::new();

        registry.register_package("test".to_string(), Version::new(1, 0, 0));
        registry.register_package("test".to_string(), Version::new(1, 1, 0));
        registry.register_package("test".to_string(), Version::new(2, 0, 0));

        let versions = registry.get_versions("test").unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0], Version::new(1, 0, 0));
        assert_eq!(versions[1], Version::new(1, 1, 0));
        assert_eq!(versions[2], Version::new(2, 0, 0));
    }

    #[test]
    fn test_pending_updates() {
        let mut registry = PackageRegistry::new();

        let pkg_ref = PackageRef::new("test".to_string(), Version::new(2, 0, 0));
        registry.add_pending(pkg_ref.clone());

        assert_eq!(registry.get_pending().len(), 1);
        assert_eq!(registry.get_pending()[0], pkg_ref);

        registry.remove_pending(&pkg_ref);
        assert_eq!(registry.get_pending().len(), 0);
    }
}
