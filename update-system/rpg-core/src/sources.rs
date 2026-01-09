// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Repository sources management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Default sources list file path
pub const SOURCES_LIST_PATH: &str = "/etc/rpg/sources.list";

/// Default sources
pub const DEFAULT_SOURCES: &[(&str, &str)] = &[
    ("kernel", "http://rustux.com/kernel"),
    ("system", "http://rustux.com/rustica"),
    ("apps", "http://rustux.com/apps"),
];

/// Repository source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Source name/identifier
    pub name: String,
    /// Source URL
    pub url: String,
    /// Source type (kernel, system, apps)
    #[serde(rename = "type")]
    pub source_type: String,
    /// Whether this source is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Source priority (lower = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_enabled() -> bool {
    true
}

fn default_priority() -> u32 {
    100
}

impl Source {
    /// Create a new source
    pub fn new(name: String, url: String, source_type: String) -> Self {
        Self {
            name,
            url,
            source_type,
            enabled: true,
            priority: 100,
        }
    }

    /// Create a source with priority
    pub fn with_priority(name: String, url: String, source_type: String, priority: u32) -> Self {
        Self {
            name,
            url,
            source_type,
            enabled: true,
            priority,
        }
    }

    /// Check if this source is for kernels
    pub fn is_kernel(&self) -> bool {
        self.source_type == "kernel"
    }

    /// Check if this source is for system packages
    pub fn is_system(&self) -> bool {
        self.source_type == "system"
    }

    /// Check if this source is for apps
    pub fn is_apps(&self) -> bool {
        self.source_type == "apps"
    }

    /// Get the package index URL for this source
    pub fn index_url(&self) -> String {
        format!("{}/index.json", self.url.trim_end_matches('/'))
    }

    /// Get the package URL for a specific package
    pub fn package_url(&self, package_name: &str, version: &str) -> String {
        format!(
            "{}/{}/{}.rpg",
            self.url.trim_end_matches('/'),
            package_name,
            version
        )
    }

    /// Check if the source is reachable
    pub async fn check_reachable(&self) -> bool {
        // In production, would perform an HTTP HEAD request
        // For now, just return true
        true
    }
}

/// Sources configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesConfig {
    /// List of sources
    #[serde(default)]
    pub sources: Vec<Source>,
}

impl Default for SourcesConfig {
    fn default() -> Self {
        Self {
            sources: DEFAULT_SOURCES
                .iter()
                .map(|(name, url)| Source::new(name.to_string(), url.to_string(), name.to_string()))
                .collect(),
        }
    }
}

impl SourcesConfig {
    /// Load sources from the default path
    pub fn load() -> crate::Result<Self> {
        Self::load_from_path(SOURCES_LIST_PATH)
    }

    /// Load sources from a specific path
    pub fn load_from_path(path: &str) -> crate::Result<Self> {
        if !Path::new(path).exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(|e| {
            crate::Error::Other(format!("Failed to read sources list: {}", e))
        })?;

        // Parse the file (line by line format)
        let mut sources = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse source line
            // Format: type url [priority]
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let source_type = parts[0];
            let url = parts[1];
            let priority = if parts.len() > 2 {
                parts[2].parse().unwrap_or(100)
            } else {
                100
            };

            sources.push(Source::with_priority(
                format!("{}-{}", source_type, url),
                url.to_string(),
                source_type.to_string(),
                priority,
            ));
        }

