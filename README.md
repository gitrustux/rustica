# Rustica OS

The userspace OS distribution for the Rustux kernel - featuring an interactive shell, Dracula theme, and path toward a minimal GUI.

## Current Status

**Phase 6 COMPLETE: Interactive Shell** 🟢

- C shell implementation with built-in commands
- Dracula-themed console interface
- Process spawning and management
- Ramdisk filesystem integration

**Boot Flow:**
```
UEFI Firmware → BOOTX64.EFI → Kernel → Init (PID 1) → Shell (PID 2)
```

## Quick Start

### Download Live Image

Visit https://rustux.com/rustica/ to download the latest live USB image.

### Write to USB

```bash
# Identify your USB device
lsblk

# Write the image (replace /dev/sdX with your device)
sudo dd if=rustica-live-amd64-0.1.0.img of=/dev/sdX bs=4M status=progress conv=fsync
sudo sync
```

### Boot

1. Insert USB and restart your computer
2. Enter boot menu (F12, F2, F10, Del, or Esc key)
3. Select the USB drive (look for "UEFI: USB...")
4. System boots directly to the Rustux shell

## What's Working

### Shell Commands

| Command | Description |
|---------|-------------|
| `help` | Show available commands |
| `clear` | Clear the screen |
| `echo <text>` | Print text to console |
| `ps` | List running processes |
| `hello` | Run hello world program |
| `counter` | Run counter test program |
| `exit` | Exit the shell |

### Dracula Theme

The shell uses the Dracula color palette (mandatory invariant):

```
rustux> echo Hello World
```

Colors:
- Purple (`#BD93F9`) - Prompt and directory names
- Cyan (`#8BE9FD`) - Commands and executables
- Green (`#50FA7B`) - Success messages
- Red (`#FF5555`) - Error messages
- Orange (`#FFB86C`) - Warnings

## Project Structure

```
/var/www/rustux.com/prod/rustica/
├── docs/              # Documentation
│   ├── BUILD.md       # Live USB build instructions
│   ├── IMAGE.md       # System architecture and boot flow
│   ├── PLAN.md        # Development roadmap (Phases 1-7)
│   └── README.md      # This file
├── shell/             # Rust shell implementation (reference)
│   ├── src/
│   │   ├── main.rs
│   │   ├── parser.rs
│   │   ├── builtins.rs
│   │   └── theme.rs
│   └── Cargo.toml
└── update-system/     # Live update system (planned for future)
```

## Build Instructions

See BUILD.md for complete build instructions.

### Quick Build

```bash
cd /var/www/rustux.com/prod/rustux

# Build kernel
cargo build --release --target x86_64-unknown-uefi

# Build userspace programs
cd test-userspace
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector \
    shell.c -o shell.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector \
    init.c -o init.elf

# Build live USB image
./build-live-image.sh
```

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **Architecture** | x86_64 (AMD64) | x86_64 (AMD64) |
| **Boot** | UEFI 2.0 | UEFI 2.3+ |
| **RAM** | 512 MB | 1 GB |
| **Storage** | 128 MB (USB) | 4 GB |
| **Input** | PS/2 Keyboard | PS/2 or USB HID* |

\* USB HID support planned for Phase 7

## Roadmap

### Completed Phases

| Phase | Description | Status |
|-------|-------------|--------|
| **Phase 4** | Userspace & Process Execution | ✅ Complete |
| **Phase 5** | Process Management & Essential Syscalls | ✅ Complete |
| **Phase 6** | Input, Display, Interactive Shell | ✅ Complete |

### Phase 7: Minimal GUI (Planned)

| Subphase | Description | Timeline |
|----------|-------------|----------|
| **7A** | USB HID driver + Framebuffer mapping | 1-2 weeks |
| **7B** | GUI server process (rustica-gui) | 2-3 weeks |
| **7C** | GUI client library (librustica_gui) | 1-2 weeks |

**GUI Architecture (Early Mac OS Style):**
```
┌─────────────────────────────────────┐
│           Application (Rust)          │
│         (uses librustica_gui)        │
├─────────────────────────────────────┤
│          GUI Server (rustica-gui)    │
│    (owns framebuffer, input events)   │
├─────────────────────────────────────┤
│              Rustux Kernel            │
│  (syscalls, scheduler, drivers)      │
├─────────────────────────────────────┤
│              UEFI Firmware            │
│         (BOOTX64.EFI)                 │
└─────────────────────────────────────┘
```

### Future Phases (Beyond Phase 7)

- USB HID keyboard/mouse driver
- Minimal GUI with window manager
- Networking stack (TCP/IP)
- Ext2/3 filesystem support
- ARM64 and RISC-V architecture support
- Package manager
- Live update system

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

## Documentation

- **BUILD.md** - Complete live USB build instructions
- **IMAGE.md** - System architecture and boot flow
- **PLAN.md** - Development roadmap with detailed phase specs

## Contributing

See PLAN.md for:
- Coding standards
- Development workflow
- Phase specifications
- Technical decisions

## License

MIT License - See LICENSE file for details.

## Links

- **Main Repository:** https://github.com/gitrustux/rustux
- **Kernel Repository:** https://github.com/gitrustux/rustux
- **Apps Repository:** https://github.com/gitrustux/apps
- **Website:** https://rustux.com
- **Issue Tracker:** https://github.com/gitrustux/rustux/issues

---

**Last Updated:** January 23, 2025
**Status:** Phase 6 COMPLETE - Interactive shell running with Dracula theme
