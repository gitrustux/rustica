// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Configuration management for the package manager

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for the package manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Repository URLs
    pub repositories: Vec<String>,

    /// Whether to enable automatic background updates
    pub auto_updates_enabled: bool,

    /// Update check interval in seconds
    pub update_check_interval: u64,

    /// Maximum bandwidth for updates (bytes/sec, 0 = unlimited)
    pub max_bandwidth: u64,

    /// Whether to verify package signatures
    pub verify_signatures: bool,

    /// Embedded public key for signature verification (base64)
    pub trust_key: Option<String>,

    /// Cache directory
    pub cache_dir: PathBuf,

    /// Metadata directory
    pub metadata_dir: PathBuf,

    /// State directory
    pub state_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repositories: vec!["https://repo.rustica.os".to_string()],
            auto_updates_enabled: false,
            update_check_interval: 86400, // 24 hours
            max_bandwidth: 0,
            verify_signatures: true,
            trust_key: None,
            cache_dir: PathBuf::from("/var/cache/rpg"),
            metadata_dir: PathBuf::from("/var/lib/rpg"),
            state_dir: PathBuf::from("/var/run/rpg"),
        }
    }
}

impl Config {
    /// Load configuration from the default path
    pub fn load() -> crate::Result<Self> {
        let config_path = "/etc/rpg/config.json";
        Self::load_from_path(config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &str) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|_| {
            crate::Error::Other(format!("Configuration file not found: {}", path))
        })?;

        serde_json::from_str(&content)
            .map_err(|e| crate::Error::Serialization(e.to_string()))
    }

    /// Save configuration to the default path
    pub fn save(&self) -> crate::Result<()> {
        let config_path = "/etc/rpg/config.json";

        // Ensure directory exists
        if let Some(parent) = PathBuf::from(config_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Get the repository URLs
    pub fn repositories(&self) -> &[String] {
        &self.repositories
    }

    /// Add a repository
    pub fn add_repository(&mut self, url: String) {
        if !self.repositories.contains(&url) {
            self.repositories.push(url);
        }
    }

    /// Remove a repository
    pub fn remove_repository(&mut self, url: &str) {
        self.repositories.retain(|r| r != url);
    }

    /// Check if signature verification is enabled
    pub fn verify_signatures_enabled(&self) -> bool {
        self.verify_signatures
    }
}

/// Update-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Whether to enable live background updates
    pub live_updates_enabled: bool,

    /// Whether to pause updates during high system load
    pub pause_on_high_load: bool,

    /// Maximum CPU usage percentage to allow updates
    pub max_cpu_usage: u8,

    /// Whether to notify user before installing updates
    pub notify_before_install: bool,

    /// Preferred update time (for scheduled updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_time: Option<String>,

    /// Whether to auto-apply updates that don't require reboot
    pub auto_apply_non_kernel: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            live_updates_enabled: true,
            pause_on_high_load: true,
            max_cpu_usage: 50,
            notify_before_install: false,
            preferred_time: None,
            auto_apply_non_kernel: true,
        }
    }
}

impl UpdateConfig {
    /// Load update configuration
    pub fn load() -> crate::Result<Self> {
        let config_path = "/etc/rpg/update-config.json";
        let path = PathBuf::from(config_path);

        if path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&content)
                .map_err(|e| crate::Error::Serialization(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// Save update configuration
    pub fn save(&self) -> crate::Result<()> {
        let config_path = "/etc/rpg/update-config.json";

        // Ensure directory exists
        if let Some(parent) = PathBuf::from(config_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Check if live updates are enabled
    pub fn live_updates_enabled(&self) -> bool {
        self.live_updates_enabled
    }

    /// Check if updates should pause based on CPU usage
    pub fn should_pause(&self, current_cpu_usage: u8) -> bool {
        self.pause_on_high_load && current_cpu_usage > self.max_cpu_usage
    }
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Whether the user has opted in to live updates
    pub live_updates_opt_in: bool,

    /// Whether to show update notifications
    pub show_notifications: bool,

    /// Maximum bandwidth to use for updates
    pub max_bandwidth_mbps: Option<u32>,

    /// Whether to download only on WiFi (if applicable)
    pub wifi_only: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            live_updates_opt_in: true,
            show_notifications: true,
            max_bandwidth_mbps: None,
            wifi_only: false,
        }
    }
}

impl UserPreferences {
    /// Load user preferences
    pub fn load() -> crate::Result<Self> {
        let config_path = "/etc/rpg/user-prefs.json";
        let path = PathBuf::from(config_path);

        if path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&content)
                .map_err(|e| crate::Error::Serialization(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// Save user preferences
    pub fn save(&self) -> crate::Result<()> {
        let config_path = "/etc/rpg/user-prefs.json";

        // Ensure directory exists
        if let Some(parent) = PathBuf::from(config_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Check if user has opted in to live updates
    pub fn live_updates_enabled(&self) -> bool {
        self.live_updates_opt_in
    }
}
