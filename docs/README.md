# Rustica OS

A modern Linux distribution built on Rust, featuring a complete Wayland desktop environment, mobile support, and advanced security capabilities.

## Directory Structure

- `releases/` - Official releases (CLI, Desktop, Server)
- `repo/` - Package repository (system, apps, kernel, metadata)
- `images/` - Installation images (installer, live, VM)
- `docs/` - Documentation
- `tools/` - Build and deployment tools

---

## Implementation Status (January 2025)

### ✅ Completed: Core System (Phases 0-12)

All high and medium priority system components are complete, including CLI utilities, GUI desktop environment, mobile support, and comprehensive documentation.

---

## 🖥️ Graphical User Interface (Phases 0-12)

### Desktop Environment: Rustica Shell

A complete Wayland-based desktop environment built with Rust and Smithay.

#### Wayland Compositor (`rustica-comp`)
- **Location**: `/var/www/rustux.com/prod/apps/gui/rustica-comp/`
- **Features**:
  - Smithay-based Wayland compositor
  - DRM/KMS backend for GPU acceleration
  - libinput for input device handling
  - Multi-monitor support
  - XDG shell protocol support
  - Layer shell for panels/docks
- **Protocols**: Wayland core, XDG shell, layer shell, viewporter, pointer constraints, tablet support

#### Shell Components
- **Panel** (`rustica-panel`): Top panel with app menu, system tray, status indicators
- **Dock** (`rustica-dock`): Application launcher and switcher with drag-and-drop
- **App Launcher** (`rustica-launcher`): Grid-based application launcher
- **Workspace Manager** (`rustica-workspaces`): Multiple virtual workspaces with dynamic switching
- **Notification System** (`rustica-notifications`): Modern notification daemon

#### Desktop Applications
- **Terminal** (`rustica-term`): VT100/ANSI terminal emulator with tabs
- **File Manager** (`rustica-files`): Modern file browser with thumbnail generation
- **Text Editor** (`rustica-edit`): Lightweight text editor with syntax highlighting
- **Screenshot Tool** (`rustica-screenshot`): Screen capture with region selection
- **Settings** (`rustica-settings`): System configuration application
- **Application Library** (`rustica-applibrary`): App management and discovery

#### Web Browser (`rustica-web`) ✨ NEW
- **Location**: `/var/www/rustux.com/prod/apps/gui/rustica-web/`
- **Based on**: WebKitGTK 2.50+
- **Features**:
  - Modern tabbed browsing interface
  - Privacy-focused (DuckDuckGo search, no tracking)
  - **Mobile Mode**: Auto-detects touch devices and small screens
  - **Desktop UI**: Traditional navigation bar with back/forward/refresh
  - **Mobile UI**: Touch-optimized with bottom navigation bar
  - **Mobile User Agent**: Android 13 user agent for mobile sites
  - JavaScript, WebGL, and hardware acceleration enabled
  - Developer tools integration
- **Mobile Features**:
  - Touch gesture support (swipe navigation, pinch-to-zoom)
  - Kinetic scrolling
  - Spatial navigation (D-pad support)
  - Larger touch targets (40px minimum)
  - Responsive viewport meta tag handling
- **Usage**:
  ```bash
  # Desktop mode (auto-detected)
  rustica-web

  # Force mobile mode
  RUSTICA_MOBILE_MODE=true rustica-web

  # Touch device mode
  RUSTICA_TOUCH_DEVICE=true rustica-web
  ```

---

## 📱 Mobile Support (Phases 7-8)

### Mobile-Optimized Components

#### Login Greeter (`rustica-greeter`)
- Touch-friendly login interface
- Mobile-optimized virtual keyboard integration
- User session selection

#### Initial Setup (`rustica-initial-setup`)
- First-boot configuration wizard
- Touch-optimized setup flow
- Mobile device calibration

