# Rustica OS - Kernel Integration Plan

**Date:** 2025-01-18
**Status:** Phase 3A Complete → Phase 4 (Userspace Foundation)
**Last Milestone:** CLI tool with QEMU integration ✅
**Next Milestone:** Phase 4A - ELF Loader implementation

---

## Executive Summary

The Rustux kernel has been refactored into a modern Zircon-inspired architecture located at `/var/www/rustux.com/prod/rustux/`. This plan outlines the integration of the refactored kernel into the Rustica OS distribution.

**Key Changes:**
- **CLI First**: All kernel interaction will be through the CLI tools (no GUI initially)
- **GUI On Hold**: Desktop environment integration is postponed until CLI is complete
- **Architecture Support**: AMD64 fully supported, ARM64/RISC-V placeholders ready

---

## Quick Reference: Kernel Status

### ✅ Completed: Phase 2C Migration (~13,500 lines)

| Phase | Component | Status | Lines |
|-------|-----------|--------|-------|
| 2C.1 | AMD64 Architecture | ✅ Complete | ~2,500 |
| 2C.2 | Memory Management | ✅ Complete | ~1,500 |
| 2C.3 | Process & Thread Management | ✅ Complete | ~2,000 |
| 2C.4 | Synchronization Primitives | ✅ Complete | ~1,000 |
| 2C.5 | Objects & Capabilities | ✅ Complete | ~2,500 |
| 2C.6 | System Calls | ✅ Complete | ~1,500 |
| 2C.7 | Device Drivers (UART) | ✅ Complete | ~500 |
| 2C.8 | ARM64 & RISC-V Support | ✅ Complete | ~1,500 |

**Kernel Location:** `/var/www/rustux.com/prod/rustux/`

**Repository:** https://github.com/gitrustux/rustux

---

## Important: Existing Userspace CLI Tools

**Location:** `/var/www/rustux.com/prod/rustica/tools/cli/`

The Rustica OS project already has a comprehensive set of **userspace CLI tools** (~5,150 lines) that are designed to run ON TOP of the kernel. These are NOT kernel management tools.

### Userspace CLI Tools (Completed ✅)

| Category | Tools | Status | Location |
|----------|-------|--------|----------|
| Shell | `sh` | ✅ Complete | `src/sh/` |
| Init | `init` | ✅ Complete | `src/init/` |
| Core Utils | `ls`, `cat`, `cp`, `mv`, `rm`, `mkdir`, `touch`, `echo` | ✅ Complete | `src/coreutils/` |
| System Utils | `ps`, `kill`, `dmesg`, `uname`, `date` | ✅ Complete | `src/sysutils/` |
| Networking | `ip`, `ping`, `hostname`, `nslookup` | ✅ Complete | `src/networkutils/` |
| Package Mgr | `pkg` | ✅ Complete | `src/pkgutil/` |
| Firewall | `fwctl` | ✅ Complete | `src/fwctl/` |
| Storage | `mount`, `umount`, `blklist`, `mkfs-rfs` | ✅ Complete | `src/storageutils/` |
| Services | `svc`, `system-check` | ✅ Complete | `src/svc/` |

**Build Command:**
```bash
cd /var/www/rustux.com/prod/rustica/tools/cli
cargo build --release
```

**Documentation:** See `/var/www/rustux.com/prod/rustica/tools/cli/README.md`

### Distinction: Userspace CLI vs Kernel Management CLI

- **Userspace CLI** (`/var/www/rustux.com/prod/rustica/tools/cli/`): Tools that run ON the OS (sh, ls, pkg, etc.)
- **Kernel Management CLI**: Tools to BUILD and TEST the kernel itself (build kernels, run QEMU, create images)

Both are needed, but they serve different purposes.

### QEMU Validation Script

**Location:** `/var/www/rustux.com/prod/rustica/tools/cli/scripts/qemu-validation.sh`

**Status:** Needs updating for new kernel location

The script currently references the old kernel location (`target/x86_64-unknown-none/release/rustux`) and needs to be updated to work with:
- **New Kernel:** `/var/www/rustux.com/prod/rustux/`
- **New Target:** `x86_64-unknown-uefi` (UEFI bootloader)
- **New Binary:** `rustux.efi`

---

## Part 1: Kernel Directory Structure

### Current Refactored Layout

