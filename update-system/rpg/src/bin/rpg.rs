// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! RPG (Rustica Package Manager) CLI
//!
//! The main command-line interface for managing packages
//! in the Rustica Operating System.

use clap::{Parser, Subcommand};
use rpg_core::{ops::PackageManager, sources::SourcesConfig, Error};
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

/// RPG - Rustica Package Manager
#[derive(Parser, Debug)]
#[command(name = "rpg")]
#[command(about = "Rustica Package Manager - Manage system and application packages", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Rpg {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Path to configuration directory
    #[arg(short, long, default_value = "/etc/rpg")]
    config_dir: PathBuf,

    /// Path to sources list file
    #[arg(short, long, default_value = "/etc/rpg/sources.list")]
    sources_file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Check for and install updates
    Update {
        /// Update in background without prompting
        #[arg(short, long)]
        background: bool,

        /// Only check for updates, don't install
        #[arg(long)]
        check_only: bool,

        /// Specific package to update (default: all packages)
        #[arg(short, long)]
        package: Option<String>,

        /// Force re-download even if package exists
        #[arg(long)]
        force: bool,
    },

    /// Rollback to a previous version
    Rollback {
        /// Package to rollback (or "system" for system rollback)
        package: String,

        /// Specific version to rollback to (default: previous version)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Show system and package status
    Status {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Show only installed packages
        #[arg(long)]
        installed: bool,

        /// Show only available updates
        #[arg(long)]
        updates: bool,
    },

    /// Manage repository sources
    Sources {
        #[command(subcommand)]
        action: SourcesCommands,
    },

    /// List available packages
    List {
        /// Search pattern for package names
        #[arg(short, long)]
        pattern: Option<String>,

        /// Filter by package type
        #[arg(short, long)]
        kind: Option<String>,
    },

    /// Install a package
    Install {
        /// Package name
        package: String,

        /// Specific version to install
        #[arg(short, long)]
        version: Option<String>,

        /// Don't install dependencies
        #[arg(long)]
        no_deps: bool,
    },

    /// Remove a package
    Remove {
        /// Package name
        package: String,

        /// Remove configuration files
        #[arg(long)]
        purge: bool,
    },
}

/// Sources management commands
#[derive(Subcommand, Debug)]
enum SourcesCommands {
    /// List configured sources
    List {
        /// Show disabled sources
        #[arg(long)]
        all: bool,
    },

    /// Add a new source
    Add {
        /// Source name
        name: String,

        /// Source URL
        url: String,

        /// Source type (kernel, system, apps)
        kind: String,

        /// Priority (lower = higher priority)
        #[arg(short, long, default_value = "100")]
        priority: u32,
    },

    /// Remove a source
    Remove {
        /// Source name
        name: String,
    },

    /// Enable a source
    Enable {
        /// Source name
        name: String,
    },

    /// Disable a source
    Disable {
        /// Source name
        name: String,
    },

    /// Check if sources are reachable
    Check {
        /// Source name (default: all sources)
        name: Option<String>,
    },

    /// Update repository indices from sources
    Update,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Rpg::parse();

    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(log_level.into())
                .from_env_lossy(),
        )
        .init();

    // Execute command
    match args.command {
        Commands::Update {
            background,
            check_only,
            package,
            force,
        } => {
            cmd_update(background, check_only, package, force, &args.sources_file).await?;
        }
        Commands::Rollback { package, version } => {
            cmd_rollback(package, version).await?;
        }
        Commands::Status {
            detailed,
            installed,
            updates,
        } => {
            cmd_status(detailed, installed, updates, &args.sources_file).await?;
        }
        Commands::Sources { action } => {
            cmd_sources(action, &args.sources_file).await?;
        }
        Commands::List { pattern, kind } => {
            cmd_list(pattern, kind).await?;
        }
        Commands::Install {
            package,
            version,
            no_deps,
        } => {
            cmd_install(package, version, no_deps).await?;
        }
        Commands::Remove { package, purge } => {
            cmd_remove(package, purge).await?;
        }
    }

    Ok(())
}

