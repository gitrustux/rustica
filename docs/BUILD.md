# Building Rustux OS Live USB

This document describes how to build a bootable Rustux OS live USB image from source.

---

## Overview

Rustux OS uses **direct UEFI boot** - no GRUB, no Linux kernel, no external bootloader. The kernel is a standalone UEFI application that:

1. UEFI firmware loads `BOOTX64.EFI`
2. Kernel initializes hardware (drivers, memory, scheduler)
3. Init process (PID 1) spawns shell (PID 2)
4. User interacts with the interactive shell

**Boot Flow:**
```
UEFI firmware → BOOTX64.EFI → kernel → init (PID 1) → shell (PID 2)
```

---

## Prerequisites

### Build Tools

```bash
# Rust toolchain (UEFI target)
rustup target add x86_64-unknown-uefi

# GCC for cross-compiling userspace C programs
apt install gcc-x86-64-linux-gnu

# Image creation tools
apt install parted dosfstools coreutils
```

### System Requirements

- **Architecture**: x86_64 (AMD64)
- **OS**: Linux (recommended) or macOS
- **Disk Space**: 500 MB for build artifacts
- **RAM**: 4 GB recommended for compilation

---

## Project Layout

```
/var/www/rustux.com/prod/
├── rustux/                 # Kernel (UEFI application)
│   ├── src/                # Kernel source code
│   ├── build.rs            # Embed ramdisk with userspace binaries
│   ├── test-userspace/     # C programs for shell, init, tests
│   ├── build-live-image.sh # Live USB build script
│   └── target/
│       └── x86_64-unknown-uefi/
│           └── release/
│               └── rustux.efi    # Built kernel binary
└── rustica/                # Userspace OS distribution
    └── docs/               # Documentation (this file)
```

---

## Build Steps

### Step 1: Build the Kernel

```bash
cd /var/www/rustux.com/prod/rustux

# Build UEFI kernel with release optimizations
cargo build --release --bin rustux --features uefi_kernel --target x86_64-unknown-uefi
```

**Output:** `target/x86_64-unknown-uefi/release/rustux.efi`

### Step 2: Build Userspace Programs

```bash
cd /var/www/rustux.com/prod/rustux/test-userspace

# Build shell (C program, static linking, no stdlib)
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector shell.c -o ../target/shell.elf

# Build init (first userspace process)
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector init.c -o ../target/init.elf

# Build test programs (optional)
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector hello.c -o ../target/hello.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector counter.c -o ../target/counter.elf
```

**Note:** The kernel's `build.rs` automatically embeds these binaries into the ramdisk during compilation.

### Step 3: Build Live USB Image

```bash
cd /var/www/rustux.com/prod/rustux

# Make build script executable
chmod +x build-live-image.sh

# Build image (uses default version 0.1.0)
./build-live-image.sh

# Or specify a custom version
RUSTUX_VERSION=1.0.0 ./build-live-image.sh
```

**Output:** `/var/www/rustux.com/html/rustica/rustica-live-amd64-{VERSION}.img`

---

## Image Specifications

| Property | Value |
|----------|-------|
| **Format** | Raw disk image |
| **Partition Table** | GPT |
| **Partition 1** | EFI System Partition (ESP) |
| **Filesystem** | FAT32 |
| **Size** | 128 MB (sparse) |
| **Boot Path** | `/EFI/BOOT/BOOTX64.EFI` |
| **Boot Method** | UEFI direct boot |

### Directory Structure (Inside Image)

```
[EFI System Partition]
└── EFI/
    └── BOOT/
        └── BOOTX64.EFI    (rustux.efi - the kernel)
```

### Embedded Ramdisk (Inside Kernel)

The kernel binary contains an embedded ramdisk with:

```
/bin/
├── init       (PID 1 - first process)
├── shell      (PID 2 - interactive shell)
├── hello      (test program)
└── counter    (test program)
```