```
/var/www/rustux.com/prod/rustux/
├── src/
│   ├── main.rs                 # Kernel entry point
│   ├── lib.rs                  # Library root with module declarations
│   ├── init.rs                 # Initialization code
│   ├── test_entry.rs           # Test entry point
│   ├── traits.rs               # Common traits (InterruptController, etc.)
│   │
│   ├── acpi/                   # ACPI table parsing
│   │   ├── rsdp.rs
│   │   ├── sdt.rs
│   │   └── mod.rs
│   │
│   ├── arch/                   # Architecture-specific code
│   │   ├── mod.rs              # Architecture module root
│   │   ├── amd64/              # x86_64 architecture (FULLY IMPLEMENTED)
│   │   │   ├── mod.rs          # AMD64 module root
│   │   │   ├── bootstrap16.rs  # 16-bit bootstrap code
│   │   │   ├── cache.rs        # Cache management
│   │   │   ├── descriptor.rs   # GDT/IDT descriptors
│   │   │   ├── faults.rs       # Exception handlers
│   │   │   ├── idt.rs          # Interrupt Descriptor Table
│   │   │   ├── init.rs         # AMD64 initialization
│   │   │   ├── ioport.rs       # Port I/O
│   │   │   ├── mm/             # AMD64 memory management
│   │   │   │   ├── mod.rs
│   │   │   │   ├── page_tables.rs
│   │   │   │   └── mmu.rs
│   │   │   ├── mod.rs
│   │   │   ├── ops.rs          # CPU operations
│   │   │   ├── registers.rs    # CPU registers
│   │   │   ├── syscall.rs      # AMD64 syscall interface
│   │   │   ├── tsc.rs          # Time Stamp Counter
│   │   │   └── uspace_entry.rs # Userspace entry
│   │   │
│   │   ├── arm64/              # ARM64 architecture (PLACEHOLDER)
│   │   │   ├── mod.rs
│   │   │   ├── arch.rs         # Architecture definitions
│   │   │   ├── interrupt/      # GIC interrupt controller
│   │   │   │   ├── mod.rs
│   │   │   │   └── gic.rs
│   │   │   └── mm/             # ARM64 MMU
│   │   │       └── mod.rs
│   │   │
│   │   └── riscv64/            # RISC-V architecture (PLACEHOLDER)
│   │       ├── mod.rs
│   │       ├── arch.rs         # Architecture definitions
│   │       ├── interrupt/      # PLIC interrupt controller
│   │       │   ├── mod.rs
│   │       │   └── plic.rs
│   │       └── mm/             # RISC-V MMU
│   │           └── mod.rs
│   │
│   ├── drivers/                # Device drivers
│   │   ├── mod.rs
│   │   └── uart.rs             # UART driver
│   │
│   ├── interrupt/              # Interrupt handling
│   │   ├── mod.rs
│   │   └── pic.rs              # 8259 PIC
│   │
│   ├── mm/                     # Memory management
│   │   ├── mod.rs
│   │   ├── allocator.rs        # Page allocator
│   │   └── pmm.rs              # Physical memory manager
│   │
│   ├── object/                 # Zircon-style kernel objects
│   │   ├── mod.rs
│   │   ├── handle.rs           # Handle, Rights, HandleTable
│   │   ├── event.rs            # Event objects
│   │   ├── timer.rs            # Timer objects
│   │   ├── channel.rs          # IPC channels
│   │   ├── vmo.rs              # Virtual Memory Objects
│   │   └── job.rs              # Job objects
│   │
│   ├── process/                # Process management
│   │   ├── mod.rs
│   │   └── process.rs          # Process, Thread, AddressSpace
│   │
│   ├── sched/                  # Scheduler
│   │   └── mod.rs
│   │
│   ├── sync/                   # Synchronization primitives
│   │   ├── mod.rs
│   │   ├── spinlock.rs         # SpinLock
│   │   ├── event.rs            # Event (renamed to SyncEvent)
│   │   └── wait_queue.rs       # WaitQueue
│   │
│   ├── syscall/                # System call interface
│   │   ├── mod.rs
│   │   └── definitions.rs      # Syscall number definitions
│   │
│   └── testing/                # Testing utilities
│       └── mod.rs
│
├── build.sh                    # Build script
├── test-qemu.sh                # QEMU test script
├── scripts/
│   └── create-bootable-image.sh
├── Cargo.toml                  # Workspace configuration
└── target/                     # Build output
```

