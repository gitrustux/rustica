# Rustux OS - Phase 6: Interactive Shell (January 2025)

**Status:** 🟡 Phase 6A-6C COMPLETE | Keyboard IRQ debugging in progress

---

## Project Overview

**Rustux OS** is a hobby operating system written in Rust, featuring a native UEFI microkernel with an interactive shell and Dracula-themed interface.

**Boot Flow:**
```
UEFI Firmware → BOOTX64.EFI → Transition Kernel → Init (PID 1) → Shell (PID 2)
```

---

## Current Status (January 23, 2025)

### ✅ Completed: Phase 6A-6C (Interactive Shell)

| Component | Status | Notes |
|-----------|--------|-------|
| **Direct UEFI Boot** | ✅ Complete | No GRUB, standalone BOOTX64.EFI |
| **PS/2 Keyboard Driver** | ✅ Complete | IRQ1, scancode-to-ASCII, modifiers |
| **Framebuffer Console** | ✅ Complete | RGB565, PSF2 font (8x16), scrolling |
| **Process Management** | ✅ Complete | 256-slot table, round-robin scheduler |
| **Syscall Interface** | ✅ Complete | read, write, spawn, exit, getpid, yield |
| **VFS + Ramdisk** | ✅ Complete | Embedded ELF binaries (init, shell, hello, counter) |
| **Interactive Shell** | ✅ Complete | C shell with Dracula theme, built-in commands |
| **Live USB Image** | ✅ Complete | 128MB GPT disk image with FAT32 ESP |

### 🟡 In Progress: Phase 6D (Keyboard IRQ Delivery)

**Issue:** PS/2 keyboard IRQ is configured correctly (level-triggered, active-low), but interrupt is not reaching the CPU.

**Latest Fixes Applied:**
1. ✅ IOAPIC redirection: Level-triggered + active-low (bit 13 | bit 15)
2. ✅ LAPIC MSR enablement: IA32_APIC_BASE (MSR 0x1B bit 11)
3. ✅ Corrected LAPIC register offsets: SVR 0xF0, TPR 0x080
4. ✅ Added `int 33` diagnostic test to verify IDT entry
5. ✅ EOI sent to Local APIC (0xFEE00B0) not PIC port

**Current Image:** `/var/www/rustux.com/html/rustica/rustica-live-amd64-0.1.0.img`
- SHA256: `8fa30b0e97979ded6e238b067c0a240ee75d57a5f08d2145eb97e95c632ed832`

---

## Directory Structure

```
/var/www/rustux.com/prod/
├── loader/              # UEFI transition kernel + live image tooling
│   ├── kernel-efi/         # Monolithic UEFI kernel (Phase 6 validated)
│   ├── uefi-loader/        # UEFI bootloader (loads kernel.efi)
│   ├── userspace/          # Rust userspace test programs
│   ├── build-live-image.sh # Live USB build script
│   └── target/            # Built kernel.efi binary
├── rustux/               # Canonical microkernel (modular architecture)
│   └── src/               # Microkernel source code
├── rustica/              # Userspace OS distribution
│   ├── docs/              # Documentation (BUILD.md, IMAGE.md, PLAN.md)
│   └── test-userspace/    # C programs (shell, init, tests)
└── apps/                # CLI tools and GUI applications
```

**Note on Kernel Architecture:**
- `loader/kernel-efi/` - **Transition kernel** (monolithic UEFI application)
  - Used to validate Phase 6 features (live boot, PS/2 keyboard, framebuffer, shell)
  - Single binary for easier live USB testing
  - Will be retired after validated subsystems migrate to microkernel

- `rustux/` - **Canonical microkernel** (modular architecture)
  - The "real" Rustux kernel with proper separation of concerns
  - Phase 6D will migrate validated subsystems from transition kernel
  - Target for all future development

---

## Building

### Prerequisites

```bash
# Rust toolchain (UEFI target)
rustup target add x86_64-unknown-uefi

# GCC for cross-compiling userspace C programs
apt install gcc-x86-64-linux-gnu

# Image creation tools
apt install parted dosfstools coreutils
```

### Build Steps

#### 1. Build Transition Kernel

```bash
cd /var/www/rustux.com/prod/loader/kernel-efi

# Build UEFI kernel with release optimizations
cargo build --release --target x86_64-unknown-uefi
```