/// Check for and install updates
async fn cmd_update(
    background: bool,
    check_only: bool,
    package: Option<String>,
    _force: bool,
    _sources_file: &PathBuf,
) -> Result<(), Error> {
    let manager = PackageManager::new()?;

    if check_only {
        info!("Checking for available updates...");
        let update_info = manager.check_updates().await?;

        if update_info.available.is_empty() {
            println!("No updates available.");
        } else {
            println!("Available updates:");
            for update in &update_info.available {
                println!(
                    "  {} ({} -> {}) - {} bytes",
                    update.name, update.current_version, update.new_version, update.size
                );
            }
        }

        for error in &update_info.errors {
            warn!("Update check error: {}", error);
        }

        return Ok(());
    }

    if background {
        info!("Running in background mode...");
        // TODO: Implement background update mode
    }

    if let Some(pkg) = package {
        info!("Updating package: {}", pkg);
        // TODO: Implement single package update
    } else {
        info!("Updating all packages...");
        let result = manager.update_all().await?;

        if result.succeeded.is_empty() && result.failed.is_empty() {
            println!("No updates available.");
        } else {
            if !result.succeeded.is_empty() {
                println!("Successfully updated {} package(s):", result.succeeded.len());
                for pkg in &result.succeeded {
                    println!("  - {}", pkg);
                }
            }

            if !result.failed.is_empty() {
                println!("\nFailed to update {} package(s):", result.failed.len());
                for (pkg, error) in &result.failed {
                    println!("  - {}: {}", pkg, error);
                }
            }

            if !result.requires_reboot.is_empty() {
                println!("\nReboot required for: {}", result.requires_reboot.join(", "));
            }
        }
    }

    Ok(())
}

/// Rollback to a previous version
async fn cmd_rollback(package: String, version: Option<String>) -> Result<(), Error> {
    let manager = PackageManager::new()?;

    info!("Rolling back {} to {:?}", package, version);

    if package == "system" {
        info!("Rolling back system...");
        // TODO: Implement system rollback
        println!("System rollback not yet implemented");
    } else {
        info!("Rolling back package: {}", package);
        let result = manager.rollback(&package, version.as_deref()).await?;

        match result {
            rpg_core::transaction::TransactionResult::Success { activated, .. } => {
                println!("Successfully rolled back {}", activated.join(", "));
            }
            rpg_core::transaction::TransactionResult::Failed { error, .. } => {
                println!("Rollback failed: {}", error);
                return Err(Error::Other(error));
            }
            rpg_core::transaction::TransactionResult::RolledBack { reason, .. } => {
                println!("Rollback completed: {}", reason);
            }
        }
    }

    Ok(())
}

/// Show system and package status
async fn cmd_status(
    detailed: bool,
    installed: bool,
    updates: bool,
    sources_file: &PathBuf,
) -> Result<(), Error> {
    // Load sources configuration
    let sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
        .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

    println!("=== Rustica Package Manager Status ===\n");

    // Show sources statistics
    let stats = sources.stats();
    println!("Sources:");
    println!("  Total: {}", stats.total);
    println!("  Enabled: {}", stats.enabled);
    println!("  Disabled: {}", stats.disabled);
    println!("  Kernel: {}", stats.kernel_count);
    println!("  System: {}", stats.system_count);
    println!("  Apps: {}", stats.apps_count);

    if detailed {
        println!("\nConfigured Sources:");
        for source in &sources.sources {
            let status = if source.enabled { "enabled" } else { "disabled" };
            println!("  - {} ({}, {}, priority: {})",
                source.name, source.source_type, status, source.priority);
            println!("    URL: {}", source.url);
        }
    }

    // Show installed packages
    if installed || (!updates && !installed) {
        let manager = PackageManager::new()?;
        let installed_packages = manager.list_installed().await?;

        println!("\nInstalled Packages:");
        if installed_packages.is_empty() {
            println!("  (No packages installed)");
        } else {
            for pkg in &installed_packages {
                println!("  {} ({})", pkg.name, pkg.version);
                if detailed {
                    println!("    Kind: {:?}", pkg.kind);
                    if pkg.versions.len() > 1 {
                        println!("    Other versions: {}", pkg.versions.iter()
                            .filter(|v| *v != &pkg.version)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", "));
                    }
                }
            }
        }
    }

    // Show available updates
    if updates || (!updates && !installed) {
        let manager = PackageManager::new()?;
        let update_info = manager.check_updates().await?;

        println!("\nAvailable Updates:");
        if update_info.available.is_empty() {
            println!("  (No updates available)");
        } else {
            for update in &update_info.available {
                println!(
                    "  {} ({} -> {}) - {} bytes",
                    update.name, update.current_version, update.new_version, update.size
                );
            }
        }
    }

    Ok(())
}