#### Mobile Features
- **Touch Gesture System**: Swipe, pinch, tap gestures
- **On-Screen Keyboard**: Full virtual keyboard with layouts
- **Mobile UI Components**: Touch-optimized widgets
- **Sensor Integration**: Accelerometer, gyroscope, ambient light
- **Battery Optimization**: Power management for mobile devices

---

## 🔧 System Integration (Phase 7)

### Desktop Portal (`xdg-desktop-portal-rustica`)
- File picker dialogs
- Screenshot permissions
- Camera/microphone access
- Wallpaper access
- Screen casting

### Theme Engine
- Material Design 3 theming
- Dark/light mode support
- Custom accent colors
- Font configuration

### Session Manager
- User session startup
- Application autostart
- Session restoration

---

## 📦 Package Management (Phase 9)

### Live Update System - RPG (Rustica Package Manager)

Rustica OS features a comprehensive live update system built from scratch in Rust, enabling atomic package operations, full rollback support, and background updates without system interruption.

#### System Architecture

**Location**: `/var/www/rustux.com/prod/rustica/update-system/`

```
update-system/
├── rpg-core/              # Core library
│   ├── lib.rs           # Main entry point
│   ├── archive.rs        # .rpg package format
│   ├── ops.rs            # High-level operations
│   ├── fetch.rs          # HTTP downloading with failover
│   ├── sources.rs        # Repository source management
│   ├── transaction.rs    # Transaction management
│   ├── registry.rs       # Package registry
│   ├── layout.rs         # Filesystem layout
│   ├── signature.rs      # Ed25519 cryptographic signing
│   ├── package.rs        # Package types
│   ├── version.rs        # Semantic versioning
│   └── symlink.rs        # Atomic symlink operations
├── rpg/                  # CLI tool
│   └── src/bin/rpg.rs    # Command implementation
└── update-daemon/        # Background service
    └── src/main.rs       # Update daemon
```

#### Package Format (.rpg)

RPG packages are tar.gz archives with the following structure:

```
package.rpg
├── metadata.json          # Package manifest
├── files/                 # Actual files to install
│   ├── usr/
│   ├── bin/
│   └── ...
├── scripts/               # Installation scripts (optional)
│   ├── pre-install.sh
│   ├── post-install.sh
│   └── pre-remove.sh
└── signature.sig          # Detached Ed25519 signature
```

**metadata.json structure:**
```json
{
  "name": "example-app",
  "version": "1.2.3",
  "type": "app",
  "arch": "x86_64",
  "description": "An example application",
  "size": 1048576,
  "sha256": "abc123...",
  "dependencies": ["libfoo >= 1.0"],
  "files": ["usr/bin/app", "usr/share/app/..."],
  "signature": "base64-encoded-signature"
}
```

#### Filesystem Layout

Versioned packages are installed to separate directories:

```
/system/
├── v1.0.0/           # System version 1.0.0
│   ├── usr/
│   ├── bin/
│   └── ...
├── v1.1.0/           # System version 1.1.0
├── current -> v1.1.0 # Active version (symlink)

/apps/
├── example-app/
│   ├── 1.0.0/       # App version 1.0.0
│   │   ├── bin/
│   │   └── share/
│   ├── 1.1.0/       # App version 1.1.0
│   └── current -> 1.1.0
```

#### Key Features

1. **Atomic Operations**
   - Never overwrites active files
   - Versioned directory layout
   - Atomic symlink swaps for activation
   - POSIX `rename()` ensures atomicity

2. **Rollback Support**
   - Previous versions retained until explicitly removed
   - Transaction history tracking
   - One-command rollback to any previous version
   - Automatic rollback on failure

3. **Live Updates**
   - Applications update while running
   - Userland updates without interruption
   - Kernel updates installed alongside, activated on reboot
   - Background download capability

4. **Cryptographic Security**
   - Ed25519 digital signatures
   - SHA-256 checksum verification
   - Public key infrastructure
   - Signature verification before installation

