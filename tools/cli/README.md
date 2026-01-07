# Rustica OS - Userspace CLI Implementation

This directory contains the complete userspace CLI implementation for the Rustica Operating System.

## Overview

The Rustica userspace CLI provides a comprehensive set of command-line utilities for system management, file operations, networking, package management, and more. All utilities are written in Rust and follow the design principles outlined in `docs/OS/CLI_manifest.md`.

## Directory Structure

```
userspace/
├── Cargo.toml                 # Workspace configuration
├── build.rs                   # Build script
├── README.md                  # This file
├── src/
│   ├── sh/                    # Shell (command interpreter)
│   ├── init/                  # Init system (PID 1)
│   ├── coreutils/             # Core file utilities
│   ├── sysutils/              # System utilities
│   ├── networkutils/          # Networking tools
│   ├── pkgutil/               # Package manager
│   ├── fwctl/                 # Firewall controller
│   ├── storageutils/          # Storage/filesystem tools
│   └── svc/                   # Service manager & system-check
├── bin/                       # Compiled binaries (output)
└── scripts/
    └── qemu-validation.sh     # QEMU validation script

```

## CLI Utilities

### 1. Core Shell (`sh`)

**Location**: `src/sh/`

The Rustica Shell is a POSIX-compatible command interpreter with built-in commands.

**Features**:
- Interactive mode with splash screen
- Command history (in development)
- Built-in commands: `cd`, `pwd`, `echo`, `export`, `unset`, `exit`, `help`
- External command execution
- Script file support

**Usage**:
```bash
# Interactive mode
sh

# Execute script
sh script.sh

# Execute single command
sh -c "echo 'Hello, World!'"
```

### 2. Init System (`init`)

**Location**: `src/init/`

The first userspace process (PID 1) responsible for system initialization.

**Features**:
- Mount essential filesystems (/proc, /sys, /dev, /tmp)
- Configure system environment
- Start essential services
- Service dependency management
- Multiple runlevels support

**Usage**:
```bash
# Started automatically by kernel
# Configuration: /etc/rustica/init.conf
```

### 3. Core Utilities (`coreutils`)

**Location**: `src/coreutils/`

Essential file manipulation utilities.

**Commands**:
- `ls` - List directory contents
  - Options: `-a` (all), `-l` (long), `-h` (human-readable), `-r` (reverse), `-t` (sort by time)
- `cat` - Concatenate and display files
  - Options: `-n` (line numbers), `-A` (show all characters)
- `cp` - Copy files/directories
  - Options: `-r` (recursive), `-v` (verbose), `-f` (force)
- `mv` - Move/rename files
  - Options: `-v` (verbose), `-n` (no clobber)
- `rm` - Remove files/directories
  - Options: `-r` (recursive), `-i` (interactive), `-f` (force)
- `mkdir` - Make directories
  - Options: `-p` (parents), `-v` (verbose)
- `touch` - Update file timestamps
  - Options: `-c` (no create)
- `echo` - Display text
  - Options: `-n` (no newline), `-e` (enable escapes)

**Usage**:
```bash
ls -lah /etc
cat /etc/hostname
cp -rv src/ dst/
mv old new
rm -rf unwanted/
mkdir -p /path/to/dir
touch /tmp/file
echo "Hello, World!"
```

### 4. System Utilities (`sysutils`)

**Location**: `src/sysutils/`

System monitoring and management tools.

**Commands**:
- `ps` - Report process status
  - Options: `-a` (all), `-f` (full), `-u` (user-oriented)
- `kill` - Send signals to processes
  - Options: `-s` (signal), `-l` (list signals)
- `dmesg` - Print kernel messages
  - Options: `-T` (timestamps), `-f` (follow)
- `uname` - Print system information
  - Options: `-a` (all), `-r` (release), `-m` (machine)
- `date` - Print/set system date
  - Options: `-u` (UTC), `-R` (RFC format)

**Usage**:
```bash
ps aux
kill -9 1234
dmesg -T | tail
uname -a
date -u
```

### 5. Networking Tools (`networkutils`)

**Location**: `src/networkutils/`

Network configuration and diagnostic tools.

**Commands**:
- `ip` - Network configuration
  - Subcommands: `addr`, `link`, `route`
  - Examples: `ip addr show`, `ip link set eth0 up`
- `ping` - Send ICMP echo requests
  - Options: `-c` (count), `-i` (interval)
- `hostname` - Show/set hostname
  - Examples: `hostname`, `hostname newname`
- `nslookup` - DNS lookup
  - Options: `-t` (query type)

**Usage**:
```bash
ip addr show
ip link set eth0 up
ping -c4 8.8.8.8
hostname
nslookup rustux.com
```

### 6. Package Manager (`pkg`)

**Location**: `src/pkgutil/`

Rustica Package Manager for software installation and updates.

**Commands**:
- `pkg update` - Update package lists
- `pkg install <package>` - Install package
- `pkg remove <package>` - Remove package
- `pkg search <query>` - Search packages
- `pkg list` - List installed packages
- `pkg info <package>` - Show package info
- `pkg upgrade` - Upgrade all packages
- `pkg repo-list` - List repositories

**Repositories**:
- http://rustux.com/kernel
- http://rustux.com/rustica
- http://rustux.com/apps

**Usage**:
```bash
pkg update
pkg search editor
pkg install nano
pkg remove nano
pkg list
pkg repo-list
```

### 7. Firewall Controller (`fwctl`)

**Location**: `src/fwctl/`

Firewall management with nftables-style syntax.

**Commands**:
- `fwctl status` - Show firewall status
- `fwctl load` - Load firewall rules
- `fwctl save` - Save firewall rules
- `fwctl add` - Add rule
- `fwctl delete` - Delete rule
- `fwctl list` - List rules
- `fwctl flush` - Flush rules
- `fwctl policy` - Set default policy