        Ok(Self { sources })
    }

    /// Save sources to the default path
    pub fn save(&self) -> crate::Result<()> {
        self.save_to_path(SOURCES_LIST_PATH)
    }

    /// Save sources to a specific path
    pub fn save_to_path(&self, path: &str) -> crate::Result<()> {
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Other(format!("Failed to create directory: {}", e))
            })?;
        }

        // Write sources file
        let mut content = String::from("# Rustica Package Sources\n");
        content.push_str("# Format: type url [priority]\n");
        content.push_str("# Types: kernel, system, apps\n\n");

        // Sort by priority
        let mut sorted_sources = self.sources.clone();
        sorted_sources.sort_by_key(|s| s.priority);

        for source in &sorted_sources {
            if !source.enabled {
                content.push_str("# ");
            }
            content.push_str(&format!("{} {}\n", source.source_type, source.url));
        }

        fs::write(path, content).map_err(|e| {
            crate::Error::Other(format!("Failed to write sources list: {}", e))
        })?;

        Ok(())
    }

    /// Get sources for a specific type
    pub fn get_sources_for_type(&self, source_type: &str) -> Vec<&Source> {
        self.sources
            .iter()
            .filter(|s| s.enabled && s.source_type == source_type)
            .collect()
    }

    /// Get all kernel sources
    pub fn kernel_sources(&self) -> Vec<&Source> {
        self.get_sources_for_type("kernel")
    }

    /// Get all system sources
    pub fn system_sources(&self) -> Vec<&Source> {
        self.get_sources_for_type("system")
    }

    /// Get all app sources
    pub fn app_sources(&self) -> Vec<&Source> {
        self.get_sources_for_type("apps")
    }

    /// Add a source
    pub fn add_source(&mut self, source: Source) {
        // Remove existing source with same name
        self.sources.retain(|s| s.name != source.name);
        self.sources.push(source);
    }

    /// Remove a source by name
    pub fn remove_source(&mut self, name: &str) {
        self.sources.retain(|s| s.name != name);
    }

    /// Enable a source
    pub fn enable_source(&mut self, name: &str) -> bool {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a source
    pub fn disable_source(&mut self, name: &str) -> bool {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == name) {
            source.enabled = false;
            true
        } else {
            false
        }
    }

    /// Get all enabled sources
    pub fn enabled_sources(&self) -> Vec<&Source> {
        self.sources.iter().filter(|s| s.enabled).collect()
    }

    /// Validate sources configuration
    pub fn validate(&self) -> crate::Result<()> {
        for source in &self.sources {
            if source.url.is_empty() {
                return Err(crate::Error::Other(format!(
                    "Source '{}' has empty URL",
                    source.name
                )));
            }

            // Validate URL format
            if !source.url.starts_with("http://") && !source.url.starts_with("https://") {
                return Err(crate::Error::Other(format!(
                    "Source '{}' has invalid URL: {}",
                    source.name, source.url
                )));
            }

            // Validate source type
            if !["kernel", "system", "apps"].contains(&source.source_type.as_str()) {
                return Err(crate::Error::Other(format!(
                    "Source '{}' has invalid type: {}",
                    source.name, source.source_type
                )));
            }
        }

        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> SourcesStats {
        let total = self.sources.len();
        let enabled = self.sources.iter().filter(|s| s.enabled).count();
        let kernel_count = self.kernel_sources().len();
        let system_count = self.system_sources().len();
        let apps_count = self.app_sources().len();

        SourcesStats {
            total,
            enabled,
            disabled: total - enabled,
            kernel_count,
            system_count,
            apps_count,
        }
    }
}

/// Sources statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesStats {
    /// Total number of sources
    pub total: usize,
    /// Number of enabled sources
    pub enabled: usize,
    /// Number of disabled sources
    pub disabled: usize,
    /// Number of kernel sources
    pub kernel_count: usize,
    /// Number of system sources
    pub system_count: usize,
    /// Number of app sources
    pub apps_count: usize,
}

/// Repository index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryIndex {
    /// Repository name
    pub name: String,
    /// Repository URL
    pub url: String,
    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
    /// Available packages
    pub packages: Vec<String>,
}

/// Package manifest from repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Package size in bytes
    pub size: u64,
    /// SHA-256 checksum
    pub sha256: String,
    /// Package signature (base64)
    pub signature: String,
    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Download URL
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = Source::new(
            "test".to_string(),
            "http://example.com".to_string(),
            "kernel".to_string(),
        );

        assert_eq!(source.name, "test");
        assert!(source.is_kernel());
        assert!(!source.is_apps());
    }

    #[test]
    fn test_sources_config() {
        let config = SourcesConfig::default();
        assert_eq!(config.sources.len(), 3);
        assert!(config.kernel_sources().len() > 0);
    }

    #[test]
    fn test_source_urls() {
        let source = Source::new(
            "test".to_string(),
            "http://example.com/".to_string(),
            "apps".to_string(),
        );

        assert_eq!(source.index_url(), "http://example.com/index.json");
        assert_eq!(source.package_url("foo", "1.0.0"), "http://example.com/foo/1.0.0.rpg");
    }
}