**Output:** `target/x86_64-unknown-uefi/release/kernel.efi`

#### 2. Build Userspace Programs

```bash
cd /var/www/rustux.com/prod/rustica/test-userspace

# Build shell (C program, static linking, no stdlib)
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector shell.c -o shell.elf

# Build init (first userspace process)
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector init.c -o init.elf

# Build test programs
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector hello.c -o hello.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector counter.c -o counter.elf
```

**Note:** The kernel's `build.rs` automatically embeds these binaries into the ramdisk during compilation.

#### 3. Build Live USB Image

```bash
cd /var/www/rustux.com/prod

# Use wrapper script (delegates to loader/build-live-image.sh)
chmod +x build-live-image.sh
./build-live-image.sh
```

**Output:** `/var/www/rustux.com/html/rustica/rustica-live-amd64-0.1.0.img`

---

## Live USB Image Specifications

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
        └── BOOTX64.EFI    (kernel.efi - the kernel)
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

## Dracula Theme (MANDATORY INVARIANT)

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

## Known Issues

### Keyboard IRQ Not Delivering (Active Investigation)

**Symptoms:**
- Green line appears (IOAPIC initialized)
- `int 33` test will confirm if IDT entry is correct
- If pixel changes on boot = IDT/handler works, problem is IRQ routing
- If nothing happens = IDT/gate type/selector is broken

**Attempts So Far:**
1. ✅ Fixed IOAPIC trigger mode (edge→level, high→low)
2. ✅ Enabled IA32_APIC_BASE MSR (bit 11)
3. ✅ Corrected LAPIC register offsets
4. ✅ Verified Local APIC EOI target

**Possible Remaining Issues:**
- Interrupt flag (IF) not set at runtime
- TPR blocking IRQs
- IDT entry type/selector incorrect
- PS/2 controller not actually generating IRQs

**Next Debug Steps:**
1. Check if `int 33` triggers the handler (test IDT)
2. Verify `sti` is called after ExitBootServices
3. Add direct port 0x64/0x60 polling test
4. May need UEFI SimpleTextInput fallback if PS/2 is dead

---

## Phase 6 Summary

### Completed (6A-6C)

- **6A: Input Subsystem** - PS/2 keyboard driver with scancode conversion, modifier tracking, circular buffer
- **6B: Display Subsystem** - Framebuffer driver, PSF2 font (8x16), text console, scrolling, Dracula colors
- **6C: Interactive Shell** - C shell with built-in commands (help, clear, echo, ps, exit), Dracula theme, program spawning

### In Progress (6D)

- **6D: Stability & UX** - Troubleshooting keyboard IRQ delivery to CPU
- **6E: Live Boot Media** - Build scripts and image creation (complete, awaiting IRQ fix)

---

## Planned Features (Phase 7+)

- **7A: USB HID Driver** - Keyboard + mouse support via USB
- **7B: GUI Server** - Single-process window manager (early Mac OS style)
- **7C: GUI Client Library** - librustica_gui for building GUI applications

---

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **Architecture** | x86_64 (AMD64) | x86_64 (AMD64) |
| **Boot** | UEFI 2.0 | UEFI 2.3+ |
| **RAM** | 512 MB | 1 GB |
| **Storage** | 128 MB (USB) | 4 GB |
| **Input** | PS/2 Keyboard | PS/2 or USB HID* |

\* USB HID support planned for Phase 7

---

## Documentation

- **[BUILD.md](rustica/docs/BUILD.md)** - Live USB build instructions
- **[IMAGE.md](rustica/docs/IMAGE.md)** - System architecture and boot flow
- **[PLAN.md](rustica/docs/PLAN.md)** - Development roadmap with detailed phase specs
- **[PASTE.md](rustica/docs/PASTE.md)** - Keyboard IRQ debugging notes

---

## Git Repositories

- **Kernel:** https://github.com/gitrustux/rustux
- **Transition Kernel:** https://github.com/gitrustux/rustux-efi
- **OS Distribution:** https://github.com/gitrustux/rustica
- **Applications:** https://github.com/gitrustux/apps

---

## License

MIT License - See LICENSE file for details.

---

*Last Updated: January 23, 2025*
**Status:** Phase 6A-6C Complete | 6D IRQ debugging in progress