5. **Multiple Source Support**
   - Configurable repository sources
   - Priority-based source selection
   - Automatic failover on source failure
   - Per-source enable/disable

#### Configuration

**Sources Configuration** (`/etc/rpg/sources.list`):
```
# Rustica Package Sources
# Format: type url [priority]
# Types: kernel, system, apps

kernel http://rustux.com/kernel 10
system http://rustux.com/rustica 10
apps http://rustux.com/apps 10

# Mirror sources (higher priority = preferred)
kernel-mirror http://mirror.example.com/kernel 5
```

**Source Types:**
- `kernel` - Kernel packages (require reboot)
- `system` - System userland packages
- `apps` - Application packages

#### CLI Commands

```bash
# Check for updates
rpg update --check-only

# Install all updates
rpg update

# Install updates in background
rpg update --background

# Install specific package
rpg install <package>

# Install specific version
rpg install <package> --version 1.2.3

# Remove package
rpg remove <package>

# Rollback to previous version
rpg rollback <package>

# Rollback system
rpg rollback system

# Show status
rpg status

# Show detailed status
rpg status --detailed

# List packages
rpg list

# List with filter
rpg list --pattern "web" --kind app

# Manage sources
rpg sources list
rpg sources add mirror https://mirror.example.com/apps 50
rpg sources remove mirror
rpg sources enable mirror
rpg sources disable mirror
rpg sources check
```

#### Package Manager API

**High-level operations:**
```rust
use rpg_core::PackageManager;

let manager = PackageManager::new()?;

// Check for updates
let updates = manager.check_updates().await?;

// Install package
manager.install_package("app", Some("1.0.0"), PackageKind::App).await?;

// Update all packages
let result = manager.update_all().await?;

// Rollback
manager.rollback("app", None).await?;

// List installed
let packages = manager.list_installed().await?;

// Get status
let status = manager.get_status().await?;
```

#### Transaction Management

All package operations run within transactions:

```rust
use rpg_core::{Transaction, TransactionKind};

// Transaction automatically:
// 1. Creates versioned directory
// 2. Extracts package files
// 3. Updates metadata
// 4. Performs atomic symlink swap
// 5. Records rollback information
// 6. On failure: automatically rolls back
```

**Transaction Results:**
- `Success { activated, requires_reboot }` - Installation succeeded
- `Failed { error, partial }` - Installation failed with partial installation
- `RolledBack { reason }` - Transaction was rolled back

#### Package Creation

Create an `.rpg` package:

```rust
use rpg_core::{PackageManifest, PackageArchive, create_package};
use rpg_core::{PackageKind, signature::KeyPair};

// Generate signing key
let key = KeyPair::generate();
let signature = key.sign(b"package-data");

// Create manifest
let manifest = PackageManifest::new(
    "myapp".to_string(),
    "1.0.0".to_string(),
    PackageKind::App,
    "x86_64".to_string(),
    1024 * 1024, // size
    sha256_hash,
    "https://rustux.com/apps/myapp-1.0.0.rpg".to_string(),
    signature,
);

// Create package
create_package(
    "/path/to/source/files",
    "/output/myapp-1.0.0.rpg",
    manifest,
)?;
```

#### Registry

Package installation state tracked in `/var/lib/rpg/registry.json`:

```json
{
  "packages": {
    "myapp": [
      {"major": 1, "minor": 0, "patch": 0, "pre": null, "build": null},
      {"major": 0, "minor": 9, "patch": 0, "pre": null, "build": null}
    ]
  },
  "active": {
    "myapp": {"major": 1, "minor": 0, "patch": 0}
  },
  "transactions": [...],
  "pending": []
}
```

#### Update Daemon

Background service (`rustic-update-daemon`) for:

- Periodic update checks
- Background downloads
- User preference handling
- Transaction queue management
- System reboot coordination

#### Security Features

1. **Ed25519 Signatures**
   - Each package signed with developer key
   - Public key verification before installation
   - Signature stored in package manifest