/// Manage repository sources
async fn cmd_sources(action: SourcesCommands, sources_file: &PathBuf) -> Result<(), Error> {
    match action {
        SourcesCommands::List { all } => {
            let sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            println!("=== Configured Sources ===\n");
            for source in &sources.sources {
                if !source.enabled && !all {
                    continue;
                }
                let status = if source.enabled { "enabled" } else { "disabled" };
                println!("{} ({})", source.name, status);
                println!("  Type: {}", source.source_type);
                println!("  URL: {}", source.url);
                println!("  Priority: {}", source.priority);
                println!();
            }
        }
        SourcesCommands::Add { name, url, kind, priority } => {
            info!("Adding source: {} -> {}", name, url);

            let mut sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            let source = rpg_core::Source::with_priority(name.clone(), url, kind, priority);
            sources.add_source(source);
            sources.validate()?;
            sources.save()?;

            println!("Added source: {}", name);
        }
        SourcesCommands::Remove { name } => {
            info!("Removing source: {}", name);

            let mut sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            sources.remove_source(&name);
            sources.save()?;

            println!("Removed source: {}", name);
        }
        SourcesCommands::Enable { name } => {
            info!("Enabling source: {}", name);

            let mut sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            if sources.enable_source(&name) {
                sources.save()?;
                println!("Enabled source: {}", name);
            } else {
                warn!("Source not found: {}", name);
                return Err(Error::Other(format!("Source not found: {}", name)));
            }
        }
        SourcesCommands::Disable { name } => {
            info!("Disabling source: {}", name);

            let mut sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            if sources.disable_source(&name) {
                sources.save()?;
                println!("Disabled source: {}", name);
            } else {
                warn!("Source not found: {}", name);
                return Err(Error::Other(format!("Source not found: {}", name)));
            }
        }
        SourcesCommands::Check { name } => {
            let sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            if let Some(name) = name {
                // Check specific source
                let source = sources.sources.iter()
                    .find(|s| s.name == name)
                    .ok_or_else(|| Error::Other(format!("Source not found: {}", name)))?;

                println!("Checking source: {}", source.name);
                let reachable = source.check_reachable().await;
                if reachable {
                    println!("  Status: Reachable");
                } else {
                    println!("  Status: Not reachable");
                }
            } else {
                // Check all sources
                println!("=== Checking All Sources ===\n");
                for source in &sources.sources {
                    if !source.enabled {
                        continue;
                    }
                    println!("{}: ", source.name);
                    let reachable = source.check_reachable().await;
                    if reachable {
                        println!("  Reachable");
                    } else {
                        println!("  Not reachable");
                    }
                }
            }
        }
        SourcesCommands::Update => {
            info!("Updating repository indices from sources...");

            let sources = SourcesConfig::load_from_path(sources_file.to_str().unwrap())
                .map_err(|e| Error::Other(format!("Failed to load sources: {}", e)))?;

            // TODO: Fetch indices from all sources
            println!("Updating indices from {} sources", sources.enabled_sources().len());
            println!("(Not yet implemented)");
        }
    }

    Ok(())
}

