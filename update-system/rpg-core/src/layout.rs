// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Filesystem layout for versioned packages
//!
//! This module defines the directory structure for storing versioned
//! packages and systems.

use std::path::{Path, PathBuf};

/// Base system directory
pub const SYSTEM_BASE: &str = "/system";

/// Apps directory
pub const APPS_BASE: &str = "/apps";

/// Current system symlink
pub const SYSTEM_CURRENT: &str = "/system/current";

/// System versions directory
pub const SYSTEM_VERSIONS: &str = "/system";

/// Cache directory for downloads
pub const CACHE_DIR: &str = "/var/cache/rpg";

/// Metadata directory
pub const META_DIR: &str = "/var/lib/rpg";

/// State directory
pub const STATE_DIR: &str = "/var/run/rpg";

/// Configuration directory
pub const CONFIG_DIR: &str = "/etc/rpg";

/// System layout definition
#[derive(Debug, Clone)]
pub struct SystemLayout {
    /// Base path for system versions
    pub base: PathBuf,
}

impl SystemLayout {
    /// Create a new system layout
    pub fn new() -> Self {
        Self {
            base: PathBuf::from(SYSTEM_BASE),
        }
    }

    /// Get the path to a specific version
    pub fn version_path(&self, version: &str) -> PathBuf {
        self.base.join(format!("v{}", version))
    }

    /// Get the current symlink path
    pub fn current_path(&self) -> PathBuf {
        PathBuf::from(SYSTEM_CURRENT)
    }

    /// Get the boot directory for a version
    pub fn boot_path(&self, version: &str) -> PathBuf {
        self.version_path(version).join("boot")
    }

    /// Get the kernel path for a version
    pub fn kernel_path(&self, version: &str) -> PathBuf {
        self.boot_path(version).join("kernel")
    }

    /// Get the initrd path for a version
    pub fn initrd_path(&self, version: &str) -> PathBuf {
        self.boot_path(version).join("initrd")
    }

    /// Get the userland binaries path for a version
    pub fn bin_path(&self, version: &str) -> PathBuf {
        self.version_path(version).join("bin")
    }

    /// Get the libraries path for a version
    pub fn lib_path(&self, version: &str) -> PathBuf {
        self.version_path(version).join("lib")
    }

    /// Get the metadata path for a version
    pub fn metadata_path(&self, version: &str) -> PathBuf {
        self.version_path(version).join("metadata.json")
    }

    /// List all installed versions
    pub fn list_versions(&self) -> crate::Result<Vec<String>> {
        let mut versions = Vec::new();

        if !self.base.exists() {
            return Ok(versions);
        }

        for entry in std::fs::read_dir(&self.base)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only count directories starting with 'v'
            if entry.file_type()?.is_dir() && name_str.starts_with('v') {
                versions.push(name_str[1..].to_string());
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Get the currently active version
    pub fn current_version(&self) -> crate::Result<Option<String>> {
        let current = self.current_path();

        if !current.exists() {
            return Ok(None);
        }

        let target = current.read_link()?;
        let version_str = target
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| crate::Error::Layout("Invalid current symlink".to_string()))?;

        // Remove 'v' prefix if present
        let version = if version_str.starts_with('v') {
            version_str[1..].to_string()
        } else {
            version_str.to_string()
        };

        Ok(Some(version))
    }

    /// Check if a version exists
    pub fn version_exists(&self, version: &str) -> bool {
        self.version_path(version).exists()
    }
}

impl Default for SystemLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Application layout definition
#[derive(Debug, Clone)]
pub struct AppLayout {
    /// Base path for applications
    pub base: PathBuf,
}

impl AppLayout {
    /// Create a new app layout
    pub fn new() -> Self {
        Self {
            base: PathBuf::from(APPS_BASE),
        }
    }

    /// Get the path to an app directory
    pub fn app_path(&self, app_name: &str) -> PathBuf {
        self.base.join(app_name)
    }

    /// Get the path to a specific version of an app
    pub fn version_path(&self, app_name: &str, version: &str) -> PathBuf {
        self.app_path(app_name).join(version)
    }

    /// Get the current symlink path for an app
    pub fn current_path(&self, app_name: &str) -> PathBuf {
        self.app_path(app_name).join("current")
    }

    /// Get the metadata path for an app version
    pub fn metadata_path(&self, app_name: &str, version: &str) -> PathBuf {
        self.version_path(app_name, version).join("metadata.json")
    }

