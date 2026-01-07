// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! svc - Service manager

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Service Manager
#[derive(Parser, Debug)]
#[command(name = "svc")]
#[command(about = "Service Manager", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all services
    List {
        /// Show all services including stopped
        #[arg(short, long)]
        all: bool,
    },

    /// Start service
    Start {
        /// Service name
        service: String,
    },

    /// Stop service
    Stop {
        /// Service name
        service: String,
    },

    /// Restart service
    Restart {
        /// Service name
        service: String,
    },

    /// Show service status
    Status {
        /// Service name
        service: String,
    },

    /// Enable service (start at boot)
    Enable {
        /// Service name
        service: String,
    },

    /// Disable service (don't start at boot)
    Disable {
        /// Service name
        service: String,
    },

    /// Show service logs
    Logs {
        /// Service name
        service: String,

        /// Number of lines
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: usize,

        /// Follow logs
        #[arg(short = 'f', long)]
        follow: bool,
    },
}

#[derive(Debug, Clone)]
struct Service {
    name: String,
    description: String,
    exec_start: String,
    exec_stop: Option<String>,
    working_dir: Option<String>,
    enabled: bool,
    running: bool,
    pid: Option<u32>,
    auto_start: bool,
}

struct ServiceManager {
    services_dir: PathBuf,
    services: HashMap<String, Service>,
}

impl ServiceManager {
    fn new() -> Result<Self> {
        let services_dir = PathBuf::from("/etc/rustica/services");

        let mut sm = Self {
            services_dir,
            services: HashMap::new(),
        };

        sm.load_services()?;

        Ok(sm)
    }

    fn load_services(&mut self) -> Result<()> {
        if !self.services_dir.exists() {
            // Create default services
            fs::create_dir_all(&self.services_dir)?;

            // Add default services
            self.services.insert(
                "network".to_string(),
                Service {
                    name: "network".to_string(),
                    description: "Network initialization".to_string(),
                    exec_start: "/usr/bin/network-init".to_string(),
                    exec_stop: None,
                    working_dir: None,
                    enabled: true,
                    running: true,
                    pid: None,
                    auto_start: true,
                },
            );

            self.services.insert(
                "firewall".to_string(),
                Service {
                    name: "firewall".to_string(),
                    description: "Firewall service".to_string(),
                    exec_start: "/usr/bin/fwctl load".to_string(),
                    exec_stop: Some("/usr/bin/fwctl flush".to_string()),
                    working_dir: None,
                    enabled: true,
                    running: true,
                    pid: None,
                    auto_start: true,
                },
            );

            return Ok(());
        }

        // Load service files from directory
        for entry in fs::read_dir(&self.services_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("service") {
                // Parse service file
                if let Ok(service) = self.parse_service_file(&path) {
                    self.services.insert(service.name.clone(), service);
                }
            }
        }

        Ok(())
    }

    fn parse_service_file(&self, path: &PathBuf) -> Result<Service> {
        let content = fs::read_to_string(path)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse service file (simplified INI format)
        let mut description = String::new();
        let mut exec_start = String::new();
        let mut exec_stop = None;
        let mut working_dir = None;
        let mut auto_start = false;

        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_string();
                continue;
            }