2. **SHA-256 Checksums**
   - Computed before download from repository
   - Verified after download
   - Mismatch = automatic rejection

3. **Atomic Operations**
   - No partial states
   - All-or-nothing installation
   - Automatic rollback on failure

4. **Capability Integration**
   - Packages declare capability requirements
   - Verified before installation
   - Integration with `capctl` system

#### Error Handling

Comprehensive error types:
- `Io` - Filesystem errors
- `Serialization` - JSON parse errors
- `Network` - Download failures
- `SignatureVerification` - Invalid signatures
- `PackageNotFound` - Missing packages
- `VersionNotFound` - Missing versions
- `TransactionFailed` - Operation failures
- `RollbackFailed` - Rollback failures
- `Layout` - Filesystem layout issues
- `PermissionDenied` - Permission issues

#### Package Manager Integration

- **Flatpak**: Flatpak integration for third-party apps
- **AppImage**: AppImage support for portable applications
- **Native RPG**: Full `rpg` integration with live updates

---

## 🧪 Testing Framework (Phase 10)

### Complete Test Suite

#### Testing Framework (`Phase 10.1`)
- Unit test framework
- Integration test framework
- Test runners and reporters

#### Integration Tests (`Phase 10.2`)
- System-level testing
- Component integration testing
- End-to-end workflows

#### UI/UX Testing (`Phase 10.3`)
- Automated UI testing
- User interaction simulation
- Visual regression testing

#### Performance Testing (`Phase 10.4`)
- Rendering benchmarks
- Memory profiling
- CPU usage monitoring
- Frame rate testing

#### Accessibility Testing (`Phase 10.5`)
- Screen reader testing
- Keyboard navigation testing
- AT-SPI compliance verification

---

## 📚 Documentation (Phase 11)

### Comprehensive Documentation Suite

#### Developer Documentation (`Phase 11.1`)
- **Location**: `/var/www/rustux.com/prod/apps/gui/docs/documentation/developer.md`
- Getting started guides
- Architecture deep dives
- Development guides (custom widgets, extensions)
- API documentation standards
- Testing and debugging guides
- Contributing guidelines

#### User Documentation (`Phase 11.2`)
- **Location**: `/var/www/rustux.com/prod/apps/gui/docs/documentation/user.md`
- Getting started with Rustica OS
- Desktop tour and component guides
- Application usage instructions
- Customization guides (themes, wallpapers, shortcuts)
- Mobile guide (touch gestures, on-screen keyboard)
- Troubleshooting common issues

#### API Documentation (`Phase 11.3`)
- **Location**: `/var/www/rustux.com/prod/apps/gui/docs/documentation/api.md`
- Rustdoc standards with examples
- D-Bus API documentation for all interfaces
- Wayland protocol extensions
- Data format specifications
- API versioning and deprecation policies

#### Architecture Documentation (`Phase 11.4`)
- **Location**: `/var/www/rustux.com/prod/apps/gui/docs/architecture/system-architecture.md`
- High-level system architecture
- Component interaction diagrams
- Data flow between components
- Threading model and memory management
- Security architecture
- Protocol architecture (Wayland, D-Bus)
- Design decisions and trade-offs
- Performance characteristics

#### Contributing Guidelines (`Phase 11.5`)
- **Location**: `/var/www/rustux.com/prod/apps/gui/docs/contributing.md`
- Getting started setup
- Development workflow
- Code standards (Rust style, performance, security)
- Commit message format (Conventional Commits)
- Pull request process
- Review process
- Testing requirements
- Community guidelines

---

## 🌐 Accessibility & Internationalization (Phase 1)

### Accessibility Framework (`Phase 1.1`)
- AT-SPI 2.0 implementation
- Screen reader support
- Magnifier support
- On-screen keyboard
- High contrast mode