    /// Get the executable path for an app
    pub fn executable_path(&self, app_name: &str) -> PathBuf {
        self.current_path(app_name).join(app_name)
    }

    /// List all installed apps
    pub fn list_apps(&self) -> crate::Result<Vec<String>> {
        let mut apps = Vec::new();

        if !self.base.exists() {
            return Ok(apps);
        }

        for entry in std::fs::read_dir(&self.base)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    apps.push(name.to_string());
                }
            }
        }

        apps.sort();
        Ok(apps)
    }

    /// List versions of an app
    pub fn list_versions(&self, app_name: &str) -> crate::Result<Vec<String>> {
        let mut versions = Vec::new();
        let app_path = self.app_path(app_name);

        if !app_path.exists() {
            return Ok(versions);
        }

        for entry in std::fs::read_dir(&app_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip the current symlink
            if name_str == "current" {
                continue;
            }

            if entry.file_type()?.is_dir() {
                versions.push(name_str.to_string());
            }
        }

        versions.sort();
        Ok(versions)
    }

    /// Get the currently active version of an app
    pub fn current_version(&self, app_name: &str) -> crate::Result<Option<String>> {
        let current = self.current_path(app_name);

        if !current.exists() {
            return Ok(None);
        }

        let target = current.read_link()?;
        let version = target
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| crate::Error::Layout("Invalid current symlink".to_string()))?;

        Ok(Some(version.to_string()))
    }

    /// Check if an app exists
    pub fn app_exists(&self, app_name: &str) -> bool {
        self.app_path(app_name).exists()
    }

    /// Check if a specific version of an app exists
    pub fn version_exists(&self, app_name: &str, version: &str) -> bool {
        self.version_path(app_name, version).exists()
    }
}

impl Default for AppLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout manager for managing system and app layouts
#[derive(Debug, Clone)]
pub struct LayoutManager {
    /// System layout
    pub system: SystemLayout,
    /// App layout
    pub apps: AppLayout,
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new() -> Self {
        Self {
            system: SystemLayout::new(),
            apps: AppLayout::new(),
        }
    }

    /// Initialize the layout directories
    pub fn initialize(&self) -> crate::Result<()> {
        // Create base directories
        std::fs::create_dir_all(&self.system.base)?;
        std::fs::create_dir_all(APPS_BASE)?;
        std::fs::create_dir_all(CACHE_DIR)?;
        std::fs::create_dir_all(META_DIR)?;
        std::fs::create_dir_all(STATE_DIR)?;
        std::fs::create_dir_all(CONFIG_DIR)?;

        Ok(())
    }

    /// Get layout statistics
    pub fn stats(&self) -> crate::Result<LayoutStats> {
        Ok(LayoutStats {
            system_versions: self.system.list_versions()?.len(),
            installed_apps: self.apps.list_apps()?.len(),
            cache_size: Self::dir_size(CACHE_DIR)?,
            metadata_size: Self::dir_size(META_DIR)?,
        })
    }

    /// Get the size of a directory
    fn dir_size(path: &str) -> crate::Result<u64> {
        let path = Path::new(path);
        if !path.exists() {
            return Ok(0);
        }

        let mut total = 0;
        let mut stack = vec![path.to_path_buf()];

        while let Some(current) = stack.pop() {
            for entry in std::fs::read_dir(current)? {
                let entry = entry?;
                let ty = entry.file_type()?;

                if ty.is_file() {
                    total += entry.metadata()?.len();
                } else if ty.is_dir() {
                    stack.push(entry.path());
                }
            }
        }

        Ok(total)
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout statistics
#[derive(Debug, Clone)]
pub struct LayoutStats {
    /// Number of system versions installed
    pub system_versions: usize,
    /// Number of apps installed
    pub installed_apps: usize,
    /// Size of cache directory in bytes
    pub cache_size: u64,
    /// Size of metadata directory in bytes
    pub metadata_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_layout_paths() {
        let layout = SystemLayout::new();
        assert_eq!(layout.version_path("1.0.0"), PathBuf::from("/system/v1.0.0"));
        assert_eq!(layout.current_path(), PathBuf::from("/system/current"));
    }

    #[test]
    fn test_app_layout_paths() {
        let layout = AppLayout::new();
        assert_eq!(layout.app_path("test"), PathBuf::from("/apps/test"));
        assert_eq!(
            layout.version_path("test", "1.0.0"),
            PathBuf::from("/apps/test/1.0.0")
        );
        assert_eq!(
            layout.current_path("test"),
            PathBuf::from("/apps/test/current")
        );
    }
}
