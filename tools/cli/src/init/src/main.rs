// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Rustica Init System
//!
//! The first userspace process (PID 1) responsible for:
//! - Mounting filesystems
//! - Starting essential services
//! - Setting up the system environment
//! - Launching the shell or display manager

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Child, Command};

/// Init system configuration
#[derive(Debug)]
struct InitConfig {
    /// Runlevel to start
    runlevel: RunLevel,
    /// Services to start
    services: Vec<ServiceConfig>,
    /// Whether to start shell or display manager
    target: InitTarget,
}

/// Runlevel definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RunLevel {
    /// System halt
    Halt = 0,
    /// Single user mode
    SingleUser = 1,
    /// Multiuser mode (no networking)
    Multiuser = 2,
    /// Multiuser mode (with networking)
    Network = 3,
    /// Reserved
    Reserved4 = 4,
    /// Graphical interface
    Graphical = 5,
    /// Reboot
    Reboot = 6,
}

/// Init target
#[derive(Debug, Clone, PartialEq, Eq)]
enum InitTarget {
    /// Start shell
    Shell,
    /// Start display manager
    DisplayManager,
    /// Custom command
    Command(String),
}

/// Service configuration
#[derive(Debug, Clone)]
struct ServiceConfig {
    /// Service name
    name: String,
    /// Service type
    service_type: ServiceType,
    /// Command to execute
    command: String,
    /// Arguments
    args: Vec<String>,
    /// Working directory
    workdir: Option<String>,
    /// Environment variables
    env: Vec<(String, String)>,
    /// Dependencies
    depends_on: Vec<String>,
    /// Restart policy
    restart: RestartPolicy,
}

/// Service type
#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceType {
    /// Simple fork
    Simple,
    /// Forking daemon
    Forking,
    /// Oneshot
    Oneshot,
    /// DBus service
    Dbus,
}

/// Restart policy
#[derive(Debug, Clone, PartialEq, Eq)]
enum RestartPolicy {
    /// Never restart
    Never,
    /// Always restart
    Always,
    /// Restart on failure
    OnFailure,
}

/// Init state
struct InitState {
    /// Running services
    services: Vec<ServiceState>,
    /// Child processes
    children: Vec<Child>,
}

/// Service state
struct ServiceState {
    /// Service configuration
    config: ServiceConfig,
    /// Current status
    status: ServiceStatus,
    /// PID if running
    pid: Option<u32>,
}

/// Service status
#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceStatus {
    /// Service not started
    Stopped,
    /// Service starting
    Starting,
    /// Service running
    Running,
    /// Service failed
    Failed(String),
}

fn main() -> Result<()> {
    // Set up logging early
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Rustica Init v0.1.0 starting...");

    // Display splash screen
    display_splash();

    // Load configuration
    let config = load_config()?;

    // Run init
    let mut state = InitState {
        services: Vec::new(),
        children: Vec::new(),
    };

    run_init(&mut state, &config)
}

/// Display init splash screen
fn display_splash() {
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────");
    println!("─████████████████───██████──██████─██████████████─██████████████─██████████─██████████████─██████████████─");
    println!("─██░░░░░░░░░░░░██───██░░██──██░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─██░░░░░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─");
    println!("─██░░████████░░██───██░░██──██░░██─██░░██████████─██████░░██████─████░░████─██░░██████████─██░░██████░░██─");
    println!("─██░░██────██░░██───██░░██──██░░██─██░░██─────────────██░░██───────██░░██───██░░██─────────██░░██──██░░██─");
    println!("─██░░████████░░██───██░░██──██░░██─██░░██████████─────██░░██───────██░░██───██░░██─────────██░░██████░░██─");
    println!("─██░░░░░░░░░░░░██───██░░██──██░░██─██░░░░░░░░░░██─────██░░██───────██░░██───██░░██─────────██░░░░░░░░░░██─");
    println!("─██░░██████░░████───██░░██──██░░██─██████████░░██─────██░░██───────██░░██───██░░██─────────██░░██████░░██─");
    println!("─██░░██──██░░██─────██░░██──██░░██─────────██░░██─────██░░██───────██░░██───██░░██─────────██░░██──██░░██─");
    println!("─██░░██──██░░██████─██░░██████░░██─██████████░░██─────██░░██─────████░░████─██░░██████████─██░░██──██░░██─");
    println!("─██░░██──██░░░░░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─────██░░██─────██░░░░░░██─██░░░░░░░░░░██─██░░██──██░░██─");
    println!("─██████──██████████─██████████████─██████████████─────██████─────██████████─██████████████─██████──██████─");
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────");
    println!("Operating System version: v.0.0.1");
    println!("Rustux Kernel version: v.0.0.1");
    println!("Visit: http://rustux.com");
    println!();
}

