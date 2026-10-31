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
rustica/
├── docs/              # IMAGE.md (architecture, boot flow), PLAN.md (roadmap)
├── images/            # installer, live, vm images
├── releases/          # cli, desktop, server release definitions
├── repo/
│   ├── kernel/        # Rustux kernel (git submodule → gitrustux/rustux)
│   ├── apps/          # Packaged applications
│   ├── metadata/
│   └── system/
├── shell/             # Rust shell implementation (reference)
├── tools/             # CLI tools
└── update-system/     # Live update system (rpg, update-daemon)
```

Clone with the kernel included:

```bash
git clone --recurse-submodules https://github.com/gitrustux/rustica.git
# or, in an existing clone:
git submodule update --init repo/kernel
```

## Build Instructions

The kernel and live image are built from the kernel submodule. See the [Rustux README](https://github.com/gitrustux/rustux#building) for prerequisites.

```bash
cd repo/kernel

# Build kernel (UEFI application)
cargo build --release --bin rustux --features uefi_kernel --target x86_64-unknown-uefi

# Build userspace programs
cd test-userspace
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector \
    shell/shell.c -o shell/shell.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector \
    init.c -o init.elf
cd ..

# Build live USB image
./build-live-image.sh
```

To move to a newer kernel: `git submodule update --remote repo/kernel`, then commit the new pointer.

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **Architecture** | x86_64 (AMD64) | x86_64 (AMD64) |
| **Boot** | UEFI 2.0 | UEFI 2.3+ |
| **RAM** | 512 MB | 1 GB |
| **Storage** | 128 MB (USB) | 4 GB |
| **Input** | PS/2 Keyboard | PS/2 or USB HID* |

\* USB HID keyboard support is in progress (Phase 7A); USB-only machines currently have no keyboard input

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

- **[docs/IMAGE.md](docs/IMAGE.md)** - System architecture, boot flow, and image build
- **[docs/PLAN.md](docs/PLAN.md)** - Development roadmap with detailed phase specs
- **[Kernel docs](https://github.com/gitrustux/rustux/tree/main/docs)** - Kernel architecture, objects, syscalls

## Contributing

See [docs/PLAN.md](docs/PLAN.md) for:
- Coding standards
- Development workflow
- Phase specifications
- Technical decisions

## License

MIT License - See LICENSE file for details.

## Links

- **Kernel Repository:** https://github.com/gitrustux/rustux
- **Apps Repository:** https://github.com/gitrustux/apps
- **Website:** https://rustux.com
- **Issue Tracker:** https://github.com/gitrustux/rustux/issues

---

**Last Updated:** October 31, 2026
**Status:** Phase 6 COMPLETE - Interactive shell running with Dracula theme