### Old Kernel Location (To Be Deprecated)

```
/var/www/rustux.com/prod/kernel/         # OLD - Will be deprecated
├── kernel-efi/               # UEFI kernel (to be replaced)
├── uefi-loader/              # UEFI bootloader
├── src/kernel/               # Old kernel source (deprecated)
└── build-live-image.sh       # Build script (may be reused)
```

---

## Part 2: CLI Integration Plan

### Phase 1: Kernel CLI Tool (Priority: HIGH)

Create a new CLI tool `rustux-kernel` at `/var/www/rustux.com/prod/apps/cli/rustux-kernel/` that provides:

#### 1.1 Build & Test Commands

```bash
# Build kernel for specific architecture
rustux-kernel build --arch amd64
rustux-kernel build --arch arm64
rustux-kernel build --arch riscv64

# Run kernel in QEMU
rustux-kernel test --qemu
rustux-kernel test --qemu --arch amd64 --memory 512M

# Run unit tests
rustux-kernel test --unit
rustux-kernel test --integration

# Create bootable image
rustux-kernel image --output rustux.img --size 128M
```

#### 1.2 Kernel Information Commands

```bash
# Show kernel version and build info
rustux-kernel version

# Show supported features
rustux-kernel features

# Show architecture support status
rustux-kernel arch status
```

#### 1.3 Debug & Development Commands

```bash
# Run kernel with debug console
rustux-kernel debug --console serial

# Generate syscall coverage report
rustux-kernel coverage syscall

# Generate memory map
rustux-kernel debug memory-map
```

#### 1.4 Implementation Structure

```
/var/www/rustux.com/prod/apps/cli/rustux-kernel/
├── Cargo.toml
└── src/
    ├── main.rs              # CLI entry point
    ├── build.rs             # Build commands
    ├── test.rs              # Test commands
    ├── image.rs             # Image creation
    ├── qemu.rs              # QEMU integration
    ├── arch.rs              # Architecture detection
    └── info.rs              # Information commands
```

---

### Phase 2: Syscall Testing CLI (Priority: HIGH)

Extend existing CLI tools to test kernel syscalls:

#### 2.1 Integration with Existing Tools

- **`capctl`**: Test capability-based security with kernel objects
- **`svc`**: Test process/thread management syscalls
- **New tool `syscall-test`**: Dedicated syscall testing suite

#### 2.2 Syscall Test Commands

```bash
# Test object creation
syscall-test create-object --type vmo --size 4096

# Test handle operations
syscall-test handle-duplicate --id 123 --rights READ,WRITE

# Test IPC channels
syscall-test channel-create --read-buf-size 4096

# Test timer objects
syscall-test timer-set --deadline 1000000 --slack 1000
```

---

### Phase 3: Package Integration (Priority: MEDIUM)

#### 3.1 Kernel as RPG Package

Create `.rpg` package for kernel distribution:

```json
{
  "name": "rustux-kernel",
  "version": "0.2.0",
  "type": "kernel",
  "arch": "x86_64",
  "description": "Rustux microkernel with Zircon-style objects",
  "files": [
    "boot/vmlinuz-rustux",
    "boot/config-rustux",
    "lib/modules/0.2.0/kernel/*.ko"
  ]
}
```

#### 3.2 Update Commands

```bash
# Update kernel package
rpg update rustux-kernel

# Rollback to previous kernel
rpg rollback rustux-kernel

# List available kernels
rpg list --type kernel
```

---

## Part 3: Image Building Updates

### Update `/var/www/rustux.com/prod/kernel/build-live-image.sh`

Modify to use refactored kernel:

```bash
#!/bin/bash
# Updated build script for refactored kernel

KERNEL_DIR="/var/www/rustux.com/prod/rustux"
BUILD_TARGET="x86_64-unknown-uefi"

# Build refactored kernel
cd "$KERNEL_DIR"
cargo build --release --bin rustux --features uefi_kernel --target $BUILD_TARGET

# Copy to staging area
cp target/$BUILD_TARGET/release/rustux.efi $STAGING_DIR/EFI/BOOT/BOOTX64.EFI
cp target/$BUILD_TARGET/release/rustux.efi $STAGING_DIR/EFI/Rustux/kernel.efi
```