/// Load init configuration
fn load_config() -> Result<InitConfig> {
    log::info!("Loading init configuration...");

    // Default configuration
    let config = InitConfig {
        runlevel: RunLevel::Network,
        services: default_services()?,
        target: InitTarget::Shell,
    };

    // Try to load from config file
    let config_path = Path::new("/etc/rustica/init.conf");
    if config_path.exists() {
        log::info!("Loading configuration from {}", config_path.display());
        // TODO: Parse config file
    } else {
        log::info!("Using default configuration");
    }

    Ok(config)
}

/// Get default services
fn default_services() -> Result<Vec<ServiceConfig>> {
    Ok(vec![
        ServiceConfig {
            name: "syslog".to_string(),
            service_type: ServiceType::Simple,
            command: "/usr/bin/syslogd".to_string(),
            args: vec![],
            workdir: None,
            env: vec![],
            depends_on: vec![],
            restart: RestartPolicy::Always,
        },
        ServiceConfig {
            name: "network".to_string(),
            service_type: ServiceType::Simple,
            command: "/usr/bin/network-init".to_string(),
            args: vec![],
            workdir: None,
            env: vec![],
            depends_on: vec![],
            restart: RestartPolicy::OnFailure,
        },
        ServiceConfig {
            name: "firewall".to_string(),
            service_type: ServiceType::Oneshot,
            command: "/usr/bin/fwctl".to_string(),
            args: vec!["load".to_string()],
            workdir: None,
            env: vec![],
            depends_on: vec!["network".to_string()],
            restart: RestartPolicy::Never,
        },
    ])
}

/// Run init
fn run_init(state: &mut InitState, config: &InitConfig) -> Result<()> {
    log::info!("Starting init with runlevel: {:?}", config.runlevel);

    // Phase 1: Mount essential filesystems
    mount_filesystems()?;

    // Phase 2: Set up environment
    setup_environment()?;

    // Phase 3: Start services
    start_services(state, config)?;

    // Phase 4: Start target
    start_target(config)?;

    // Phase 5: Wait forever (we are PID 1)
    wait_forever();

    Ok(())
}

/// Mount essential filesystems
fn mount_filesystems() -> Result<()> {
    log::info!("Mounting essential filesystems...");

    // Create mount points
    create_dir("/dev")?;
    create_dir("/proc")?;
    create_dir("/sys")?;
    create_dir("/tmp")?;
    create_dir("/var")?;
    create_dir("/var/log")?;
    create_dir("/var/run")?;
    create_dir("/mnt")?;

    // Mount proc filesystem
    if Path::new("/proc").exists() {
        log::info!("Mounting /proc");
        let _ = Command::new("mount")
            .args(["-t", "proc", "proc", "/proc"])
            .status();
    }

    // Mount sysfs
    if Path::new("/sys").exists() {
        log::info!("Mounting /sys");
        let _ = Command::new("mount")
            .args(["-t", "sysfs", "sysfs", "/sys"])
            .status();
    }

    // Mount devtmpfs
    if Path::new("/dev").exists() {
        log::info!("Mounting /dev");
        let _ = Command::new("mount")
            .args(["-t", "devtmpfs", "devtmpfs", "/dev"])
            .status();
    }

    // Mount tmpfs on /tmp
    log::info!("Mounting /tmp");
    let _ = Command::new("mount")
        .args(["-t", "tmpfs", "tmpfs", "/tmp"])
        .status();

    log::info!("Filesystems mounted");
    Ok(())
}

