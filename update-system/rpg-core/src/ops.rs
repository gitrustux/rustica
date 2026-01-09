// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! High-level package operations

use crate::archive::PackageArchive;
use crate::fetch::{self, FetchError};
use crate::package::{Package, PackageKind, PackageMetadata};
use crate::registry::PackageRegistry;
use crate::sources::{Source, SourcesConfig};
use crate::transaction::{Transaction, TransactionKind, TransactionResult};
use crate::version::Version;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Package manager for high-level operations
#[derive(Debug, Clone)]
pub struct PackageManager {
    /// Sources configuration
    sources: Arc<RwLock<SourcesConfig>>,
    /// Package registry
    registry: Arc<RwLock<PackageRegistry>>,
    /// Download cache directory
    cache_dir: PathBuf,
    /// Temporary directory for downloads
    temp_dir: PathBuf,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new() -> crate::Result<Self> {
        let cache_dir = PathBuf::from("/var/cache/rpg");
        let temp_dir = PathBuf::from("/tmp/rpg");

        // Create directories if they don't exist
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            sources: Arc::new(RwLock::new(SourcesConfig::load()?)),
            registry: Arc::new(RwLock::new(PackageRegistry::load().unwrap_or_default())),
            cache_dir,
            temp_dir,
        })
    }

    /// Check for updates
    pub async fn check_updates(&self) -> crate::Result<UpdateInfo> {
        let sources = self.sources.read().await;
        let mut updates = Vec::new();
        let mut errors = Vec::new();

        // Fetch indices from all enabled sources
        let enabled_sources = sources.enabled_sources();
        if enabled_sources.is_empty() {
            return Ok(UpdateInfo {
                available: Vec::new(),
                errors: vec!["No enabled sources found".to_string()],
            });
        }

        // Fetch kernel updates
        let kernel_sources: Vec<&Source> = sources.kernel_sources();
        if !kernel_sources.is_empty() {
            match fetch::fetch_index(&kernel_sources, None).await {
                Ok(index) => {
                    for entry in index.packages {
                        if let Some(update) = self.check_package_update(&entry).await? {
                            updates.push(update);
                        }
                    }
                }
                Err(e) => errors.push(format!("Failed to fetch kernel index: {}", e)),
            }
        }

        // Fetch system updates
        let system_sources: Vec<&Source> = sources.system_sources();
        if !system_sources.is_empty() {
            match fetch::fetch_index(&system_sources, None).await {
                Ok(index) => {
                    for entry in index.packages {
                        if let Some(update) = self.check_package_update(&entry).await? {
                            updates.push(update);
                        }
                    }
                }
                Err(e) => errors.push(format!("Failed to fetch system index: {}", e)),
            }
        }

        // Fetch app updates
        let app_sources: Vec<&Source> = sources.app_sources();
        if !app_sources.is_empty() {
            match fetch::fetch_index(&app_sources, None).await {
                Ok(index) => {
                    for entry in index.packages {
                        if let Some(update) = self.check_package_update(&entry).await? {
                            updates.push(update);
                        }
                    }
                }
                Err(e) => errors.push(format!("Failed to fetch app index: {}", e)),
            }
        }

        Ok(UpdateInfo {
            available: updates,
            errors,
        })
    }

    /// Check if a package has an update available
    async fn check_package_update(&self, entry: &fetch::PackageEntry) -> crate::Result<Option<PackageUpdate>> {
        let registry = self.registry.read().await;
        let current_version = registry.get_active(&entry.name);

        if let Some(current) = current_version {
            let new_version = Version::parse(&entry.version)?;
            if new_version > *current {
                Ok(Some(PackageUpdate {
                    name: entry.name.clone(),
                    current_version: current.to_string(),
                    new_version: entry.version.clone(),
                    size: entry.size,
                    kind: self.infer_package_kind(&entry.name),
                }))
            } else {
                Ok(None)
            }
        } else {
            // Package not installed, but available
            Ok(Some(PackageUpdate {
                name: entry.name.clone(),
                current_version: "not installed".to_string(),
                new_version: entry.version.clone(),
                size: entry.size,
                kind: self.infer_package_kind(&entry.name),
            }))
        }
    }

    /// Infer package kind from name
    fn infer_package_kind(&self, name: &str) -> PackageKind {
        if name == "kernel" {
            PackageKind::Kernel
        } else if name == "system" {
            PackageKind::System
        } else {
            PackageKind::App
        }
    }

    /// Download a package
    pub async fn download_package(
        &self,
        name: &str,
        version: &str,
        kind: PackageKind,
    ) -> crate::Result<PathBuf> {
        let sources = self.sources.read().await;

        let sources_for_type = match kind {
            PackageKind::Kernel => sources.kernel_sources(),
            PackageKind::System => sources.system_sources(),
            PackageKind::App | PackageKind::Boot => sources.app_sources(),
        };

        if sources_for_type.is_empty() {
            return Err(crate::Error::Other(format!(
                "No sources configured for package type: {:?}",
                kind
            )));
        }

        // First fetch the index to get checksum
        let index = fetch::fetch_index(&sources_for_type, None).await?;

        let entry = index
            .packages
            .iter()
            .find(|p| p.name == name && p.version == version)
            .ok_or_else(|| crate::Error::PackageNotFound(format!("{}@{}", name, version)))?;

        // Download package
        let package_path = self.cache_dir.join(format!("{}-{}.rpg", name, version));

        let result = fetch::fetch_package(
            &sources_for_type,
            name,
            version,
            &entry.sha256,
            &package_path,
            None,
            None,
        )
        .await
        .map_err(|e| match e {
            FetchError::AllSourcesFailed => {
                crate::Error::NetworkError("All sources failed".to_string())
            }
            FetchError::ChecksumMismatch { expected, actual } => {
                crate::Error::Other(format!(
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                ))
            }
            _ => crate::Error::NetworkError(e.to_string()),
        })
        .map_err(crate::Error::from)?;

        Ok(result.path)
    }

    /// Install a package
    pub async fn install_package(
        &self,
        name: &str,
        version: Option<&str>,
        kind: PackageKind,
    ) -> crate::Result<TransactionResult> {
        // If version not specified, fetch latest
        let version_to_install = if let Some(v) = version {
            v.to_string()
        } else {
            self.get_latest_version(name, kind).await?
        };

        // Download package
        let package_path = self
            .download_package(name, &version_to_install, kind)
            .await?;

        // Open package archive
        let archive = PackageArchive::open(&package_path)?;
        let metadata = archive.metadata.clone();

        // Extract package files to versioned directory
        use crate::layout::{AppLayout, SystemLayout};

        let version_str = metadata.version.as_str();
        let extract_path = match kind {
            PackageKind::App => {
                let layout = AppLayout::new();
                layout.version_path(name, &version_str)
            }
            PackageKind::Kernel | PackageKind::System | PackageKind::Boot => {
                let layout = SystemLayout::new();
                layout.version_path(&format!("v{}", version_str))
            }
        };

        // Extract files
        archive.extract_files(&extract_path)?;

        // Create transaction
        let package = Package::new(metadata.clone());
        let mut transaction = Transaction::new(TransactionKind::Install, vec![package]);

        // Execute transaction (handles symlink activation)
        let result = transaction.execute().await;

        // Update registry if successful
        if matches!(result, TransactionResult::Success { .. }) {
            let mut registry = self.registry.write().await;
            registry.record_transaction(transaction.clone());
            registry.add_package(name, &Version::parse(&version_to_install)?);
            registry.set_active(name.to_string(), Version::parse(&version_to_install)?);
            let _ = registry.save();
        }

        Ok(result)
    }

    /// Update all packages
    pub async fn update_all(&self) -> crate::Result<UpdateResult> {
        let update_info = self.check_updates().await?;

        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        let mut requires_reboot = Vec::new();

        for update in &update_info.available {
            match self
                .install_package(&update.name, Some(&update.new_version), update.kind.clone())
                .await
            {
                Ok(TransactionResult::Success {
                    activated,
                    requires_reboot: reboot,
                }) => {
                    succeeded.push(update.name.clone());
                    requires_reboot.extend(reboot);
                    if activated.contains(&update.name) {
                        println!("Updated {} to {}", update.name, update.new_version);
                    }
                }
                Ok(TransactionResult::Failed { error, .. }) => {
                    failed.push((update.name.clone(), error));
                }
                Ok(TransactionResult::RolledBack { reason, .. }) => {
                    failed.push((update.name.clone(), reason));
                }
                Err(e) => {
                    failed.push((update.name.clone(), e.to_string()));
                }
            }
        }

        Ok(UpdateResult {
            succeeded,
            failed,
            requires_reboot,
        })
    }

    /// Rollback to a previous version
    pub async fn rollback(&self, package: &str, version: Option<&str>) -> crate::Result<TransactionResult> {
        let registry = self.registry.read().await;

        let rollback_version = if let Some(v) = version {
            Version::parse(v)?
        } else {
            // Get previous version
            let versions = registry.list_versions(package);
            if versions.len() < 2 {
                return Err(crate::Error::Other(
                    "No previous version to rollback to".to_string(),
                ));
            }
            versions[1].clone()
        };

        // Create rollback transaction
        let mut transaction = Transaction::new(TransactionKind::Rollback, vec![]);
        transaction.rollback_info.previous_app_versions.push((
            package.to_string(),
            rollback_version.clone(),
        ));

        let result = transaction.execute().await;

        // Update registry if successful
        if matches!(result, TransactionResult::Success { .. }) {
            drop(registry);
            let mut registry = self.registry.write().await;
            registry.set_active(package.to_string(), rollback_version);
            let _ = registry.save();
        }

        Ok(result)
    }

    /// Get latest version of a package
    async fn get_latest_version(&self, name: &str, kind: PackageKind) -> crate::Result<String> {
        let sources = self.sources.read().await;

        let sources_for_type = match kind {
            PackageKind::Kernel => sources.kernel_sources(),
            PackageKind::System => sources.system_sources(),
            PackageKind::App | PackageKind::Boot => sources.app_sources(),
        };

        let index = fetch::fetch_index(&sources_for_type, None).await?;

        let entry = index
            .packages
            .iter()
            .filter(|p| p.name == name)
            .max_by_key(|p| &p.version)
            .ok_or_else(|| crate::Error::PackageNotFound(name.to_string()))?;

        Ok(entry.version.clone())
    }

    /// Get system status
    pub async fn get_status(&self) -> crate::Result<SystemStatus> {
        let registry = self.registry.read().await;
        let sources = self.sources.read().await;

        let stats = registry.stats();
        let source_stats = sources.stats();

        Ok(SystemStatus {
            total_packages: stats.total_packages,
            active_packages: stats.active_count,
            pending_updates: stats.pending_count,
            sources_total: source_stats.total,
            sources_enabled: source_stats.enabled,
        })
    }

    /// List installed packages
    pub async fn list_installed(&self) -> crate::Result<Vec<InstalledPackage>> {
        let registry = self.registry.read().await;

        let mut packages = Vec::new();

        for (name, versions) in &registry.packages {
            if let Some(active) = registry.get_active(name) {
                packages.push(InstalledPackage {
                    name: name.clone(),
                    version: active.to_string(),
                    versions: versions.iter().map(|v| v.to_string()).collect(),
                    kind: self.infer_package_kind(name),
                });
            }
        }

        packages.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(packages)
    }

    /// Remove a package
    pub async fn remove_package(&self, name: &str) -> crate::Result<TransactionResult> {
        // Get package metadata
        let registry = self.registry.read().await;
        let version = registry
            .get_active(name)
            .ok_or_else(|| crate::Error::PackageNotFound(name.to_string()))?;

        let kind = self.infer_package_kind(name);

        // Create metadata for removal
        let metadata = PackageMetadata::new(
            name.to_string(),
            version.clone(),
            kind,
            0,
            "0".repeat(64),
            crate::signature::PackageSignature::new([0u8; 64]),
            String::new(),
        );

        let package = Package::new(metadata);
        let mut transaction = Transaction::new(TransactionKind::Remove, vec![package]);

        let result = transaction.execute().await;

        // Update registry if successful
        if matches!(result, TransactionResult::Success { .. }) {
            drop(registry);
            let mut registry = self.registry.write().await;
            registry.remove_active(name);
            let _ = registry.save();
        }

        Ok(result)
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PackageManager")
    }
}

/// Update information
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Available updates
    pub available: Vec<PackageUpdate>,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Package update
#[derive(Debug, Clone)]
pub struct PackageUpdate {
    /// Package name
    pub name: String,
    /// Current version
    pub current_version: String,
    /// New version
    pub new_version: String,
    /// Package size in bytes
    pub size: u64,
    /// Package kind
    pub kind: PackageKind,
}

/// Update result
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Packages that were successfully updated
    pub succeeded: Vec<String>,
    /// Packages that failed to update
    pub failed: Vec<(String, String)>,
    /// Packages requiring reboot
    pub requires_reboot: Vec<String>,
}

/// System status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Total number of packages
    pub total_packages: usize,
    /// Number of active packages
    pub active_packages: usize,
    /// Number of pending updates
    pub pending_updates: usize,
    /// Total number of sources
    pub sources_total: usize,
    /// Number of enabled sources
    pub sources_enabled: usize,
}

/// Installed package information
#[derive(Debug, Clone)]
pub struct InstalledPackage {
    /// Package name
    pub name: String,
    /// Active version
    pub version: String,
    /// All installed versions
    pub versions: Vec<String>,
    /// Package kind
    pub kind: PackageKind,
}
