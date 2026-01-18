# Rustica OS - Bootable Image

This directory contains the bootable Rustica OS disk image that can be written to a USB drive or used for installation.

---

## Kernel Update: Phase 2C Complete (2025-01-18)

**STATUS**: The Rustux kernel has been successfully refactored and migrated to a new Zicorn-inspired architecture.

### New Kernel Location

The refactored kernel is now located at:
```
/var/www/rustux.com/prod/rustux/
```

### What Changed

The kernel has been reorganized into a modular architecture with the following new structure:

| Module | Description | Lines |
|--------|-------------|-------|
| `src/arch/amd64/` | AMD64 architecture (fully implemented) | ~2,500 |
| `src/arch/arm64/` | ARM64 architecture (placeholder) | ~1,000 |
| `src/arch/riscv64/` | RISC-V architecture (placeholder) | ~1,500 |
| `src/object/` | Zircon-style kernel objects | ~2,500 |
| `src/process/` | Process & thread management | ~2,000 |
| `src/syscall/` | System call interface | ~1,500 |
| `src/sync/` | Synchronization primitives | ~1,000 |
| `src/mm/` | Memory management | ~1,500 |
| `src/drivers/` | Device drivers (UART) | ~500 |

**Total migrated**: ~13,500 lines across 8 phases (2C.1-2C.8)

### Build Instructions (Updated)

```bash
# Navigate to the new kernel location
cd /var/www/rustux.com/prod/rustux

# Build for AMD64 (UEFI)
cargo build --release --bin rustux --features uefi_kernel --target x86_64-unknown-uefi

# Build for ARM64 (placeholder, not yet functional)
cargo build --release --bin rustux --features uefi_kernel --target aarch64-unknown-uefi

# Run QEMU test
./test-qemu.sh
```

### Kernel Objects & Capabilities

The refactored kernel introduces Zicorn-style kernel objects:

- **Handle**: Capability-based access to kernel resources
- **VMO**: Virtual Memory Objects for memory regions
- **Channel**: IPC channels for message passing
- **Event**: Synchronization events
- **Timer**: High-resolution timers
- **Job**: Job objects for resource containers

### CLI Integration (In Progress)

A new CLI tool `rustux-kernel` is planned at:
```
/var/www/rustux.com/prod/apps/cli/rustux-kernel/
```

This tool will provide:
- Kernel build commands
- QEMU testing automation
- Syscall testing suite
- Image creation utilities

**Note**: GUI integration is on hold until CLI is complete.

### Documentation

For complete integration details, see:
- **Integration Plan**: `/var/www/rustux.com/prod/rustica/docs/PLAN.md`
- **Kernel Repository**: https://github.com/gitrustux/rustux

### Old Kernel Location (Deprecated)

The previous kernel location at `/var/www/rustux.com/prod/kernel/` is now deprecated and will be removed once all systems are updated to use the new location.

---

## ExitBootServices Fix Summary (2025-01)

**ISSUE**: Kernel was hanging at ExitBootServices with "memory map changed" error.

**ROOT CAUSE**: UEFI console output (`uefi::system::with_stdout`) was happening BETWEEN the final `GetMemoryMap()` call and `ExitBootServices()`. UEFI console can trigger internal allocations, which invalidates the memory map key.

**FIX**: Implemented a strict "frozen zone" - absolutely NO allocations, prints, or protocol calls between GetMemoryMap and ExitBootServices. Only raw CPU instructions and the ExitBootServices call itself are allowed in this zone.

**VERIFICATION**: After implementing the frozen zone, ExitBootServices succeeds. The UEFI console stops working after ExitBootServices (expected), but the kernel is alive and in runtime mode.