---

## Build Script Details

The `build-live-image.sh` script performs the following:

1. **Validate kernel build** - Check if `rustux.efi` exists
2. **Create sparse disk image** - 128 MB disk image
3. **Create GPT partition table** - Single ESP partition
4. **Format as FAT32** - UEFI-compatible filesystem
5. **Mount partition** - Using loop device
6. **Copy kernel** - As `/EFI/BOOT/BOOTX64.EFI`
7. **Generate checksum** - SHA256 for verification
8. **Output to directory** - `/var/www/rustux.com/html/rustica/`

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUSTUX_VERSION` | `0.1.0` | Version string for image filename |
| `OUTPUT_DIR` | `/var/www/rustux.com/html/rustica` | Output directory for images |
| `RUSTUX_ARCH` | `amd64` | Architecture identifier |

---

## Output Files

After building, you will have:

```
/var/www/rustux.com/html/rustica/
├── rustica-live-amd64-0.1.0.img         # Bootable disk image
├── rustica-live-amd64-0.1.0.img.sha256  # SHA256 checksum
└── rustica-live-amd64.img               # Symlink to versioned image
```

### Verify Checksum

```bash
cd /var/www/rustux.com/html/rustica
sha256sum -c rustica-live-amd64-0.1.0.img.sha256
```

---

## Writing to USB

### Linux

```bash
# Identify your USB device (e.g., /dev/sdb, /dev/sdc)
lsblk

# Write the image to USB (replace /dev/sdX with your device)
sudo dd if=rustica-live-amd64.img of=/dev/sdX bs=4M status=progress conv=fsync

# Sync and eject
sudo sync
sudo eject /dev/sdX
```

### macOS

```bash
# Identify your USB disk (e.g., /dev/disk2)
diskutil list

# Unmount the disk
diskutil unmountDisk /dev/disk2

# Write the image
sudo dd if=rustica-live-amd64.img of=/dev/rdisk2 bs=4m status=progress

# Eject
diskutil eject /dev/disk2
```

### Windows

Use [Rufus](https://rufus.ie/) or [BalenaEtcher](https://www.balena.io/etcher/):

1. Download and install Rufus or Etcher
2. Select the `rustica-live-amd64.img` file
3. Select your USB drive
4. Click "Flash" or "Start"
5. Wait for completion

---

## Booting from USB

1. Insert the USB drive
2. Restart your computer
3. Enter the boot menu (usually F12, F2, Del, or Esc key)
4. Select the USB drive as boot device
5. System will boot into the Rustux shell

**Expected Boot Sequence:**

```
[UEFI Firmware]
    ↓
[BOOTX64.EFI loaded]
    ↓
[Rustux kernel initializing...]
    ↓
[Init process (PID 1) starting...]
[Spawning shell...]
    ↓
[rustux> _]  (Interactive shell prompt)
```

---

## Troubleshooting

### Kernel Build Fails

**Error:** `target not found: x86_64-unknown-uefi`

**Fix:**
```bash
rustup target add x86_64-unknown-uefi
```

### Userspace Build Fails

**Error:** `x86_64-linux-gnu-gcc: command not found`

**Fix:**
```bash
sudo apt install gcc-x86-64-linux-gnu
```

### Image Creation Fails

**Error:** `parted: command not found`

**Fix:**
```bash
sudo apt install parted dosfstools coreutils
```

### USB Won't Boot

**Possible causes:**
1. **Legacy BIOS mode** - Ensure UEFI boot is enabled in firmware settings
2. **Secure Boot** - Disable Secure Boot (kernel is not signed)
3. **Wrong USB port** - Try a different USB port
4. **Corrupted image** - Verify SHA256 checksum

### Shell Not Appearing

**Check:**
1. PS/2 keyboard is connected (USB HID support planned for Phase 7)
2. Monitor is connected to primary GPU output
3. System has sufficient RAM (512 MB minimum)

---

## Development Workflow

For rapid development iteration:

```bash
#!/bin/bash
# dev-build.sh - Quick rebuild and test script

