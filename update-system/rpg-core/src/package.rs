// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Package structures and metadata

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::signature::Signature;
use crate::version::Version;

/// Package kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    /// Application package
    App,
    /// System userland package (libraries, tools)
    System,
    /// Kernel package
    Kernel,
    /// Boot configuration
    Boot,
}

impl PackageKind {
    /// Check if this is a kernel package
    pub fn is_kernel(&self) -> bool {
        matches!(self, Self::Kernel)
    }

    /// Check if this is a system package
    pub fn is_system(&self) -> bool {
        matches!(self, Self::System | Self::Boot)
    }

    /// Check if this is an application package
    pub fn is_app(&self) -> bool {
        matches!(self, Self::App)
    }

    /// Check if packages of this kind require a reboot to activate
    pub fn requires_reboot(&self) -> bool {
        matches!(self, Self::Kernel | Self::System | Self::Boot)
    }

    /// Convert from string
    pub fn from_str(s: &str) -> crate::Result<Self> {
        match s.to_lowercase().as_str() {
            "app" | "application" => Ok(Self::App),
            "system" => Ok(Self::System),
            "kernel" => Ok(Self::Kernel),
            "boot" => Ok(Self::Boot),
            _ => Err(crate::Error::InvalidVersion(format!(
                "Unknown package kind: {}",
                s
            ))),
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            Self::App => "app",
            Self::System => "system",
            Self::Kernel => "kernel",
            Self::Boot => "boot",
        }
    }
}

impl std::fmt::Display for PackageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Package state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PackageState {
    /// Package is downloaded but not installed
    #[default]
    Downloaded,
    /// Package is installed but not active
    Installed,
    /// Package is active (current)
    Active,
    /// Package is pending activation (waiting for reboot for kernel)
    Pending,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,

    /// Package version
    pub version: Version,

    /// Package kind
    pub kind: PackageKind,

    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Package author/maintainer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Package homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Package license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Dependencies (name -> version constraint)
    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    /// Package size in bytes
    pub size: u64,

    /// SHA-256 hash of the package
    pub sha256: String,

    /// Ed25519 signature
    pub signature: Signature,

    /// Download URL
    pub url: String,

    /// Build timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_at: Option<i64>,

    /// Package state
    #[serde(default)]
    pub state: PackageState,
}

impl PackageMetadata {
    /// Create new package metadata
    pub fn new(
        name: String,
        version: Version,
        kind: PackageKind,
        size: u64,
        sha256: String,
        signature: Signature,
        url: String,
    ) -> Self {
        Self {
            name,
            version,
            kind,
            description: None,
            author: None,
            homepage: None,
            license: None,
            dependencies: HashMap::new(),
            size,
            sha256,
            signature,
            url,
            built_at: None,
            state: PackageState::Downloaded,
        }
    }

    /// Check if this package requires a reboot to activate
    pub fn requires_reboot(&self) -> bool {
        self.kind.is_kernel() || self.kind.is_system()
    }

    /// Get the package identifier
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    /// Validate the metadata
    pub fn validate(&self) -> crate::Result<()> {
        // Check required fields
        if self.name.is_empty() {
            return Err(crate::Error::Other("Package name cannot be empty".into()));
        }

        // Validate SHA-256 hash (should be 64 hex chars)
        if self.sha256.len() != 64 {
            return Err(crate::Error::Other("Invalid SHA-256 hash".into()));
        }

        // Validate signature
        if self.signature.0.len() != 64 {
            return Err(crate::Error::SignatureVerification(
                "Invalid signature".to_string(),
            ));
        }

        // Validate URL
        if self.url.is_empty() {
            return Err(crate::Error::Other("Package URL cannot be empty".into()));
        }

        Ok(())
    }
}

/// A package reference
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageRef {
    /// Package name
    pub name: String,
    /// Package version
    pub version: Version,
}

impl PackageRef {
    /// Create a new package reference
    pub fn new(name: String, version: Version) -> Self {
        Self { name, version }
    }

    /// Parse a package reference from a string
    ///
    /// Format: `name@version`
    pub fn parse(s: &str) -> crate::Result<Self> {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return Err(crate::Error::InvalidVersion(
                "Invalid package reference format".into(),
            ));
        }