**KNOWN ISSUES**:
- Serial port I/O (COM1) causes hangs in QEMU/OVMF environment - avoid for now
- After ExitBootServices, need to use custom memory allocator (not UEFI's)
- Native console driver needs to be implemented for post-exit output

## IMPORTANT: ExitBootServices Debugging Protocol

If the kernel hangs at "Step 4: Exiting UEFI boot services", follow this protocol EXACTLY.

### Why ExitBootServices Hangs

ExitBootServices() can hang or "never return" only if one of these is true:

1. The memory map changed between GetMemoryMap() and ExitBootServices()
2. Boot services memory is accessed after exit
3. Interrupts fire with no handlers
4. Stack or heap still lives in firmware memory
5. The kernel prints/logs after exit (dead console)
6. Page tables reference invalid regions
7. A fault occurs but no handler exists → silent halt

### Debugging Protocol - Step by Step

When debugging ExitBootServices issues, implement INSTRUMENTATION ONLY. Do NOT optimize, refactor, or reorganize code.

#### 1. Add a "Hard Stop" Tracing Ladder (CRITICAL)

Implement numbered progress markers that survive without a console.

**Strategy: Use multiple independent channels**

- Serial port output (COM1 / UART base 0x3F8)
- Volatile memory counter
- CPU halt loop with unique signatures

**Example trace points:**

```
Before ExitBootServices:
  [BOOT-TRACE] 1 - About to disable interrupts
  [BOOT-TRACE] 2 - Interrupts disabled
  [BOOT-TRACE] 3 - About to get memory map

After ExitBootServices:
  [BOOT-TRACE] 4 - ExitBootServices returned
  [BOOT-TRACE] 5 - Starting runtime init
```

#### 2. Add Serial Output BEFORE and AFTER Exit

This is the single most important debugging step.

Add DIRECT serial writes (no UEFI console, no wrappers) at each stage:

- Before GetMemoryMap
- Before ExitBootServices
- Immediately after ExitBootServices
- First instruction of runtime loop

**Serial port initialization (x86_64 COM1):**
```c
// Base address: 0x3F8
// Initialize for 115200 baud, 8N1
```

**If you see serial output after exit:** The kernel did NOT hang — it lost visibility.

#### 3. Freeze the Memory Map (UEFI Classic Failure Mode)

UEFI requires:
1. GetMemoryMap()
2. ExitBootServices(map_key)
3. NOTHING may change memory between these calls

**Critical Rules:**
- Allocate memory map buffer from kernel-owned memory
- Do NOT allocate anything between GetMemoryMap and ExitBootServices
- Call GetMemoryMap() twice and compare map size and key
- Log if it changed

#### 4. Disable Interrupts BEFORE Exit

This is non-optional.

**Required sequence:**
1. Disable interrupts immediately before ExitBootServices (`cli` on x86_64)
2. Do NOT re-enable until exception handlers are installed
3. A stray interrupt = silent halt

#### 5. Verify Stack and Heap Location

Ensure:
- Stack pointer is NOT in EfiBootServicesData
- Heap is NOT firmware-backed
- Log stack address, heap base, and memory type

#### 6. Add Post-Exit Infinite Loop (Test-Only)

This is how you prove the CPU is alive.

**After ExitBootServices, temporarily:**
1. Skip ALL runtime init
2. Enter an infinite loop with:
   - CPU pause/hlt
   - Serial heartbeat every N cycles

**If this runs:** ExitBootServices succeeded.

#### 7. Install Minimal Exception Handlers Before Exit

Even a stub handler is better than none.

**Install minimal handlers before ExitBootServices:**
- Page fault
- General protection fault
- On fault: write a serial marker and halt

This prevents "silent death".

#### 8. DO NOT Print to Console After Exit

After ExitBootServices:
- Do NOT call any console output
- Do NOT log using UEFI helpers
- Assume output is unavailable

Many kernels "hang" simply because they print.

### Minimal "ExitBootServices Probe" Sequence

Implement this sequence to diagnose the hang:

```
1. disable_interrupts()
2. trace("A")  // serial - about to get memory map
3. get_memory_map()
4. trace("B")  // serial - memory map acquired
5. exit_boot_services()
6. trace("C")  // serial - exit returned
7. while (true) { cpu_pause(); serial_heartbeat(); }
```

**Interpret results:**
- See A, B but not C → exit failed (memory map or interrupts issue)
- See A, B, C → runtime is alive (issue is in later init)
- Nothing after exit → output died, not kernel

### Success Outcomes

**Outcome 1 (Best):**
- Serial shows post-exit trace
- Infinite loop runs
- → ExitBootServices is working

**Outcome 2:**
- Serial stops before exit
- → Memory map or interrupts are wrong

**Outcome 3:**
- Immediate reboot or freeze
- → Exception with no handler

### Image Build Requirements (DO NOT CHANGE)

When creating disk images, maintain these invariants:

1. **GPT disk image** (not raw FAT)
2. **EFI System Partition:**
   - FAT32
   - ≥ 100MB (prefer 200–512MB)
   - Required path: `/EFI/BOOT/BOOTX64.EFI`
   - Correct PE/COFF format
   - Correct target architecture (amd64 only for now)
3. **Proper component separation:**
   - Bootloader: EFI-only, minimal
   - Kernel: loaded by bootloader at `/EFI/Rustux/kernel.efi`
   - EFI CLI: debug-only scaffold
   - Runtime: NOT active until explicitly tested

4. **Boot flow must be exactly:**
   ```
   UEFI firmware → BOOTX64.EFI → load Rustux kernel → kernel entry → EFI CLI (debug)
   ```

5. **NO automatic ExitBootServices** unless explicitly testing it.

6. **Sparse image creation:**
   ```bash
   dd if=/dev/zero of=image.img bs=1M count=0 seek=512M
   ```

### Build Verification

After building images, verify:

```bash
# File size (should be 512M)
ls -lh rustica-live-amd64-0.1.0.img

# Disk usage (sparse file should be small)
du -h rustica-live-amd64-0.1.0.img

# Partition table (should be GPT)
fdisk -l rustica-live-amd64-0.1.0.img

# EFI directory structure (when mounted)
find /mnt -name "BOOTX64.EFI"
find /mnt -name "kernel.efi"
```

---

## Download

### Latest Images

- **rustica-live-amd64-0.1.0.img** - AMD64 bootable image (512 MB sparse)
- **rustica-live-amd64-0.1.0.img.sha256** - SHA256 checksum for verification

A symlink `rustica-live-amd64.img` is provided for convenience, always pointing to the latest AMD64 version.

> **Note:** Currently only AMD64 images are available. ARM64 and RISC-V support is coming soon.

## Quick Start

### Writing to USB (Linux)

```bash
# Identify your USB device (e.g., /dev/sdb, /dev/sdc)
lsblk

# Write the image to USB (replace /dev/sdX with your device)
sudo dd if=rustica-live-amd64-0.1.0.img of=/dev/sdX bs=4M status=progress conv=fsync

# Sync and eject
sudo sync
sudo eject /dev/sdX
```

### Writing to USB (macOS)

```bash
# Identify your USB disk (e.g., /dev/disk2)
diskutil list

# Unmount the disk
diskutil unmountDisk /dev/disk2

# Write the image
sudo dd if=rustica-live-amd64-0.1.0.img of=/dev/rdisk2 bs=4m status=progress

# Eject
diskutil eject /dev/disk2
```

### Writing to USB (Windows)

Use a tool like [Rufus](https://rufus.ie/) or [BalenaEtcher](https://www.balena.io/etcher/):

1. Download and install Rufus or Etcher
2. Select the `rustica-live-amd64-0.1.0.img` file
3. Select your USB drive
4. Click "Flash" or "Start"
5. Wait for completion

## Verification

Verify the downloaded image integrity:

```bash
sha256sum -c rustica-live-amd64-0.1.0.img.sha256
```

Expected output:
```
rustica-live-amd64-0.1.0.img: OK
```

## Booting from USB

1. Insert the USB drive
2. Restart your computer
3. Enter the boot menu (usually F12, F2, Del, or Esc key)
4. Select the USB drive as boot device
5. Follow the on-screen prompts

## What's Included

The live image contains:

- **rustux-install** - The Rustica OS installer
- **CLI Tools**:
  - `login` - User login management
  - `ping` - Network connectivity testing
  - `ip` - Network configuration
  - `fwctl` - Firewall control
  - `dnslookup` / `rustux-dnslookup` - DNS queries
  - `editor` / `vi` / `nano` - Text editor
  - `ssh` / `rustux-ssh` - SSH client
  - `logview` - Log viewer
- **rpg** - Rustica Package Manager

## Installation Options

When you boot the live image, you'll see a menu:

```
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║              Rustica OS v0.1.0 - Live                    ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝

What would you like to do?

  [1] Install Rustica OS to a device
      - Install Rustica OS to a hard drive or SSD
      - All data on the target device will be erased

  [2] Try out Rustica OS
      - Boots entirely into RAM; changes are lost on reboot
      - Great for testing hardware compatibility
      - Get a feel for the distro without installing

  [3] Portable Rustica OS
      - A full, persistent Linux environment you carry with you
      - Saves your files, settings, and installed software
      - All changes persist on the USB drive across reboots
```

### Mode Descriptions

**Install to Device**: Traditional installation that erases the target drive and installs Rustica OS permanently. Choose this for a full system installation.

**Try Out (Live Mode)**: Loads the entire OS into RAM. Perfect for:
- Testing if your hardware is compatible
- Trying out Rustica OS before installing
- Quick demonstrations

**Portable (Persistent Live)**: Runs from the USB drive but saves all changes. Great for:
- Carrying your personalized OS with you
- Using any computer as your own
- Having a backup system that keeps your settings

## Manual Installation

If you prefer to install manually without the interactive installer:

```bash
# Mount the image
sudo mkdir /mnt/rustica
sudo mount -o loop rustica-live-amd64-0.1.0.img /mnt/rustica

# Run the installer directly
sudo /mnt/rustica/bin/rustux-install --device /dev/sdX --yes
```

## System Requirements

### AMD64 (Current)
- **Architecture**: x86_64 (AMD64)
- **RAM**: 512 MB minimum, 1 GB recommended
- **Storage**: 4 GB minimum for installation
- **Boot**: UEFI (recommended) or Legacy BIOS support

### ARM64 (Coming Soon)
- **Architecture**: AArch64 (ARM64)
- **RAM**: 1 GB minimum, 2 GB recommended
- **Storage**: 4 GB minimum for installation
- **Boot**: UEFI required

### RISC-V (Coming Soon)
- **Architecture**: riscv64gc
- **RAM**: 1 GB minimum, 2 GB recommended
- **Storage**: 4 GB minimum for installation
- **Boot**: UEFI required

## Troubleshooting

### USB won't boot

- Ensure your USB drive is at least 4 GB
- Try reformatting the USB before writing
- Verify the SHA256 checksum
- Try a different USB port or drive
- Check that UEFI boot is enabled in BIOS

### Installation fails

- Ensure you're running as root: `sudo rustux-install`
- Check that target device is correct
- Verify you have enough disk space
- Try with `--yes` flag for automated mode

### Can't detect USB device

- Run: `lsblk` to list all block devices
- Check USB is properly connected
- Try a different USB port

## Multi-Architecture Support

The Rustux kernel has been refactored to support three architectures. Here's the current status:

### Architecture Support Matrix (2025-01-18)

| Architecture | Code | Bootloader | Kernel | Status |
|--------------|------|------------|--------|--------|
| **AMD64** | `x86_64` | ✅ Complete | ✅ Complete | ✅ Fully Functional |
| **ARM64** | `aarch64` | ⏳ Planned | 🔨 Placeholder | ⏳ Code Ready, Testing Needed |
| **RISC-V** | `riscv64gc` | ⏳ Planned | 🔨 Placeholder | ⏳ Code Ready, Testing Needed |

### AMD64 (Current - Fully Supported)

- **Location**: `/var/www/rustux.com/prod/rustux/src/arch/amd64/`
- **Features**:
  - Complete interrupt handling (IDT, PIC, APIC)
  - Memory management with 4-level page tables
  - Syscall interface (AMD64 ABI)
  - Bootstrap code (16-bit → 64-bit transition)
  - Cache management and CPU features
  - Full UEFI boot support

### ARM64 (Placeholder - Ready for Implementation)

- **Location**: `/var/www/rustux.com/prod/rustux/src/arch/arm64/`
- **Implemented**:
  - Architecture definitions (`arch.rs`)
  - GIC interrupt controller stub (`interrupt/gic.rs`)
  - MMU page table structure (`mm/mod.rs`)
- **Status**: Code structure ready, needs native ARM64 hardware/testing
- **Priority**: LOW (CLI development takes precedence)

### RISC-V (Placeholder - Ready for Implementation)

- **Location**: `/var/www/rustux.com/prod/rustux/src/arch/riscv64/`
- **Implemented**:
  - Hart (CPU) information (`arch.rs`)
  - PLIC interrupt controller stub (`interrupt/plic.rs`)
  - SBI (Supervisor Binary Interface) support
  - CLINT (Core-Local Interrupt Controller)
  - Sv39/Sv48 MMU page tables (`mm/mod.rs`)
- **Status**: Code structure ready, needs native RISC-V hardware/testing
- **Priority**: LOW (CLI development takes precedence)

### Boot Image Availability

Currently **only AMD64 bootable images** are available. ARM64 and RISC-V images require:

1. Native UEFI bootloader implementation
2. Hardware testing platforms
3. Toolchain cross-compilation verification

For the complete architecture roadmap and implementation details, see:
- **Integration Plan**: `/var/www/rustux.com/prod/rustica/docs/PLAN.md`
- **Kernel Repository**: https://github.com/gitrustux/rustux

## Support

For more information and updates:

- **Website**: https://rustux.com
- **Documentation**: https://docs.rustux.com
- **Issues**: https://github.com/gitrustux/rustica/issues

## License

Copyright (c) 2025 The Rustux Authors

Licensed under the MIT License. See LICENSE file for details.