### IME & Multilingual Text (`Phase 1.2`)
- Input Method Framework (ibus/fcitx)
- CJK input methods
- Right-to-left text support
- Input composition

### Hi-DPI & Scaling (`Phase 1.3`)
- Fractional scaling support
- Per-monitor DPI awareness
- Touch scaling optimization

---

## 🏗️ Architecture Specifications (Phase 0)

### Core Design Decisions

- **Display Stack**: Wayland compositor (no X11)
- **Rendering Backend**: GPU-accelerated (EGL/Vulkan)
- **Toolkit Strategy**: Rust + GTK for shell, pure Rust for custom components
- **Process Model**: Multi-process for security (UI, Web, Network, GPU processes)
- **Event Model**: Wayland protocol with Smithay
- **Window Lifecycle**: XDG shell protocol
- **Theme System**: Material Design 3 with CSS-like styling
- **Error Handling**: Panic recovery, crash reporting
- **State Persistence**: JSON-based state storage

---

## 🔒 Security Architecture

### Multi-Process Isolation
- **UI Process**: Main browser/application (sandboxed)
- **Web Process**: Per-tab rendering (sandboxed)
- **Network Process**: HTTP requests (sandboxed)
- **GPU Process**: Graphics/compositing (sandboxed)

### Sandboxing
- Landlock for filesystem access control
- bubblewrap for process isolation
- Seccomp filters for syscall restriction

### Security Policies
- Wayland protocol security (no pointer grabs)
- D-Bus policy filtering
- Portal-based permission requests
- HTTPS-only mode
- Certificate pinning
- Content Security Policy

---

## 📱 Mobile Mode Details

### Detection Methods

The browser automatically detects mobile mode through three methods:

1. **Environment Variable**: `RUSTICA_MOBILE_MODE=true`
2. **Touch Device**: `RUSTICA_TOUCH_DEVICE=true`
3. **Screen Size**: Automatically if screen < 768px width or height

### Desktop vs Mobile UI Comparison

| Feature | Desktop | Mobile |
|---------|---------|--------|
| Navigation | Top bar with URL + buttons | Top bar + bottom bar |
| Button Size | Normal (IconSize::Button) | Large (IconSize::LargeToolbar) |
| Status Bar | Yes | No |
| URL Bar Height | Default | 40px (larger for touch) |
| Touch Gestures | Mouse/keyboard | Swipe, pinch, tap |
| User Agent | Desktop (X11/Linux) | Mobile (Android 13) |
| Scrolling | Standard | Kinetic (momentum) |
| Spatial Navigation | Disabled | Enabled (D-pad) |
| Fullscreen Support | Standard | Enhanced |

### Mobile User Agent

```
Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko)
Chrome/120.0.0.0 Mobile Safari/537.36 RusticaMobile/1.0
```

### Mobile Settings

- ✅ Smooth scrolling enabled
- ✅ Spatial navigation (D-pad support)
- ✅ Media playback without user gesture
- ✅ Fullscreen support
- ✅ Kinetic scrolling
- ✅ Touch-optimized buttons (40px minimum)
- ✅ Swipe navigation (left/right)
- ✅ Pinch-to-zoom support
- ✅ Responsive viewport meta tag handling

---

## 🛠️ CLI Utilities & Core System

### CLI Tools Implemented (18 utilities)

