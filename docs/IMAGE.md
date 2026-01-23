# Rustux OS - Bootable Image

This directory contains the bootable Rustux OS disk image that can be written to a USB drive for live boot testing.

---

## Phase 6 Complete: Interactive Shell (2025-01-22)

**STATUS**: Phase 6 (Input/Display/Shell) is COMPLETE. Rustux now boots directly into an interactive shell without requiring QEMU or serial output.

### Boot Flow (As of Phase 6)

```
UEFI firmware → BOOTX64.EFI → kernel.efi → init (PID 1) → shell (PID 2)
```

**What's Working:**
- ✅ UEFI boot with ExitBootServices
- ✅ PS/2 keyboard driver
- ✅ Framebuffer text console
- ✅ Process management & scheduler
- ✅ Syscall interface (read, write, spawn, exit)
- ✅ Init process spawns shell
- ✅ Interactive shell with Dracula theme
- ✅ Built-in commands (help, clear, echo, ps, exit)
- ✅ External program execution

### Kernel Location

The Rustux kernel is located at:
```
/var/www/rustux.com/prod/rustux/
```

### Userspace/OS Location (Rustica)

Rustica OS userspace is located at:
```
/var/www/rustux.com/prod/rustica/
```

**Architecture Note:**
The shell is a **userspace program** shipped by Rustica OS, not part of the kernel.
The kernel provides only low-level services: syscalls, scheduler, memory management, and drivers.

### Project Layout

**Long-term layout (recommended):**
```
/prod
 ├── rustux/          # kernel only (OS-agnostic)
 ├── rustica/         # userspace OS (distribution)
 │    ├── bin/         # user programs
 │    │    └── shell
 │    ├── etc/
 │    │    └── theme.toml   (Dracula theme config)
 │    ├── usr/         # user data
 │    └── docs/
 └── tools/           # build tools
```

**Why this matters:**
- Kernel must stay OS-agnostic
- CLI is userspace, not kernel
- GUI later will sit next to CLI, not replace it

---

## Shell & Console Architecture

### Phase 6C: Interactive Shell

**Goal:** Boot directly into a usable shell with stdin/stdout, process execution, and theming.

#### 6C.1 Shell Process

**Process Flow:**
1. UEFI firmware loads BOOTX64.EFI
2. Kernel initializes (drivers, scheduler, memory)
3. Init process spawns `/bin/shell` as PID 2
4. Shell runs as normal userspace process
5. Shell owns:
   - **stdin** → keyboard (via sys_read)
   - **stdout/stderr** → text console (via sys_write)

**Shell Loop:**
```
read line
parse command
execute builtin OR spawn process
wait / continue
```

#### 6C.2 Built-in Commands

| Command | Type | Description |
|---------|------|-------------|
| `help` | Built-in | Show available commands |
| `clear` | Built-in | Clear screen (ANSI escape codes) |
| `echo` | Built-in | Print arguments to stdout |
| `ps` | Built-in | List running processes |
| `exit` | Built-in | Exit the shell |
| `hello` | External | Hello world program |
| `counter` | External | Counter test program |

**Clarification:**
- **Built-in commands** run inside shell process
- **External commands** use `sys_spawn()` + execution + wait

#### 6C.3 Dracula Theme (MANDATORY INVARIANT)

> **Theme Invariant**
>
> The Dracula color palette is the **default system theme** and must survive:
> - Kernel rebuilds
> - CLI refactors
> - Framebuffer rewrites
> - GUI introduction later

**Canonical Dracula Colors (Defined Once):**
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

**Theme Architecture:**
- Theme lives in **console layer**, not shell logic
- Shell only requests colors via ANSI escape codes
- Console enforces the Dracula palette
- This separation ensures theme consistency across all programs

### Phase 6D: Filesystem & Ramdisk

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

## Build Instructions (Updated)

```bash
# Navigate to the kernel location
cd /var/www/rustux.com/prod/rustux

# Build kernel (AMD64 UEFI)
cargo build --release --bin rustux --features uefi_kernel --target x86_64-unknown-uefi

# Build userspace programs (C programs in test-userspace/)
cd test-userspace
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector shell.c -o shell.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector init.c -o init.elf
```

---

## Image Build Requirements (Updated)

When creating disk images, maintain these invariants:

1. **GPT disk image** (not raw FAT)
2. **EFI System Partition:**
   - FAT32
   - ≥ 100MB (prefer 200–512MB)
   - Required path: `/EFI/BOOT/BOOTX64.EFI` (the kernel)
   - Correct PE/COFF format
   - Correct target architecture (amd64 only for now)

3. **Boot flow (Phase 6):**
   ```
   UEFI firmware → BOOTX64.EFI → kernel → init → shell
   ```

4. **Ramdisk embedding:**
   - Shell binary embedded as `/bin/shell`
   - Init binary embedded as `/bin/init`
   - Test programs available as `/bin/hello`, `/bin/counter`

5. **Sparse image creation:**
   ```bash
   dd if=/dev/zero of=image.img bs=1M count=0 seek=512M
   ```

---

## What's Included

The system now contains:

### Kernel (rustux/)
- UEFI boot stub
- Process & thread management
- Syscall interface
- Memory management
- Device drivers:
  - PS/2 keyboard driver
  - Framebuffer console driver
  - UART (serial) driver

### Userspace (rustica/)
- **Shell** (`/bin/shell`) - Interactive C shell
- **Init** (`/bin/init`) - First userspace process
- **Test programs** - hello, counter

### Built-in Shell Commands
- `help` - List commands
- `clear` - Clear screen
- `echo` - Print text
- `ps` - List processes
- `exit` - Exit shell

---

## Quick Start (Live USB Testing)

### Writing to USB (Linux)

```bash
# Identify your USB device (e.g., /dev/sdb, /dev/sdc)
lsblk

# Write the image to USB (replace /dev/sdX with your device)
sudo dd if=disk.img of=/dev/sdX bs=4M status=progress conv=fsync

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
sudo dd if=disk.img of=/dev/rdisk2 bs=4m status=progress

# Eject
diskutil eject /dev/disk2
```

### Writing to USB (Windows)

Use a tool like [Rufus](https://rufus.ie/) or [BalenaEtcher](https://www.balena.io/etcher/):

1. Download and install Rufus or Etcher
2. Select the `disk.img` file
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

---

## System Requirements

### AMD64 (Current - Fully Supported)
- **Architecture**: x86_64 (AMD64)
- **RAM**: 512 MB minimum, 1 GB recommended
- **Storage**: 4 GB minimum for installation
- **Boot**: UEFI required
- **Input**: PS/2 keyboard (USB HID support coming in Phase 7)

### ARM64 (Future)
- **Architecture**: AArch64 (ARM64)
- **RAM**: 1 GB minimum, 2 GB recommended
- **Boot**: UEFI required
- **Status**: Code structure ready, needs hardware testing

### RISC-V (Future)
- **Architecture**: riscv64gc
- **RAM**: 1 GB minimum, 2 GB recommended
- **Boot**: UEFI required
- **Status**: Code structure ready, needs hardware testing

---

## Support

For more information and updates:

- **Documentation**: See `PLAN.md` for the development roadmap
- **Kernel Repository**: https://github.com/gitrustux/rustux
- **Issues**: https://github.com/gitrustux/rustux/issues

---

## License

Copyright (c) 2025 The Rustux Authors

Licensed under the MIT License. See LICENSE file for details.
