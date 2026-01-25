# Rustux OS - Bootable Live Image

This document describes the Rustux OS live USB image, including system architecture, build instructions, and usage.

**Status:** 🟢 Phase 7A IN PROGRESS | USB HID Keyboard with xHCI Transfers (Polling)

---

## Table of Contents

- [Current Status](#current-status)
- [Project Overview](#project-overview)
- [System Architecture](#system-architecture)
- [Phase Status](#phase-status)
- [USB HID Keyboard Implementation](#usb-hid-keyboard-implementation)
- [Build Instructions](#build-instructions)
- [Image Specifications](#image-specifications)
- [Writing to USB](#writing-to-usb)
- [Troubleshooting](#troubleshooting)

---

## Current Status (January 25, 2025)

### ✅ Completed: Phase 6A-6E (Interactive Shell + Filesystem)

| Component | Status | Notes |
|-----------|--------|-------|
| **Direct UEFI Boot** | ✅ Complete | No GRUB, standalone BOOTX64.EFI |
| **PS/2 Keyboard Driver** | ✅ Complete | IRQ1, scancode-to-ASCII, modifiers |
| **Framebuffer Console** | ✅ Complete | RGB565, PSF2 font (8x16), scrolling |
| **Process Management** | ✅ Complete | 256-slot table, round-robin scheduler |
| **Syscall Interface** | ✅ Complete | read, write, spawn, exit, getpid, yield |
| **VFS + Ramdisk** | ✅ Complete | Embedded ELF binaries (init, shell, hello) |
| **Interactive Shell** | ✅ Complete | C shell with Dracula theme, built-in commands |

### ⚠️ In Progress: Phase 7A - USB HID Keyboard Support

| Component | Status | Notes |
|-----------|--------|-------|
| **xHCI Controller** | ✅ Complete | PCI scan, controller init, port reset |
| **TRB Structures** | ✅ Complete | NormalTrb, SetupTrb, StatusTrb, EventTrb, LinkTrb |
| **Transfer Rings** | ✅ Complete | 16-entry rings with cycle bit management |
| **Event Rings** | ✅ Complete | 16-entry event ring with polling |
| **USB Device Enumeration** | ⚠️ In Progress | Port scanning, keyboard detection |
| **Interrupt Transfers** | ✅ Complete | xHCI polling implementation for HID data |
| **HID Report Parsing** | ✅ Complete | 256-entry keycode table, shift handling |

### ⏳ Planned: Phase 7B

1. **Full USB Enumeration** - Complete descriptor parsing
2. **MSI/MSI-X Interrupts** - Event-driven transfers instead of polling
3. **HID Report Descriptor Parsing** - Support for non-standard keyboards

### ⏳ Planned: Phase 7C

- **GUI server process** - Window management
- **GUI client app library** - Widget toolkit

---

## Project Overview

**Rustux OS** is a hobby operating system written in Rust, featuring a native UEFI microkernel with USB HID keyboard support and a Dracula-themed interactive shell.

**Boot Flow:**
```
UEFI Firmware → BOOTX64.EFI → xHCI Init → USB HID Keyboard → Shell (CLI)
```

---

## System Architecture

### Direct UEFI Boot (No Chain Loading)

Traditional Linux distributions use:
```
UEFI → GRUB → Linux kernel → initramfs → init → userspace
```

Rustux OS eliminates middle stages:
```
UEFI → Rustux kernel (with embedded userspace) → shell
```

**Benefits:**
- **Simpler** - No bootloader configuration
- **Faster** - No chain-loading delays
- **Smaller** - No external bootloader binaries
- **Self-contained** - Kernel + userspace in one binary

### Kernel Location

The Rustux kernel is located at:
```
/var/www/rustux.com/prod/loader/kernel-efi/
```

### Userspace/OS Location (Rustica)

Rustica OS userspace is located at:
```
/var/www/rustux.com/prod/rustica/
```

**Architecture Note:**
The shell is a **userspace program** shipped by Rustica OS, not part of the kernel.
The kernel provides only low-level services: syscalls, scheduler, memory management, and drivers.

---

## Phase Status

### Phase 6A: Console & Display (COMPLETE)

- UEFI boot with ExitBootServices
- Framebuffer console with PSF2 font
- Scrolling and text rendering
- Dracula theme support

### Phase 6B: Process Management (COMPLETE)

- Process table with PID tracking
- Round-robin scheduler
- Process spawning via `sys_spawn()`
- Exit handling

### Phase 6C: Interactive Shell (COMPLETE)

#### Shell Process
1. UEFI firmware loads BOOTX64.EFI
2. Kernel initializes (drivers, scheduler, memory)
3. Init process spawns `/bin/shell` as PID 2
4. Shell runs as normal userspace process
5. Shell owns:
   - **stdin** → keyboard (via sys_read)
   - **stdout/stderr** → text console (via sys_write)

#### Built-in Commands

| Command | Type | Description |
|---------|------|-------------|
| `help` | Built-in | Show available commands |
| `clear` | Built-in | Clear screen (ANSI escape codes) |
| `echo` | Built-in | Print arguments to stdout |
| `ps` | Built-in | List running processes |
| `exit` | Built-in | Exit the shell |
| `mem` | Built-in | Show memory information |
| `kbd` | Built-in | Keyboard IRQ debug info |
| `irq` | Built-in | PIC configuration |
| `flush` | Built-in | Flush keyboard buffer |

### Phase 6D: Filesystem & Ramdisk (COMPLETE)

**Status:** ✅ COMPLETE - Embedded ramdisk with VFS abstraction

**Features:**
- Ramdisk embedded in kernel binary
- File operations: open, close, read, write, lseek
- VFS abstraction layer for future filesystems
- sys_open(), sys_close(), sys_read(), sys_write(), sys_lseek()

### Phase 6E: Live Image First Policy

**Primary test target is UEFI hardware, not QEMU.**

QEMU is optional for development but all debugging must be:
- Framebuffer-visible (see VNC display)
- Or persisted to disk
- Or LED / color-code based

This aligns with the **Silent Boot Phase** discipline - no port I/O between UEFI entry and ExitBootServices.

---

## USB HID Keyboard Implementation

### Architecture

The USB HID keyboard driver uses a **polling-based architecture** for Phase 7A:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Keyboard Frontend                           │
│  ┌────────────┐   ┌────────────┐   ┌──────────────────────────┐   │
│  │    USB     │   │   PS/2      │   │     None (Display-only)  │   │
│  │  (Tier-1)  │   │ (Fallback)  │   │  (No keyboard mode)     │   │
│  └─────┬──────┘   └──────┬─────┘   └──────────────────────────┘   │
│        │                 │                                            │
└────────┼─────────────────┼────────────────────────────────────────┘
         │                 │
         ▼                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     USB HID Stack                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │
│  │   pci    │→ │   xhci   │→ │  device  │→ │       hid        │    │
│  │ (scan)   │  │ (init)   │  │ (enum)   │  │ (parse reports) │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘    │
│       │           │           │                │                    │
│       ▼           ▼           ▼                ▼                    │
│   Find XHCI   Reset &     Enumerate     Parse 8-byte              │
│   controller  init ports   HID device   keyboard report           │
└─────────────────────────────────────────────────────────────────────┘
```

### File Structure

```
src/usb/
├── mod.rs          # USB module declarations, error types
├── pci.rs          # PCI configuration space scanning for XHCI
├── xhci.rs         # xHCI controller, transfer rings, event rings
├── trb.rs          # TRB structures (Normal, Setup, Status, Link, Event)
├── device.rs       # USB device enumeration, HID keyboard detection
└── hid.rs          # HID Boot Protocol keyboard parsing

src/keyboard/
├── mod.rs          # Backend selection (USB → PS/2 → Display-only)
├── ps2.rs          # Legacy PS/2 driver (IRQ1)
└── usb.rs          # USB keyboard adapter layer
```

### xHCI Interrupt Transfer Flow

```rust
// 1. Check if controller and keyboard are available
let controller = xhci::controller()?;
let keyboard = device::get_hid_keyboard()?;

// 2. Create Normal TRB for interrupt IN transfer
let trb = NormalTrb {
    data_ptr: &HID_REPORT_BUFFER as *const u8 as u64,
    status: 8,  // TRB length (8 bytes)
    control: (1 << 10) | (1 << 16) | (1 << 5),  // Type=Normal, IOC, ISP
};

// 3. Enqueue TRB to transfer ring
controller.transfer_ring.enqueue(&trb)?;

// 4. Ring doorbell for endpoint
controller.ring_doorbell(slot_id, dci, 0);

// 5. Poll for event completion
while timeout > 0 {
    if let Some(event) = controller.poll_events() {
        return event.status & 0x00FFFFFF;  // Transfer length
    }
}
```

### HID Keycode Translation

256-entry translation table maps USB HID keycodes to ASCII:
- 0x04-0x1D: a-z (lowercase)
- 0x1E-0x27: 1-0, Enter, Esc, Backspace, Tab
- 0x2D-0x38: Symbols (-, =, [, ], \, ;, ', `, ,, ., /)
- Shift modifier handling for uppercase and shifted symbols

### Keyboard Backend Selection

```rust
pub fn init() {
    unsafe {
        // First try USB (Tier-1 input for modern systems)
        if let Ok(()) = usb::init() {
            KEYBOARD_BACKEND = KeyboardBackend::Usb;
            crate::framebuffer::write_str("[USB KBD] ");
            return;
        }

        // Fall back to PS/2 (legacy hardware)
        if ps2::controller_present() {
            KEYBOARD_BACKEND = KeyboardBackend::Ps2;
            crate::framebuffer::write_str("[PS/2] ");
        } else {
            KEYBOARD_BACKEND = KeyboardBackend::None;
            crate::framebuffer::write_str("[NO KEYBOARD] ");
        }
    }
}
```

### Keyboard Support Matrix

| Keyboard Type | Connection | Method | Status |
|---------------|------------|--------|--------|
| **PS/2** | Physical port 0x60/0x64 | IRQ1 (interrupt) | ✅ Works (QEMU + real HW) |
| **PS/2** | UEFI emulation | Polling | ✅ Works (fallback) |
| **USB HID** | xHCI controller | Polling | ⚠️ In Progress (detection fails) |
| **USB HID** | xHCI controller | MSI/MSI-X | ⏳ Phase 7B |

**Honest Detection:**
- `[USB KBD]` - xHCI controller found, USB keyboard detected
- `[PS/2]` - PS/2 controller present, USB not found
- `[NO KEYBOARD]` - Neither USB nor PS/2 detected (system continues in display-only mode)

**Known Issues (January 25, 2025):**
1. **USB HID Detection Failing** - The xHCI initialization returns error, causing fallback to PS/2
   - On systems without PS/2 (modern laptops), this shows `[NO KEYBOARD]`
   - Root cause: xHCI PCI enumeration or controller initialization needs debugging
   - System now gracefully continues without keyboard instead of halting (FIXED)

2. **Display-Only CLI Mode** - When `[NO KEYBOARD]` is detected, the shell runs in display-only mode
   - Commands: `help`, `clear`, `mem`, `kbd`, `ps`, `exit` work for viewing system state
   - No interactive input possible without keyboard hardware

---

## Build Instructions

### Prerequisites

#### Build Tools

```bash
# Rust toolchain (UEFI target)
rustup target add x86_64-unknown-uefi

# GCC for cross-compiling userspace C programs
apt install gcc-x86-64-linux-gnu

# Image creation tools
apt install parted dosfstools coreutils
```

#### System Requirements

- **Architecture**: x86_64 (AMD64)
- **OS**: Linux (recommended) or macOS
- **Disk Space**: 500 MB for build artifacts
- **RAM**: 4 GB recommended for compilation

### Project Layout

```
/var/www/rustux.com/prod/
├── kernel-efi/             # UEFI transition kernel + live image tooling
│   ├── src/                # Source files (acpi, console, framebuffer, etc.)
│   ├── usb/                # USB HID stack (NEW - Phase 7A)
│   │   ├── pci.rs           # PCI scanning for XHCI
│   │   ├── xhci.rs          # xHCI controller, rings
│   │   ├── trb.rs           # TRB structures
│   │   ├── device.rs        # Device enumeration
│   │   └── hid.rs           # HID report parsing
│   ├── keyboard/           # Keyboard drivers (mod.rs, ps2.rs, usb.rs)
│   ├── build-live-image.sh # Live USB build script
│   └── target/
│       └── x86_64-unknown-uefi/
│           └── release/
│               └── rustux-kernel-efi.efi
├── rustux/                 # Canonical microkernel (modular architecture)
│   ├── src/                # Kernel source code
│   └── build.rs            # Embed ramdisk with userspace binaries
└── rustica/                # Userspace OS distribution
    ├── docs/               # Documentation
    └── test-userspace/     # C programs for shell, init, tests
```

**Note:** Currently using `loader/kernel-efi/` as the validated kernel.

### Build Steps

#### Step 1: Build the Kernel

```bash
cd /var/www/rustux.com/prod/loader/kernel-efi

# Build UEFI kernel with release optimizations
cargo build --release --target x86_64-unknown-uefi
```

**Output:** `target/x86_64-unknown-uefi/release/rustux-kernel-efi.efi`

#### Step 2: Build Live USB Image

```bash
cd /var/www/rustux.com/prod/loader

# Run the build script
chmod +x build-live-image.sh
./build-live-image.sh

# Specify a custom version
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
| **Size** | 128 MB |
| **Boot Path** | `/EFI/BOOT/BOOTX64.EFI` |
| **Boot Method** | UEFI direct boot |

### Directory Structure (Inside Image)

```
[EFI System Partition]
└── EFI/
    └── BOOT/
        └── BOOTX64.EFI    (rustux-kernel-efi.efi - the kernel)
```

### What's Included

### Kernel (kernel-efi/)
- UEFI boot stub
- Process & thread management
- Syscall interface
- Memory management
- Device drivers:
  - PS/2 keyboard driver
  - USB HID keyboard driver (Phase 7A)
  - Framebuffer console driver

### Userspace (rustica/)
- **Shell** (`/bin/shell`) - Interactive C shell
- **Test programs** - hello, counter

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
[*** RUSTUX SHELL v0.1 ***]
[rustux> _]  (Interactive shell prompt)
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
| **Input** | USB HID (xHCI) or PS/2 | Both supported |

---

## Troubleshooting

### Kernel Build Fails

**Error:** `target not found: x86_64-unknown-uefi`

**Fix:**
```bash
rustup target add x86_64-unknown-uefi
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

### Keyboard Not Detected

**Symptoms:** System shows `[NO KEYBOARD]` and enters display-only mode

**Diagnosis:**
1. Check if system has PS/2 port (most modern laptops don't)
2. Check if USB keyboard is connected
3. Run `kbd` command to see IRQ status

**Current Status:** USB HID detection is in Phase 7A (in progress). PS/2 works on systems with PS/2 ports.

### Shell Not Appearing

**Check:**
1. Monitor is connected to primary GPU output
2. System has sufficient RAM (512 MB minimum)
3. Check POST codes via port 0x80 (QEMU: `-debugcon port:53710`)

---

## Development Workflow

For rapid development iteration:

```bash
#!/bin/bash
# dev-build.sh - Quick rebuild and test script

# Rebuild kernel
cd /var/www/rustux.com/prod/loader/kernel-efi
cargo build --release --target x86_64-unknown-uefi

# Rebuild image
cd /var/www/rustux.com/prod/loader
./build-live-image.sh

# Optional: Test in QEMU (if available)
# qemu-system-x86_64 \
#   -drive format=raw,file=fat:rw:esp,if=ide \
#   -bios /usr/share/OVMF/OVMF.fd \
#   -device qemu-xhci,id=xhci \
#   -device usb-kbd,bus=xhci.0
```

---

## Dracula Theme (MANDATORY INVARIANT)

> **Theme Invariant**
>
> The Dracula color palette is the **default system theme** and must survive:
> - Kernel rebuilds
> - CLI refactors
> - Framebuffer rewrites
> - GUI introduction later

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

## Documentation

- **README.md** - Project overview and USB HID implementation details
- **PLAN.md** - Development roadmap

---

## Git Repository

- **https://github.com/gitrustux/rustux**

---

## License

Copyright (c) 2025 The Rustux Authors

Licensed under the MIT License. See LICENSE file for details.

---

*Last Updated: January 25, 2025*