        Ok(Self {
            name: parts[0].to_string(),
            version: Version::parse(parts[1])?,
        })
    }

    /// Get the package identifier string
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// A package with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package metadata
    pub metadata: PackageMetadata,
    /// Local path to the package file (if downloaded) - not serialized
    #[serde(skip)]
    pub local_path: Option<std::path::PathBuf>,
}

impl Package {
    /// Create a new package
    pub fn new(metadata: PackageMetadata) -> Self {
        Self {
            metadata,
            local_path: None,
        }
    }

    /// Create a package with a local path
    pub fn with_local(metadata: PackageMetadata, path: std::path::PathBuf) -> Self {
        Self {
            metadata,
            local_path: Some(path),
        }
    }

    /// Get the package reference
    pub fn reference(&self) -> PackageRef {
        PackageRef::new(self.metadata.name.clone(), self.metadata.version.clone())
    }

    /// Check if the package is downloaded
    pub fn is_downloaded(&self) -> bool {
        self.local_path.is_some() &&
            self.local_path.as_ref().map(|p| p.exists()).unwrap_or(false)
    }

    /// Check if the package is installed
    pub fn is_installed(&self) -> bool {
        matches!(
            self.metadata.state,
            PackageState::Installed | PackageState::Active | PackageState::Pending
        )
    }

    /// Check if the package is active
    pub fn is_active(&self) -> bool {
        self.metadata.state == PackageState::Active
    }

    /// Get the package size
    pub fn size(&self) -> u64 {
        self.metadata.size
    }

    /// Get the package name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the package version
    pub fn version(&self) -> &Version {
        &self.metadata.version
    }

    /// Get the package kind
    pub fn kind(&self) -> PackageKind {
        self.metadata.kind
    }

    /// Get the package state
    pub fn state(&self) -> PackageState {
        self.metadata.state
    }

    /// Set the package state
    pub fn set_state(&mut self, state: PackageState) {
        self.metadata.state = state;
    }
}

/// Package manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package metadata
    pub metadata: PackageMetadata,

    /// List of files in the package
    pub files: Vec<String>,

    /// Installation prefix
    pub prefix: String,

    /// Post-install script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install: Option<String>,

    /// Pre-remove script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_remove: Option<String>,
}

impl PackageManifest {
    /// Create a new package manifest
    pub fn new(metadata: PackageMetadata, prefix: String) -> Self {
        Self {
            metadata,
            files: Vec::new(),
            prefix,
            post_install: None,
            pre_remove: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_kind() {
        assert!(PackageKind::Kernel.is_kernel());
        assert!(PackageKind::System.is_system());
        assert!(PackageKind::App.is_app());
    }

    #[test]
    fn test_package_ref_parse() {
        let ref1 = PackageRef::parse("test@1.0.0").unwrap();
        assert_eq!(ref1.name, "test");
        assert_eq!(ref1.version.as_str(), "1.0.0");

        assert_eq!(ref1.id(), "test@1.0.0");
    }

    #[test]
    fn test_package_metadata_validation() {
        let version = Version::new(1, 0, 0);
        let key = crate::signature::SigningKey::generate();
        let signature = key.sign(b"test");

        let mut metadata = PackageMetadata::new(
            "test".to_string(),
            version,
            PackageKind::App,
            1024,
            "0".repeat(64),
            signature,
            "https://example.com/test.rpg".to_string(),
        );

        // Valid metadata
        assert!(metadata.validate().is_ok());

        // Invalid name
        metadata.name = String::new();
        assert!(metadata.validate().is_err());

        // Invalid SHA-256
        metadata.name = "test".to_string();
        metadata.sha256 = "invalid".to_string();
        assert!(metadata.validate().is_err());
    }

    #[test]
    fn test_package_requires_reboot() {
        let version = Version::new(1, 0, 0);
        let key = crate::signature::SigningKey::generate();
        let signature = key.sign(b"test");

        let kernel_pkg = PackageMetadata::new(
            "kernel".to_string(),
            version.clone(),
            PackageKind::Kernel,
            1024,
            "0".repeat(64),
            signature.clone(),
            "https://example.com/kernel.rpg".to_string(),
        );

        assert!(kernel_pkg.requires_reboot());

        let app_pkg = PackageMetadata::new(
            "app".to_string(),
            version,
            PackageKind::App,
            1024,
            "0".repeat(64),
            signature,
            "https://example.com/app.rpg".to_string(),
        );

        assert!(!app_pkg.requires_reboot());
    }
}