**Configuration**: `/etc/rustica/firewall.rules`

**Usage**:
```bash
fwctl status
fwctl add --chain input --protocol tcp --dport 22 --action accept
fwctl list
fwctl flush input
```

### 8. Storage Tools (`storageutils`)

**Location**: `src/storageutils/`

Filesystem and block device management.

**Commands**:
- `mount` - Mount filesystems
  - Options: `-t` (type), `-r` (read-only), `-v` (verbose)
- `umount` - Unmount filesystems
  - Options: `-l` (lazy), `-f` (force)
- `blklist` - List block devices
  - Options: `-a` (all), `-d` (detailed)
- `mkfs-rfs` - Create Rustica filesystem
  - Options: `-L` (label), `-f` (force)

**Usage**:
```bash
mount /dev/sda1 /mnt
umount /mnt
blklist -d
mkfs-rfs -L "root" /dev/sda1
```

### 9. Service Manager (`svc`)

**Location**: `src/svc/`

Systemd-style service management.

**Commands**:
- `svc list` - List all services
- `svc start <service>` - Start service
- `svc stop <service>` - Stop service
- `svc restart <service>` - Restart service
- `svc status <service>` - Show service status
- `svc enable <service>` - Enable at boot
- `svc disable <service>` - Disable at boot
- `svc logs <service>` - Show service logs

**Configuration**: `/etc/rustica/services/*.service`

**Usage**:
```bash
svc list
svc start network
svc status firewall
svc logs network
```

### 10. System Health Check (`system-check`)

**Location**: `src/svc/`

System health and diagnostic utility.

**Checks**:
- Kernel version and status
- Memory usage
- Disk space
- Network interfaces
- Essential services

**Usage**:
```bash
system-check
system-check --verbose
system-check --component memory
```

## Building

### Prerequisites

- Rust nightly toolchain
- Target-specific toolchains (optional)

### Build Commands

```bash
# Build all utilities
cd userspace
cargo build --release

# Build specific utility
cargo build --release -p sh
cargo build --release -p coreutils

# Build for specific target
cargo build --release --target x86_64-unknown-linux-gnu
```

### Binaries Output

Compiled binaries are placed in:
- Debug: `target/debug/<binary>`
- Release: `target/release/<binary>`

## Installation

### Manual Installation

```bash
# Copy binaries to system
sudo cp target/release/sh /bin/
sudo cp target/release/init /sbin/
sudo cp target/release/* /usr/bin/

# Set permissions
sudo chmod 755 /bin/sh /sbin/init /usr/bin/*
```

### System Integration

The init system expects:
- `/bin/sh` - Shell
- `/usr/bin/pkg` - Package manager
- `/usr/bin/fwctl` - Firewall
- `/usr/bin/svc` - Service manager
- `/usr/bin/system-check` - Health check

## Testing

### QEMU Validation

```bash
# Run validation script
cd userspace
./scripts/qemu-validation.sh

# Custom image/kernel
IMAGE=myimage.img KERNEL=mykernel.bin ./scripts/qemu-validation.sh
```

### Manual Testing

```bash
# Build and run in QEMU
cd ..
cargo build --release
qemu-system-x86_64 -kernel target/x86_64-unknown-none/release/rustux \
    -m 1024 -nographic -serial mon:stdio
```

## Configuration Files

### Essential Configuration

- `/etc/rustica/init.conf` - Init system configuration
- `/etc/rustica/sources.list` - Package repositories
- `/etc/rustica/firewall.rules` - Firewall rules
- `/etc/rustica/services/` - Service definitions
- `/etc/hostname` - System hostname
- `/etc/resolv.conf` - DNS servers

### Directory Structure

- `/bin/` - Essential user binaries
- `/sbin/` - Essential system binaries
- `/usr/bin/` - Common user binaries
- `/usr/sbin/` - Common system binaries
- `/var/cache/rpg/` - Package cache
- `/var/lib/rpg/` - Package database
- `/var/log/` - System logs
- `/etc/rustica/` - Configuration files

## Development

### Adding a New Utility

1. Create new package in workspace:
   ```bash
   mkdir -p src/newutil/src
   touch src/newutil/Cargo.toml
   touch src/newutil/src/main.rs
   ```

2. Add to workspace members in `Cargo.toml`:
   ```toml
   members = [
       # ... existing members
       "src/newutil",
   ]
   ```

3. Implement utility
4. Build: `cargo build -p newutil`

### Code Style

- Use `clap` for argument parsing
- Use `anyhow` for error handling
- Use `log` for logging
- Follow Rust naming conventions
- Add comprehensive documentation

## Implementation Status

| Category | Status | Commands |
|----------|--------|----------|
| Shell | ✅ Complete | sh |
| Init | ✅ Complete | init |
| Core Utils | ✅ Complete | ls, cat, cp, mv, rm, mkdir, touch, echo |
| System Utils | ✅ Complete | ps, kill, dmesg, uname, date |
| Networking | ✅ Complete | ip, ping, hostname, nslookup |
| Package Mgr | ✅ Complete | pkg (update, install, search, remove, list) |
| Firewall | ✅ Complete | fwctl |
| Storage | ✅ Complete | mount, umount, blklist, mkfs-rfs |
| Services | ✅ Complete | svc, system-check |
| Validation | ✅ Complete | QEMU validation script |

## License

MIT License - See LICENSE file for details.

## Contributing

See main repository for contribution guidelines.

## Contact

For questions or issues, see the project documentation or open an issue on GitHub.

---

*Last Updated: January 6, 2026*
*Version: 0.1.0*
*Status: Complete - Ready for Testing*