---

## Part 4: Testing Strategy

### Unit Tests (Already in Place)

Each module has `#[cfg(test)]` tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_handle_create() {
        // Test implementation
    }
}
```

### Integration Tests (To Be Added)

Create `/var/www/rustux.com/prod/rustux/tests/integration/`:

```
integration/
├── syscall_tests.rs       # Syscall integration tests
├── object_tests.rs        # Object creation and manipulation
├── process_tests.rs       # Process/thread tests
├── ipc_tests.rs           # IPC channel tests
└── stress_tests.rs        # Stress testing
```

### QEMU Test Automation

Extend `test-qemu.sh` for comprehensive testing:

```bash
# Test specific functionality
./test-qemu.sh --test timer
./test-qemu.sh --test keyboard
./test-qemu.sh --test syscalls

# Run all tests
./test-qemu.sh --all
```

---

## Part 5: Documentation Updates

### Required Documentation

1. **Update IMAGE.md** (see section below)
2. **Create `/var/www/rustux.com/prod/rustux/docs/ARCHITECTURE.md`**
   - Kernel architecture overview
   - Module interaction diagrams
   - Syscall reference

3. **Create `/var/www/rustux.com/prod/rustux/docs/SYSCALL.md`**
   - Complete syscall reference
   - Usage examples
   - Return codes

4. **Create `/var/www/rustux.com/prod/rustux/docs/OBJECTS.md`**
   - Kernel object reference
   - Handle operations
   - Capability security model

---

## Part 6: IMAGE.md Updates Required

### Changes Needed to `/var/www/rustux.com/prod/rustica/docs/IMAGE.md`

1. **Update kernel location references:**
   - Change from `/var/www/rustux.com/prod/kernel/` to `/var/www/rustux.com/prod/rustux/`

2. **Update build instructions:**
   ```bash
   # New location
   cd /var/www/rustux.com/prod/rustux
   cargo build --release --bin rustux --features uefi_kernel
   ```

3. **Update Phase 2C completion status:**
   - Add section documenting Phase 2C completion
   - List all migrated modules

4. **Add CLI tool references:**
   - Document `rustux-kernel` CLI tool
   - Add usage examples for kernel management

---

## Part 7: Implementation Order

### Immediate (Week 1-2)

1. ✅ **DONE**: Phase 2C migration complete
2. ⏳ **TODO**: Create basic `rustux-kernel` CLI skeleton
   - `rustux-kernel build` (wrap cargo build)
   - `rustux-kernel test` (wrap test-qemu.sh)
   - `rustux-kernel version`
3. ⏳ **TODO**: Document kernel build process in ARCHITECTURE.md

### Short Term (Week 3-4)

4. ⏳ **TODO**: Add QEMU integration to CLI
5. ⏳ **TODO**: Add arch detection
6. ⏳ **TODO**: Update build-live-image.sh
7. ⏳ **TODO**: Add integration tests

### Medium Term (Month 2)

8. ⏳ **TODO**: Implement syscall test suite
9. ⏳ **TODO**: Create kernel RPG packages
10. ⏳ **TODO**: Update IMAGE.md
11. ⏳ **TODO**: Create ARCHITECTURE.md
12. ⏳ **TODO**: ARM64 native testing
13. ⏳ **TODO**: RISC-V native testing
14. ⏳ **TODO**: Performance benchmarking
15. ⏳ **TODO**: Security audit

### Long Term (Month 3+)

16. ⏳ **TODO**: GUI integration (when CLI is complete)
17. ⏳ **TODO**: Desktop environment integration
18. ⏳ **TODO**: Mobile device testing

---

## Part 8: Dependencies & Prerequisites

### External Dependencies

| Dependency | Version | Purpose | Status |
|------------|---------|---------|--------|
| Rust | 1.75+ | Language | ✅ Installed |
| QEMU | 7.0+ | Testing | ✅ Installed |
| OVMF | 2022.11+ | UEFI firmware | ✅ Installed |
| cargo | Latest | Build tool | ✅ Installed |

### Internal Dependencies

| Component | Location | Required By | Status |
|-----------|----------|-------------|--------|
| rpg-core | rustica/update-system/rpg-core | Package management | ✅ Complete |
| capctl | apps/cli/capctl | Capability testing | ✅ Complete |
| rutils | apps/libs/rutils | Utilities | ✅ Complete |

---

## Part 9: Risk Assessment

### High Risk Items

1. **UEFI Boot Issues** ⚠️
   - **Risk**: ExitBootServices failures
   - **Mitigation**: Use proven image format from working kernel
   - **Status**: Documented in IMAGE.md

2. **Syscall Compatibility** ⚠️
   - **Risk**: New syscall numbers may break existing tools
   - **Mitigation**: Maintain compatibility layer
   - **Status**: Need to audit existing tools

### Medium Risk Items

3. **ARM64/RISC-V Support**
   - **Risk**: Placeholder implementations may not work
   - **Mitigation**: Mark as experimental
   - **Status**: Placeholders ready, testing needed

4. **Performance**
   - **Risk**: New architecture may have performance issues
   - **Mitigation**: Benchmark against old kernel
   - **Status**: Need benchmarks

---

## Part 10: Rollback Plan

If critical issues arise:

1. **Keep old kernel** at `/var/www/rustux.com/prod/kernel-old/`
2. **Maintain old image builds** in `images/legacy/`
3. **Revert package** to old kernel: `rpg rollback rustux-kernel`
4. **Document issues** in `docs/ROLLBACK.md`

---

## Part 11: Proven Working Features (2025-01-18)

### ✅ Verified Functional

| Feature | Status | Evidence |
|---------|--------|----------|
| UEFI Boot | ✅ Working | Boots to kernel mode |
| GDT Setup | ✅ Working | No triple faults |
| IDT Setup | ✅ Working | Handles exceptions |
| APIC Init | ✅ Working | LAPIC enabled |
| Timer Interrupts | ✅ Working | `[TICK]` messages in QEMU |
| Keyboard IRQ | ✅ Routed | IRQ1 → Vector 33 configured |
| ACPI Discovery | ✅ Working | RSDP found at 0x... |
| Exit Boot Services | ✅ Working | Transitions cleanly |

### ⚠️ Not Yet Tested

| Feature | Status | Reason |
|---------|--------|--------|
| Keyboard Input | 🔶 Partial | Handler installed, not tested in QEMU |
| Syscalls | ❌ Untested | No userspace yet |
| Process Creation | ❌ Untested | No scheduler started |
| Memory Allocation | ❌ Untested | PMM not initialized |

### ❌ Not Implemented (Current Limitations)

| Feature | Status | Notes |
|---------|--------|-------|
| **Userspace** | ❌ Not Implemented | No process execution, no ELF loader |
| **Process Execution** | ❌ Not Implemented | Scheduler exists but not started |
| **Filesystem** | ❌ Not Implemented | No VFS layer, no storage drivers |
| **Installer** | ❌ Not Implemented | Kernel-only, no OS installer |
| **Syscalls** | 🔶 Stub Only | 1 working (CLOCK_GET), 28 stubs |
| **Network** | ❌ Not Implemented | No network stack |
| **GUI** | ❌ Not Implemented | On hold until CLI complete |

**Note:** The kernel is currently a bare microkernel that boots to runtime mode. Userspace CLI tools exist at `/var/www/rustux.com/prod/rustica/tools/cli/` but cannot run until process execution is implemented.

---

## Part 12: Success Criteria (Updated)

### Phase 3A Success (2025-01-18 - Session Summary)

**Completed:**
- ✅ PLAN.md updated with existing userspace CLI information
- ✅ Kernel tested and boots successfully in QEMU
- ✅ ARCHITECTURE.md documentation created (comprehensive kernel architecture doc)
- ✅ Timer interrupts verified working ([TICK] messages in debug log)
- ✅ Bootable image creation verified working

**Kernel Test Results (2025-01-18):**
```
✓ UEFI boot successful
✓ ACPI RSDP discovered
✓ Exit boot services clean
✓ GDT configured
✓ IDT configured
✓ Timer handler installed (vector 32)
✓ Keyboard handler installed (vector 33)
✓ APIC initialized
✓ Keyboard IRQ configured (IRQ1 → Vector 33)
✓ Timer configured and running
✓ [TICK] messages verified
```

**Documentation Created:**
- `/var/www/rustux.com/prod/rustux/docs/ARCHITECTURE.md` - Complete kernel architecture reference

**Still Pending:**
- ⏳ Integration tests pass (at least 5 tests)
- ⏳ CLI can create bootable USB image
- ⏳ At least one userspace program runs

### Previous Phase Success Criteria

Phase 2C (Completed):
- ✅ All Phase 2C modules compiled (82 warnings remaining, all non-critical)
- ✅ Kernel boots to runtime mode
- ⏳ Basic syscalls work (process create, memory allocate)
- ⏳ Integration test suite passes
- ⏳ RPG package can be installed and updated
- ⏳ Documentation is complete

---

## Part 13: Out of Scope (For Now)

**DO NOT attempt until CLI is stable:**
- ❌ GUI integration
- ❌ Desktop environment
- ❌ Native ARM64/RISC-V testing (emulation OK)
- ❌ Performance optimization (correctness first)
- ❌ Security hardening (functional first)

**DO NOT attempt until userspace works:**
- ❌ Full syscall suite (start with 5-10 basic calls)
- ❌ Complex IPC patterns
- ❌ Multi-process scenarios

**Reason:** Build incrementally. Each layer must be solid before adding the next.

---

## Part 14: Phase 4 - Userspace & Live Image Implementation

**Goal:** Transform bare kernel into bootable live OS with working CLI tools
**Status:** Phase 3A (CLI) → Phase 4 (Userspace Foundation)
**Duration:** 6-8 weeks estimated

### Overview

This phase transforms the bare microkernel (which boots to runtime mode) into a bootable live OS with working userspace CLI tools.

### Phase 4A: ELF Loader (CRITICAL - Week 1-2)
**Priority:** 🔴 HIGHEST - Nothing else works without this

#### 4A.1: Implement ELF Parser
```rust
// src/exec/elf.rs
struct ElfHeader {
    e_ident: [u8; 16],     // Magic number: 0x7F 'ELF'
    e_type: u16,           // Relocatable, Executable, etc.
    e_machine: u16,        // Architecture: x86_64
    e_entry: u64,         // Entry point address
    // ...
}