| Tool | Description | Location |
|------|-------------|----------|
| `rpg` | **Live update system** with atomic operations, rollback, Ed25519 signatures | `rustica/update-system/` |
| `pkg-compat` | Backward compatibility wrapper (pkg → rpg) | `cli/pkg-compat/` |
| `svc` | Service manager (systemd-style init) | `cli/svc/` |
| `ip` | Network configuration (netlink-based) | `cli/ip/` |
| `login` | User login utility | `cli/login/` |
| `ping` | ICMP echo utility | `cli/ping/` |
| `fwctl` | Firewall control (nftables frontend) | `cli/fwctl/` |
| `installer` | OS installer with multi-architecture support | `cli/installer/` |
| `tar` | Archive utility with gzip compression | `cli/tar/` |
| `dnslookup` | DNS lookup utility | `cli/dnslookup/` |
| `editor` | Text editor (nano-like) | `cli/editor/` |
| `ssh` | SSH client wrapper | `cli/ssh/` |
| `logview` | System log viewer and crash reporter | `cli/logview/` |
| `capctl` | Capability control - maps kernel objects to file permissions | `cli/capctl/` |
| `sbctl` | Secure boot control - key generation and binary signing | `cli/sbctl/` |
| `bootctl` | UEFI boot entry management | `cli/bootctl/` |
| `apt` | apt compatibility wrapper | `cli/apt/` |
| `apt-get` | apt-get compatibility wrapper | `cli/apt-get/` |

### System Tools

**Capability Control (`capctl`)**:
- 16 capabilities across 7 categories (file, directory, network, system, device, process, package)
- Extended attributes (xattr) based storage
- Commands: `get`, `set`, `remove`, `list`, `from-perms`, `database`, `init-db`

**Secure Boot Control (`sbctl`)**:
- Creates and manages secure boot keys (PK, KEK, DB)
- Signs kernels and bootloaders
- Commands: `create-keys`, `sign`, `verify`, `list-keys`, `export-key`, `sign-kernels`, `status`

**UEFI Boot Management (`bootctl`)**:
- Lists and manages UEFI boot entries
- Detects other operating systems
- Commands: `list`, `set-order`, `add`, `remove`, `set-next-boot`, `detect-os`, `export`, `status`

**Package Manager (`rpg`)** - Live Update System:
- **Location**: `/var/www/rustux.com/prod/rustica/update-system/`
- **Features**:
  - Live updates without system interruption
  - Atomic package operations with versioned filesystem
  - Full rollback support to any previous version
  - Background download and installation
  - Ed25519 digital signature verification
  - SHA-256 checksum verification
  - Multiple repository sources with automatic failover
  - Transaction-based operations (install, remove, upgrade, rollback)
  - Support for kernel, system, and application packages
- **Package Format**: `.rpg` tar.gz archives with metadata.json
- **Filesystem Layout**: Versioned directories (`/system/vX.Y.Z/`, `/apps/<name>/<version>/`)
- **Activation**: Atomic symlink swaps (POSIX rename)
- **Configuration**: `/etc/rpg/sources.list` for repository sources
- **Registry**: `/var/lib/rpg/registry.json` for package state
- **Commands**: `update`, `install`, `remove`, `rollback`, `status`, `list`, `sources`

**System Installer (`rustux-install`)**:
- Filesystem selection: **ext4**, **F2FS** (mobile-optimized), **btrfs**
- Profile-based package installation (Desktop, Laptop, Mobile, Server, Minimal)
- **F2FS auto-selection for mobile devices**
- Network configuration
- Device-type detection (desktop, laptop, mobile)
- UEFI boot support with multi-boot configuration

---

## 🎯 Key Features

### Live Update System
- **Atomic Package Operations**: Never overwrites active files
- **Versioned Filesystem**: `/system/vX.Y.Z/` and `/apps/<name>/<version>/` layout
- **Full Rollback Support**: One-command rollback to any previous version
- **Live Updates**: Applications update while running
- **Background Downloads**: Non-intrusive update downloads
- **Cryptographic Security**: Ed25519 signatures + SHA-256 verification
- **Multiple Sources**: Configurable repositories with automatic failover
- **Transaction-Based**: ACID-like properties for all operations

### Capability-Based Security
- Kernel object model mapped to file permissions
- Packages declare capability requirements
- Capability levels (0-255) for privilege escalation
- Extended attributes for storing file capabilities
- Integration with package manager

### Secure Boot Support
- Complete key infrastructure (PK, KEK, DB)
- Kernel and bootloader signing
- Integration with sbsign/sbverify tools
- ESP key export for firmware enrollment