/// Create directory if it doesn't exist
fn create_dir(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("cannot create directory: {}", path))?;
    }
    Ok(())
}

/// Set up environment
fn setup_environment() -> Result<()> {
    log::info!("Setting up environment...");

    // Set hostname
    let hostname = fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "rustica".to_string());
    let _ = Command::new("hostname")
        .arg(&hostname.trim())
        .status();

    // Set environment variables
    env::set_var("PATH", "/bin:/usr/bin:/usr/local/bin:/sbin:/usr/sbin");
    env::set_var("HOME", "/root");
    env::set_var("USER", "root");
    env::set_var("SHELL", "/bin/sh");
    env::set_var("TERM", "xterm-256color");
    env::set_var("LANG", "C.UTF-8");

    log::info!("Environment configured");
    Ok(())
}

/// Start services
fn start_services(state: &mut InitState, config: &InitConfig) -> Result<()> {
    log::info!("Starting services...");

    for service_config in &config.services {
        log::info!("Starting service: {}", service_config.name);

        // Check dependencies
        let deps_met = service_config.depends_on.iter().all(|dep| {
            state.services.iter().any(|s| {
                s.config.name == *dep && s.status == ServiceStatus::Running
            })
        });

        if !deps_met {
            log::warn!("Skipping service {} (dependencies not met)", service_config.name);
            continue;
        }

        // Start service
        match start_service(service_config) {
            Ok(child) => {
                state.children.push(child);
                state.services.push(ServiceState {
                    config: service_config.clone(),
                    status: ServiceStatus::Running,
                    pid: None,
                });
                log::info!("Service {} started", service_config.name);
            }
            Err(e) => {
                log::error!("Failed to start service {}: {}", service_config.name, e);
                state.services.push(ServiceState {
                    config: service_config.clone(),
                    status: ServiceStatus::Failed(e.to_string()),
                    pid: None,
                });
            }
        }
    }

    log::info!("Services started");
    Ok(())
}

/// Start a single service
fn start_service(config: &ServiceConfig) -> Result<Child> {
    let mut cmd = Command::new(&config.command);

    // Add arguments
    cmd.args(&config.args);

    // Set working directory
    if let Some(ref workdir) = config.workdir {
        cmd.current_dir(workdir);
    }

    // Set environment variables
    for (key, value) in &config.env {
        cmd.env(key, value);
    }

    cmd.spawn()
        .with_context(|| format!("failed to start service: {}", config.name))
}

/// Start init target (shell or display manager)
fn start_target(config: &InitConfig) -> Result<()> {
    log::info!("Starting init target: {:?}", config.target);

    match &config.target {
        InitTarget::Shell => {
            log::info!("Starting shell");
            let mut child = Command::new("/bin/sh")
                .spawn()
                .context("failed to start shell")?;

            // Wait for shell to exit
            child.wait()?;
            log::warn!("Shell exited, this should not happen in normal operation");
        }
        InitTarget::DisplayManager => {
            log::info!("Starting display manager");
            let mut child = Command::new("/usr/bin/display-manager")
                .spawn()
                .context("failed to start display manager")?;

            child.wait()?;
        }
        InitTarget::Command(cmd) => {
            log::info!("Running custom command: {}", cmd);
            let mut child = Command::new(cmd)
                .spawn()
                .context("failed to run command")?;

            child.wait()?;
        }
    }

    Ok(())
}

/// Wait forever (init is PID 1 and should never exit)
fn wait_forever() -> ! {
    log::info!("Init is now running (PID 1)");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
