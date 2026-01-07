# Rustica Applications Repository

This repository contains all applications, libraries, and tools for the Rustica Operating System.

## Structure

### CLI Applications (`cli/`)

Minimal, essential command-line utilities for Rustica OS.

- **redit** - Text editor (Notepad++-like with Rust)
  - Syntax highlighting
  - Multiple tabs
  - File browser integration

- **net-tools** - Networking utilities
  - ping, ip, ifconfig replacements
  - Network diagnostics
  - Configuration tools

- **sys-tools** - System utilities
  - Process management
  - System monitoring
  - Service control

### GUI Applications (`gui/`)

Aurora Desktop Environment applications.

- **aurora-shell** - Desktop shell/compositor
  - Window management
  - Compositing
  - Desktop environment

- **aurora-panel** - Top panel/bar
  - Application menu
  - System tray
  - Status indicators
  - Clock and calendar

- **aurora-launcher** - Application launcher
  - App grid view
  - Search functionality
  - Category filtering

### Libraries (`libs/`)

Shared Rust libraries used by multiple applications.

- **rutils** - Rustica utilities library
  - Common data structures
  - Helper functions
  - System integration

- **rgui** - GUI helpers for Aurora
  - Widget library
  - Theme system
  - Rendering utilities

- **netlib** - Networking utilities
  - HTTP client wrappers
  - DNS resolution
  - Network protocols

### Examples (`examples/`)

Example applications and demonstrations.

### Tests (`tests/`)

Integration tests and CI test suites.

### Scripts (`scripts/`)

Build and deployment scripts.

## Building

### Build All Applications

```bash
# Build everything in the workspace
cargo build --release --workspace

# Build specific category
cargo build --release -p cli/redit
cargo build --release -p gui/aurora-shell
```

### Build Specific Architecture

```bash
# AMD64 (default)
cargo build --release --target x86_64-unknown-linux-gnu

# ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# RISC-V
cargo build --release --target riscv64gc-unknown-linux-gnu
```

### Using Build Scripts

```bash
# Build all applications
./scripts/build-all.sh

# Package applications for distribution
./scripts/package.sh
```

## Installing

### Manual Installation

```bash
# Copy binaries to system
sudo cp target/release/redit /usr/bin/
sudo cp target/release/aurora-shell /usr/bin/
```

### Package Installation

```bash
# Install from package
pkg install redit
pkg install aurora-shell
```

## Development

### Adding a New Application

1. Create directory structure:
   ```bash
   mkdir -p cli/myapp/src
   ```

2. Create `Cargo.toml`:
   ```toml
   [package]
   name = "myapp"
   version.workspace = true
   edition.workspace = true

   [dependencies]
   anyhow.workspace = true
   clap.workspace = true
   ```

3. Add to workspace members in root `Cargo.toml`

4. Build and test:
   ```bash
   cargo build -p myapp
   ```

### Adding a New Library

1. Create library structure:
   ```bash
   mkdir -p libs/mylib/src
   ```

2. Create `Cargo.toml`:
   ```toml
   [package]
   name = "mylib"
   version.workspace = true
   edition.workspace = true

   [dependencies]
   anyhow.workspace = true
   ```

3. Add to workspace members

4. Use in other applications:
   ```toml
   [dependencies]
   mylib = { path = "../../libs/mylib" }
   ```

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### Run Integration Tests

```bash
cd tests
cargo test
```

## Distribution

Applications are built and packaged for:

- **CLI**: `releases/cli/{amd64,arm64,riscv64}/`
- **Desktop**: `releases/desktop/aurora/{amd64,arm64,riscv64}/`
- **Server**: `releases/server/{amd64,arm64,riscv64}/`

## License

MIT License - See LICENSE file for details.

## Contributing

See main repository for contribution guidelines.

---

*Last Updated: January 6, 2026*
*Version: 0.1.0*