struct ProgramHeader {
    p_type: u32,          // LOAD, DYNAMIC, INTERP, etc.
    p_flags: u32,         // R, W, X permissions
    p_vaddr: u64,         // Virtual address
    p_paddr: u64,         // Physical address
    p_filesz: u64,        // Size in file
    p_memsz: u64,         // Size in memory
    p_offset: u64,        // Offset in file
}
```

#### 4A.2: Map ELF Segments
- Create VMO for code segment (LOAD, R+X)
- Create VMO for data segment (LOAD, R+W)
- Create VMO for BSS segment (zero-filled)
- Handle dynamic linking (initially: reject dynamic ELFs)

#### 4A.3: Set Up Initial User Stack
- Allocate stack VMO (default: 8MB)
- Map stack at high address (e.g., 0x7fff_ffff_f000)
- Push argc, argv, envp

#### 4A.4: Create Initial Thread
- Set instruction pointer to ELF entry
- Set stack pointer to user stack
- Set up user mode segment selectors

#### 4A.5: Success Criteria
- ✅ Can load static ELF binary
- ✅ Can jump to user mode
- ✅ Binary executes at least one instruction

---

### Phase 4B: Syscall Implementation (CRITICAL - Week 2-3)
**Priority:** 🔴 HIGHEST - Userspace needs working syscalls

#### 4B.1: Essential Syscalls (Minimum Viable Set)

Implement these 10 syscalls first:

| Syscall | Priority | Description |
|---------|----------|-------------|
| `sys_exit` | 🔴 Critical | Process termination |
| `sys_write` | 🔴 Critical | Console output (stdout/stderr) |
| `sys_read` | 🔴 Critical | Console input (stdin) |
| `sys_mmap` | 🔴 High | Memory allocation |
| `sys_munmap` | 🟡 Medium | Memory deallocation |
| `sys_brk` | 🟡 Medium | Heap management |
| `sys_clock_gettime` | ✅ Done | Time queries (already working!) |
| `sys_nanosleep` | 🟡 Medium | Sleep/delays |
| `sys_getpid` | 🟢 Low | Get process ID |
| `sys_kill` | 🟢 Low | Signal delivery |

#### 4B.2: Syscall Descriptions

**sys_exit(status)**
- Clean up process resources
- Remove from scheduler
- Return status to parent (if any)

**sys_write(fd, buf, count)**
- Validate fd (initially: only stdout/stderr = 1/2)
- Copy buffer from userspace
- Write to debug console (port 0xE9 for now)
- Return bytes written

**sys_read(fd, buf, count)**
- Validate fd (initially: only stdin = 0)
- Block until input available
- Copy to user buffer
- Return bytes read

**sys_mmap(addr, length, prot, flags)**
- Create VMO of requested size
- Map into process address space
- Set protection flags (R/W/X)
- Return mapped address

**sys_munmap(addr, length)**
- Find VMO at address
- Unmap from address space
- Destroy VMO

**sys_brk(addr)**
- Adjust process heap end
- Allocate/deallocate pages as needed
- Return new heap end

#### 4B.3: Success Criteria
- ✅ Can call sys_write from userspace
- ✅ Can see output on debug console
- ✅ Can allocate memory with sys_mmap
- ✅ Can exit with sys_exit

---

### Phase 4C: Scheduler Start (HIGH - Week 3)
**Priority:** 🟠 HIGH - Needed for multi-process

#### 4C.1: Bootstrap Initial Process
- Create init process (PID 1)
- Load /sbin/init ELF
- Set up address space
- Create initial thread
- Add to run queue

#### 4C.2: Start Scheduler
- Enable timer interrupts for preemption
- Implement context switch in timer handler
- Round-robin scheduling initially

#### 4C.3: Process Spawning
- `sys_fork()` - Create child process
- `sys_execve()` - Replace process image
- `sys_waitpid()` - Wait for child termination

#### 4C.4: Success Criteria
- ✅ Can run init process (PID 1)
- ✅ Can fork child process
- ✅ Can switch between processes
- ✅ Timer preemption works

---

### Phase 4D: Minimal Filesystem (HIGH - Week 4)
**Priority:** 🟠 HIGH - Needed to load programs

#### 4D.1: Initial Ramdisk (initrd)
Don't implement a full VFS yet - just load files from memory:

**Create initrd format:**
```
Simple tar-like format: [header][data][header][data]...
Header: {name: [256]u8, size: u64, offset: u64}
```

**Implement initrd parser:**
- Parse headers
- Build file table in memory
- Lookup files by path

**Implement minimal file operations:**
- `sys_open(path, flags)` - Open file from initrd
- `sys_close(fd)` - Close file descriptor
- `sys_read(fd, buf, count)` - Read from initrd file
- `sys_stat(path, buf)` - Get file info

**Files to include in initrd:**
- `/sbin/init` - Init process (PID 1)
- `/bin/sh` - Shell
- `/bin/ls` - List files
- `/bin/cat` - Display files
- `/bin/echo` - Print text

#### 4D.2: Success Criteria
- ✅ Can load files from initrd
- ✅ Can open, read, close files
- ✅ Can execute programs from initrd
- ✅ Shell runs from /bin/sh

---

### Phase 4E: Console Driver (MEDIUM - Week 4-5)
**Priority:** 🟡 MEDIUM - Better than debug console

#### 4E.1: Choose Console Type

**Option A: VGA Text Mode (simpler)**
- Initialize VGA buffer at 0xB8000
- Implement scrolling
- Handle cursor positioning
- Map to sys_write for stdout

**Option B: Serial Console (better for debugging)**
- Initialize UART (COM1: 0x3F8)
- Configure baud rate (115200)
- Implement TX/RX buffers
- Map to sys_write/sys_read

#### 4E.2: Success Criteria
- ✅ Console replaces debug port
- ✅ Can type and see echo
- ✅ Can scroll output
- ✅ Cursor positioning works

---

### Phase 4F: Live Image Creation (MEDIUM - Week 5)
**Priority:** 🟡 MEDIUM - Packaging for distribution

#### 4F.1: Bootable Image Structure

```
FAT32 image:
  /EFI/BOOT/BOOTX64.EFI   # Kernel
  /initrd.tar              # Initial ramdisk
  /boot/config             # Kernel config