### Filesystem Options
- **ext4**: Stable, compatible (default for Server/Minimal)
- **F2FS**: Flash-Friendly File System (default for Mobile)
- **btrfs**: Advanced features with snapshots (default for Desktop/Laptop)
- Automatic filesystem recommendation based on profile

### Cross-Compilation Support
- **amd64** (x86_64-unknown-linux-gnu) - ✅ Fully supported
- **arm64** (aarch64-unknown-linux-gnu) - ✅ Built
- **riscv64** (riscv64gc-unknown-linux-gnu) - ✅ Built
- Build script: `build-all-archs.sh`

---

## 📂 Project Structure

```
/var/www/rustux.com/prod/
├── apps/
│   ├── cli/              # CLI applications (18 tools)
│   │   ├── pkg-compat/   # Backward compatibility wrapper
│   │   ├── capctl/       # Capability control
│   │   ├── sbctl/        # Secure boot management
│   │   ├── bootctl/      # UEFI boot management
│   │   ├── installer/    # OS installer
│   │   ├── svc/          # Service manager
│   │   ├── ip/           # Network config
│   │   ├── login/        # Login utility
│   │   ├── ping/         # ICMP echo
│   │   ├── fwctl/        # Firewall control
│   │   ├── tar/          # Archive utility
│   │   ├── dnslookup/    # DNS lookup
│   │   ├── editor/       # Text editor
│   │   ├── ssh/          # SSH client
│   │   ├── logview/      # Log viewer
│   │   ├── apt/          # apt compatibility wrapper
│   │   └── apt-get/      # apt-get compatibility wrapper
│   ├── gui/              # GUI applications
│   │   ├── rustica-comp/ # Wayland compositor
│   │   ├── rustica-web/  # Web browser with mobile mode ✨
│   │   └── docs/         # GUI documentation
│   │       ├── documentation/
│   │       │   ├── developer.md
│   │       │   ├── user.md
│   │       │   └── api.md
│   │       ├── architecture/
│   │       │   └── system-architecture.md
│   │       ├── specifications/
│   │       │   └── browser.md
│   │       └── contributing.md
│   ├── libs/             # Shared libraries
│   │   ├── rutils/       # Rust utilities
│   │   └── netlib/       # Network library
│   ├── Cargo.toml        # Workspace configuration
│   └── build-all-archs.sh
├── kernel/               # RUSTUX microkernel
└── rustica/              # Distribution platform
    ├── docs/
    │   ├── rustica_checklist.txt
    │   └── README.md      # This file
    ├── todo.md
    └── update-system/    # Live update system (RPG) ✨ NEW
        ├── rpg-core/     # Core library
        ├── rpg/          # CLI tool
        └── update-daemon/ # Background service
```

---

## 🚀 Building

### CLI Tools
```bash
# Build all CLI tools for native architecture
cargo build --release --workspace

# Build specific tool
cargo build --release -p rustux-capctl
cargo build --release -p rustux-sbctl
cargo build --release -p rustux-bootctl

# Build for specific architecture (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# Build all tools for all architectures
./build-all-archs.sh
```

### Live Update System (RPG)
```bash
# Navigate to update system directory
cd /var/www/rustux.com/prod/rustica/update-system

# Build all components
cargo build --release

# Build specific component
cargo build --release -p rpg-core
cargo build --release -p rpg
cargo build --release -p update-daemon

# Run the CLI
./target/release/rpg --help
./target/release/rpg status
./target/release/rpg update --check-only

# Run the daemon
./target/release/rustic-update-daemon
```

### GUI Applications
```bash
# Build Wayland compositor
cargo build --release -p rustica-comp

# Build web browser
cargo build --release -p rustica-web

# Binary locations
./target/release/rustica-comp
./target/release/rustica-web
```