            // Key-value pair
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match (current_section, key) {
                    ("Unit", "Description") => description = value.to_string(),
                    ("Service", "ExecStart") => exec_start = value.to_string(),
                    ("Service", "ExecStop") => exec_stop = Some(value.to_string()),
                    ("Service", "WorkingDirectory") => working_dir = Some(value.to_string()),
                    ("Install", "WantedBy") => auto_start = true,
                    _ => {}
                }
            }
        }

        Ok(Service {
            name,
            description,
            exec_start,
            exec_stop,
            working_dir,
            enabled: auto_start,
            running: false,
            pid: None,
            auto_start,
        })
    }

    fn list_services(&self, show_all: bool) {
        println!("Loaded services:");

        for (name, service) in &self.services {
            if !show_all && !service.running {
                continue;
            }

            let status = if service.running {
                "\x1b[1;32mrunning\x1b[0m"
            } else {
                "\x1b[1;31mstopped\x1b[0m"
            };

            let enabled = if service.enabled {
                "enabled"
            } else {
                "disabled"
            };

            println!("  {} - {} ({})", name, status, enabled);
            println!("    {}", service.description);
            println!();
        }
    }

    fn start_service(&mut self, name: &str) -> Result<()> {
        if let Some(service) = self.services.get_mut(name) {
            if service.running {
                println!("Service {} is already running", name);
                return Ok(());
            }

            println!("Starting {}...", name);

            // In production, would:
            // 1. Fork process
            // 2. Execute service command
            // 3. Track PID
            // 4. Update status

            service.running = true;
            println!("Service {} started", name);
        } else {
            anyhow::bail!("Service not found: {}", name);
        }

        Ok(())
    }

    fn stop_service(&mut self, name: &str) -> Result<()> {
        if let Some(service) = self.services.get_mut(name) {
            if !service.running {
                println!("Service {} is not running", name);
                return Ok(());
            }

            println!("Stopping {}...", name);

            // In production, would:
            // 1. Send SIGTERM to process
            // 2. Wait for graceful shutdown
            // 3. Force kill if needed
            // 4. Update status

            service.running = false;
            service.pid = None;
            println!("Service {} stopped", name);
        } else {
            anyhow::bail!("Service not found: {}", name);
        }

        Ok(())
    }

    fn restart_service(&mut self, name: &str) -> Result<()> {
        println!("Restarting {}...", name);
        self.stop_service(name)?;
        self.start_service(name)?;
        Ok(())
    }

    fn show_status(&self, name: &str) -> Result<()> {
        if let Some(service) = self.services.get(name) {
            println!("Service: {}", service.name);
            println!("Description: {}", service.description);
            println!("Status: {}", if service.running { "running" } else { "stopped" });
            println!("Enabled: {}", if service.enabled { "yes" } else { "no" });

            if let Some(pid) = service.pid {
                println!("PID: {}", pid);
            }

            if let Some(ref exec) = service.exec_stop {
                println!("Stop Command: {}", exec);
            }
        } else {
            anyhow::bail!("Service not found: {}", name);
        }

        Ok(())
    }

    fn enable_service(&mut self, name: &str) -> Result<()> {
        if let Some(service) = self.services.get_mut(name) {
            service.enabled = true;
            println!("Service {} enabled", name);
        } else {
            anyhow::bail!("Service not found: {}", name);
        }

        Ok(())
    }

    fn disable_service(&mut self, name: &str) -> Result<()> {
        if let Some(service) = self.services.get_mut(name) {
            service.enabled = false;
            println!("Service {} disabled", name);
        } else {
            anyhow::bail!("Service not found: {}", name);
        }

        Ok(())
    }

    fn show_logs(&self, name: &str, lines: usize, _follow: bool) -> Result<()> {
        let log_path = PathBuf::from("/var/log").join(format!("{}.log", name));

        if !log_path.exists() {
            println!("No logs found for service: {}", name);
            return Ok(());
        }

        // Read and display last N lines
        let content = fs::read_to_string(&log_path)?;
        let log_lines: Vec<&str> = content.lines().collect();

        let start = if log_lines.len() > lines {
            log_lines.len() - lines
        } else {
            0
        };

        for line in &log_lines[start..] {
            println!("{}", line);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut sm = ServiceManager::new()?;

    match args.command {
        Commands::List { all } => {
            sm.list_services(all);
        }
        Commands::Start { service } => {
            sm.start_service(&service)?;
        }
        Commands::Stop { service } => {
            sm.stop_service(&service)?;
        }
        Commands::Restart { service } => {
            sm.restart_service(&service)?;
        }
        Commands::Status { service } => {
            sm.show_status(&service)?;
        }
        Commands::Enable { service } => {
            sm.enable_service(&service)?;
        }
        Commands::Disable { service } => {
            sm.disable_service(&service)?;
        }
        Commands::Logs { service, lines, follow } => {
            sm.show_logs(&service, lines, follow)?;
        }
    }

    Ok(())
}