```

#### 4F.2: Update build-live-image.sh
1. Build kernel
2. Build userspace programs
3. Create initrd with programs
4. Package into bootable image

#### 4F.3: Success Criteria
- ✅ Boots from USB
- ✅ Runs on real hardware
- ✅ Shell is interactive
- ✅ Basic commands work

---

### Phase 4G: Basic Installer (LOW - Week 6+)
**Priority:** 🟢 LOW - Nice to have, not critical

Defer this until Phase 4A-4E complete.

---

### Dependency Graph

```
Phase 4A (ELF Loader)
    ↓
Phase 4B (Syscalls) ← Must have 4A
    ↓
Phase 4C (Scheduler) ← Must have 4A + 4B
    ↓
Phase 4D (Initrd) ← Must have 4B (file syscalls)
    ↓
Phase 4E (Console) ← Can happen anytime after 4B
    ↓
Phase 4F (Live Image) ← Must have 4A-4D working
    ↓
Phase 4G (Installer) ← Needs everything
```

---

### Success Criteria Summary

| Phase | Success Criteria |
|-------|-----------------|
| **4A** | ELF loads, jumps to user mode, executes instruction |
| **4B** | sys_write output, sys_mmap allocates, sys_exit works |
| **4C** | Init runs, fork works, preemption works |
| **4D** | Can exec programs from initrd, shell runs |
| **4E** | Console displays output, can type and see echo |
| **4F** | USB boots on real hardware, shell interactive |
| **4G** | Can install to disk from live USB |

---

### Time Estimates

| Phase | Effort | Duration |
|-------|--------|----------|
| 4A - ELF Loader | Medium | 1-2 weeks |
| 4B - Syscalls | High | 1-2 weeks |
| 4C - Scheduler | Medium | 1 week |
| 4D - Initrd | Low | 3-5 days |
| 4E - Console | Low | 3-5 days |
| 4F - Live Image | Low | 2-3 days |
| 4G - Installer | Medium | 1 week |
| **Total** | | **6-8 weeks** |

---

### Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| ELF loader bugs | 🔴 Critical | Test with simple binaries first |
| Syscall interface wrong | 🔴 Critical | Use Linux syscall ABI |
| Context switch crashes | 🟠 High | Test scheduler in isolation |
| Initrd format issues | 🟡 Medium | Use standard tar format |
| Hardware compatibility | 🟡 Medium | Test in QEMU first |

---

### What NOT to Implement (Yet)

Defer these until Phase 4A-4F complete:

- ❌ Full VFS layer (use initrd only)
- ❌ Disk drivers (boot from memory)
- ❌ Network stack
- ❌ GUI/Wayland
- ❌ Package manager integration
- ❌ Multi-user support
- ❌ Security hardening
- ❌ ARM64/RISC-V ports

---

### Quick Start: Week 1 Tasks

Focus on **4A (ELF Loader)** first:

1. Create `src/exec/elf.rs` module
2. Implement ELF header parsing
3. Create simple test binary: `hello.c`
4. Load ELF into memory
5. Jump to entry point
6. **Celebrate first userspace instruction!** 🎉

---

## Part 15: Contact & Resources

### Key Locations

- **Kernel Code**: `/var/www/rustux.com/prod/rustux/`
- **CLI Tools**: `/var/www/rustux.com/prod/apps/cli/`
- **This Plan**: `/var/www/rustux.com/prod/rustica/docs/PLAN.md`
- **Image Docs**: `/var/www/rustux.com/prod/rustica/docs/IMAGE.md`

### Git Repositories

- **Kernel**: https://github.com/gitrustux/rustux
- **CLI Tools**: Part of rustica workspace

### Documentation References

- Zircon Kernel Objects: https://fuchsia.dev/fuchsia-src/concepts/kernel/concepts
- UEFI Specification: https://uefi.org/specifications
- Wayland Protocol: https://wayland.freedesktop.org/

---

*Last Updated: 2025-01-18*

**Next Review:** After CLI tool implementation (Week 2)

---

## Appendix: Quick Reference for New Sessions

When starting a new session to continue this work:

1. **Read this file**: `/var/www/rustux.com/prod/rustica/docs/PLAN.md`
2. **Check kernel status**: `cd /var/www/rustux.com/prod/rustux && cargo build`
3. **Review existing tests**: `cd /var/www/rustux.com/prod/rustux && find . -name "*.rs" -exec grep -l "#\[cfg(test)\]" {} \;`
4. **Check CLI tools**: `ls /var/www/rustux.com/prod/apps/cli/`
5. **Run QEMU test**: `cd /var/www/rustux.com/prod/rustux && ./test-qemu.sh`

**Current Status**: Phase 2C complete, ready for CLI integration