cd /var/www/rustux.com/prod/rustux

# Rebuild kernel
cargo build --release --bin rustux --features uefi_kernel --target x86_64-unknown-uefi

# Rebuild userspace
cd test-userspace
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector shell.c -o ../target/shell.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector init.c -o ../target/init.elf
cd ..

# Rebuild image
./build-live-image.sh

# Optional: Test in QEMU (if available)
# qemu-system-x86_64 -drive file=rustica-live-amd64.img,format=raw -bios /usr/share/OVMF/OVMF_CODE.fd
```

---

## System Requirements

### Minimum Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **Architecture** | x86_64 (AMD64) | x86_64 (AMD64) |
| **RAM** | 512 MB | 1 GB |
| **Storage** | 128 MB (USB) | 4 GB |
| **Boot** | UEFI 2.0 | UEFI 2.3+ |
| **Input** | PS/2 Keyboard | PS/2 or USB HID* |

\* USB HID support planned for Phase 7

### Supported Features (Phase 6)

- ✅ UEFI boot with ExitBootServices
- ✅ PS/2 keyboard driver
- ✅ Framebuffer text console
- ✅ Process management & scheduler
- ✅ Syscall interface (read, write, spawn, exit)
- ✅ Embedded ramdisk filesystem
- ✅ Interactive shell with Dracula theme

### Planned Features (Phase 7)

- ⏳ USB HID keyboard/mouse driver
- ⏳ Minimal GUI with window manager
- ⏳ Networking stack
- ⏳ Ext2/3 filesystem support

---

## Architecture Notes

### Why Direct UEFI Boot?

Traditional Linux distributions use a multi-stage boot process:

```
UEFI → GRUB → Linux kernel → initramfs → init → userspace
```

Rustux OS eliminates the middle stages:

```
UEFI → Rustux kernel (with embedded userspace) → shell
```

**Benefits:**
- **Simpler** - No bootloader configuration
- **Faster** - No chain-loading delays
- **Smaller** - No external bootloader binaries
- **Self-contained** - Kernel + userspace in one binary

### Ramdisk Embedding

The `build.rs` script embeds userspace binaries directly into the kernel binary:

```rust
// build.rs excerpt
let potential_elf_files = vec![
    ("target/shell.elf", "bin/shell"),
    ("target/init.elf", "bin/init"),
    ("target/hello.elf", "bin/hello"),
    ("target/counter.elf", "bin/counter"),
];
```

At runtime, the kernel extracts these files into a virtual ramdisk.

### Dracula Theme (MANDATORY INVARIANT)

The Dracula color palette is the default system theme and must survive:

- Kernel rebuilds
- CLI refactors
- Framebuffer rewrites
- GUI introduction later

**Canonical Dracula Colors:**
```
FG_DEFAULT = #F8F8F2  (r: 248, g: 248, b: 242)
BG_DEFAULT = #282A36  (r: 40, g: 42, b: 54)
CYAN       = #8BE9FD  (r: 139, g: 233, b: 253)
PURPLE     = #BD93F9  (r: 189, g: 147, b: 249)
GREEN      = #50FA7B  (r: 80, g: 250, b: 123)
RED        = #FF5555  (r: 255, g: 85, b: 85)
ORANGE     = #FFB86C  (r: 255, g: 184, b: 108)
YELLOW     = #F1FA8C  (r: 241, g: 250, b: 140)
```

---

## Support

For more information:

- **Documentation**: See `IMAGE.md` for system architecture
- **Roadmap**: See `PLAN.md` for development phases
- **Repository**: https://github.com/gitrustux/rustux
- **Issues**: https://github.com/gitrustux/rustux/issues

---

## License

Copyright (c) 2025 The Rustux Authors

Licensed under the MIT License. See LICENSE file for details.