### Build Browser for Mobile Mode
```bash
# Normal build (desktop mode, auto-detects screen size)
cargo build --release -p rustica-web

# Force mobile mode
RUSTICA_MOBILE_MODE=1 cargo build --release -p rustica-web
```

---

## 💡 Quick Start Examples

### Package Management
```bash
# Check for available updates
rpg update --check-only

# Update all packages
rpg update

# Install a package
rpg install firefox

# Install specific version
rpg install firefox --version 120.0.0

# List packages
rpg list

# Remove package
rpg remove firefox

# Rollback to previous version
rpg rollback firefox

# Check system status
rpg status --detailed

# Manage repository sources
rpg sources list
rpg sources add mirror https://mirror.example.com/apps 50
```

### Capability Management
```bash
capctl list
capctl get /usr/bin/sudo
capctl set /usr/bin/myapp file_execute,net_bind
```

### Secure Boot
```bash
sbctl status
sbctl create-keys
sbctl sign /boot/vmlinuz-linux
sbctl sign-kernels
```

### UEFI Boot Management
```bash
bootctl list
bootctl detect-os
bootctl set-order 0001,0002,0003
```

### Web Browser
```bash
# Desktop mode (auto-detected based on screen size)
rustica-web

# Force mobile mode (for testing on desktop)
RUSTICA_MOBILE_MODE=true rustica-web

# Force mobile mode with touch detection
RUSTICA_TOUCH_DEVICE=true rustica-web
```

### Installation
```bash
sudo rustux-install --auto --device /dev/sda
sudo rustux-install  # Interactive mode
```

---

## 📖 Documentation

### GUI Documentation
- **Developer Guide**: `gui/docs/documentation/developer.md`
- **User Guide**: `gui/docs/documentation/user.md`
- **API Reference**: `gui/docs/documentation/api.md`
- **Architecture**: `gui/docs/architecture/system-architecture.md`
- **Browser Spec**: `gui/docs/specifications/browser.md`
- **Contributing**: `gui/docs/contributing.md`

### System Documentation
- `rustica_checklist.txt` - Full implementation checklist
- `todo.md` - Task documentation and implementation notes
- Individual tool documentation in respective `cli/*/` directories

---

## 🎨 GUI Features

### Desktop Environment
- **Wayland Compositor**: Modern display server with GPU acceleration
- **Panel**: Top panel with app menu and system tray
- **Dock**: Application launcher and switcher
- **Notifications**: Modern notification system
- **Settings**: System configuration application
- **Terminal**: Feature-rich terminal emulator
- **File Manager**: Modern file browser
- **Text Editor**: Lightweight text editing
- **Screenshot Tool**: Screen capture utility

### Mobile Features
- **Touch Gesture System**: Swipe, pinch, tap
- **On-Screen Keyboard**: Full virtual keyboard
- **Mobile UI Components**: Touch-optimized widgets
- **Sensor Integration**: Accelerometer, gyroscope
- **Battery Optimization**: Power management
- **Mobile Browser**: Touch-optimized web browsing

### Testing & Quality
- **Unit Tests**: Component-level testing
- **Integration Tests**: System-level testing
- **UI/UX Tests**: User interface testing
- **Performance Tests**: Benchmarking and profiling
- **Accessibility Tests**: AT-SPI compliance testing

---

## 🔮 Future Roadmap

While all core functionality is complete, ongoing enhancements include:

- Additional desktop applications (image viewer, music player, etc.)
- Enhanced mobile gestures and optimizations
- Additional accessibility features
- Performance optimizations
- Extended theme options
- Additional language packs
- Cloud integration features

---

## 📄 License

MIT License - See LICENSE file for details

---

## 🙏 Credits

**Rustica OS** is built with:
- Rust programming language
- Smithay (Wayland compositor framework)
- WebKitGTK (browser engine)
- GTK3 (UI toolkit)
- And many other open-source projects

---

*Last updated: January 8, 2026*

**Status**: ✅ All phases (0-12) complete - Production ready