/// List available packages
async fn cmd_list(pattern: Option<String>, kind: Option<String>) -> Result<(), Error> {
    let manager = PackageManager::new()?;

    info!("Listing packages...");

    // Check what's available from sources
    let update_info = manager.check_updates().await?;

    if update_info.available.is_empty() {
        println!("No packages available (sources may be unreachable)");
    } else {
        println!("Available Packages:");

        for mut update in update_info.available {
            // Filter by pattern if specified
            if let Some(ref p) = pattern {
                if !update.name.contains(p) {
                    continue;
                }
            }

            // Filter by kind if specified
            if let Some(ref k) = kind {
                let kind_str = match update.kind {
                    rpg_core::PackageKind::App => "app",
                    rpg_core::PackageKind::System => "system",
                    rpg_core::PackageKind::Kernel => "kernel",
                    rpg_core::PackageKind::Boot => "boot",
                };
                if kind_str != k {
                    continue;
                }
            }

            println!(
                "  {} ({}) - {} bytes - {}",
                update.name, update.new_version, update.size, update.kind
            );
        }
    }

    // Also show installed packages
    let installed = manager.list_installed().await?;
    if !installed.is_empty() {
        println!("\nInstalled Packages:");
        for pkg in installed {
            // Apply filters
            if let Some(ref p) = pattern {
                if !pkg.name.contains(p) {
                    continue;
                }
            }
            if let Some(ref k) = kind {
                let kind_str = match pkg.kind {
                    rpg_core::PackageKind::App => "app",
                    rpg_core::PackageKind::System => "system",
                    rpg_core::PackageKind::Kernel => "kernel",
                    rpg_core::PackageKind::Boot => "boot",
                };
                if kind_str != k {
                    continue;
                }
            }

            println!("  {} ({}) - {}", pkg.name, pkg.version, pkg.kind);
        }
    }

    Ok(())
}

/// Install a package
async fn cmd_install(package: String, version: Option<String>, _no_deps: bool) -> Result<(), Error> {
    let manager = PackageManager::new()?;

    info!("Installing package: {}", package);

    // Determine package kind
    let kind = if package == "kernel" {
        rpg_core::PackageKind::Kernel
    } else if package == "system" {
        rpg_core::PackageKind::System
    } else {
        rpg_core::PackageKind::App
    };

    match manager.install_package(&package, version.as_deref(), kind).await? {
        rpg_core::transaction::TransactionResult::Success { activated, requires_reboot } => {
            if !activated.is_empty() {
                println!("Successfully installed: {}", activated.join(", "));
            }
            if !requires_reboot.is_empty() {
                println!("Reboot required for: {}", requires_reboot.join(", "));
            }
        }
        rpg_core::transaction::TransactionResult::Failed { error, .. } => {
            println!("Installation failed: {}", error);
            return Err(Error::Other(error));
        }
        rpg_core::transaction::TransactionResult::RolledBack { reason, .. } => {
            println!("Installation rolled back: {}", reason);
            return Err(Error::Other(reason));
        }
    }

    Ok(())
}

/// Remove a package
async fn cmd_remove(package: String, _purge: bool) -> Result<(), Error> {
    let manager = PackageManager::new()?;

    info!("Removing package: {}", package);

    match manager.remove_package(&package).await? {
        rpg_core::transaction::TransactionResult::Success { activated, .. } => {
            println!("Successfully removed: {}", activated.join(", "));
        }
        rpg_core::transaction::TransactionResult::Failed { error, .. } => {
            println!("Removal failed: {}", error);
            return Err(Error::Other(error));
        }
        rpg_core::transaction::TransactionResult::RolledBack { reason, .. } => {
            println!("Removal rolled back: {}", reason);
            return Err(Error::Other(reason));
        }
    }

    Ok(())
}
