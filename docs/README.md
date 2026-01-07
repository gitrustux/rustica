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

### Package Manager Integration
- **Flatpak**: Flatpak integration for third-party apps
- **AppImage**: AppImage support for portable applications
- **System Package Manager**: Native `rpg` integration

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
| `rpg` | Package manager with ed25519 digital signatures | `cli/rpg/` |
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

**Package Manager (`rpg`)**:
- Repository management and package installation
- SHA256 checksum verification
- **ed25519 digital signature verification**
- Key generation, import, export, trust management
- **Capability-aware package installation**
- Commands: `update`, `install`, `remove`, `search`, `upgrade`, `list`, `info`, `keygen`, `export-key`, `import-key`, `sign-package`, `list-keys`

**System Installer (`rustux-install`)**:
- Filesystem selection: **ext4**, **F2FS** (mobile-optimized), **btrfs**
- Profile-based package installation (Desktop, Laptop, Mobile, Server, Minimal)
- **F2FS auto-selection for mobile devices**
- Network configuration
- Device-type detection (desktop, laptop, mobile)
- UEFI boot support with multi-boot configuration

---

## 🎯 Key Features

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
│   │   ├── rpg/          # Package manager
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
    └── todo.md
```

---

## 🚀 Building

### CLI Tools
```bash
# Build all CLI tools for native architecture
cargo build --release --workspace

# Build specific tool
cargo build --release -p rustux-rpg
cargo build --release -p rustux-capctl
cargo build --release -p rustux-sbctl
cargo build --release -p rustux-bootctl

# Build for specific architecture (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# Build all tools for all architectures
./build-all-archs.sh
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
rpg update
rpg install vim
rpg list
rpg info vim
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

*Last updated: January 7, 2025*

**Status**: ✅ All phases (0-12) complete - Production ready
