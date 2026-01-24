# Rustux OS - Phase 6: Interactive Shell (January 2025)

**Status:** 🟡 Phase 6A-6C COMPLETE | Phase 6D Keyboard IRQ blocked - UNSOLVED

---

## Project Overview

**Rustux OS** is a hobby operating system written in Rust, featuring a native UEFI microkernel with an interactive shell and Dracula-themed interface.

**Boot Flow:**
```
UEFI Firmware → BOOTX64.EFI → Transition Kernel → Shell (CLI with [POLLING])
```

---

## Current Status (January 24, 2025)

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

### ❌ BLOCKED: Phase 6D (Keyboard IRQ Delivery)

**Current Symptom:** Shell boots but shows **[POLLING]** message. Keyboard input only works via polling fallback, not via interrupts.

**Visual Debug Markers Observed:**
- Solid green line (0,0)-(20,3) → IOAPIC configured, unmasked
- Blue dot at (2,0) → LAPIC ID = 0 (BSP detected)
- Yellow pixel at (1,0) → LAPIC base matches 0xFEE00000
- **No '!' at VGA 0xB8000** → IRQ stub never executes
- **No pixel changes at top-right** → IRQ never fires

**What This Means:**
All hardware is configured correctly, but IRQs are **not reaching the CPU**. The system falls back to polling mode which works, confirming:
- ✅ PS/2 keyboard hardware is functional
- ✅ I/O ports (0x60/0x64) are accessible
- ❌ Interrupt delivery path is broken somewhere

---

## Debugging History: All Fixes Attempted

### Fix #1: IRQ Stub Register Ordering ✅
**Problem:** VGA debug write happened BEFORE saving registers
**Impact:** rax corrupted, iretq returned to garbage
**Fix:** Save ALL registers FIRST, then debug writes
**Result:** System no longer hangs at interrupt init

### Fix #2: IOAPIC Trigger Mode ✅
**Problem:** IOAPIC configured as level-triggered + active-low
**Impact:** "One IRQ only" symptom
**Fix:** Changed to edge-triggered + active-high (remove flags)
**Result:** Green line appears, but IRQ still doesn't fire

### Fix #3: IOAPIC Destination APIC ID ✅
**Problem:** high_dword = 0 routes IRQ1 to APIC ID 0
**Impact:** IRQ delivered to non-existent CPU
**Fix:** Read BSP APIC ID, route to actual BSP
**Result:** Blue dot appears (ID=0 on this system), but IRQ still doesn't fire

### Fix #4: EOI Address in IRQ Stub ✅
**Problem:** IRQ stub sent EOI to 0xFEE00040 (wrong offset)
**Impact:** EOI never acknowledged, no further IRQs
**Fix:** Changed to 0xFEE000B0 (correct EOI offset)
**Result:** No change (still no IRQ fires)

### Fix #5: Remove Duplicate EOI ✅
**Problem:** Both stub AND handler sent EOI
**Impact:** Double EOI can cause APIC malfunction
**Fix:** Removed pic_send_eoi() from handler
**Result:** No change (still no IRQ fires)

### Fix #6: Add sti to Enable CPU Interrupts ✅
**Problem:** UEFI leaves IF=0, sti was never called
**Impact:** CPU silently drops all external interrupts
**Fix:** Added `sti` at end of init_keyboard_interrupts()
**Result:** No change (still no IRQ fires, still shows POLLING)

### Fix #7: Correct LocalApicRegisters Struct ✅
**Problem:** Struct had eoi at offset 0x40, actual offset is 0xB0
**Impact:** Would write EOI to wrong register if struct used
**Fix:** Corrected struct layout to place eoi at 0xB0
**Result:** No change (still no IRQ fires, still shows POLLING)

---

## Current Image (All Fixes Applied)

**File:** `/var/www/rustux.com/html/rustica/rustica-live-amd64-0.1.0.img`
**SHA256:** `b56f6ab934d2c3527e2fd4908634befd7669b4e08657c0f2299c2f466d53d944`

**This image includes all 7 fixes listed above.**

---

## Technical Details: Current Implementation

### IOAPIC Configuration (runtime.rs)

```rust
const IRQ1_VECTOR: u32 = 0x41; // Vector 65

// Edge-triggered, active-high, unmasked (default IOAPIC config)
let low_dword = IRQ1_VECTOR;

// Route to actual BSP APIC ID
let lapic_id = (0xFEE00020 as *const u32).read_volatile() >> 24;
let high_dword = (lapic_id as u32) << 24;

// IOAPIC redirection entry
ioapic_sel.write_volatile(0x12);  // IRQ1 low dword offset
ioapic_win.write_volatile(low_dword);
ioapic_sel.write_volatile(0x13);  // IRQ1 high dword offset
ioapic_win.write_volatile(high_dword);
```

### IRQ Handler Stub (runtime.rs)

```rust
#[unsafe(naked)]
unsafe extern "C" fn keyboard_irq_stub() -> ! {
    core::arch::naked_asm!(
        // Save ALL registers FIRST
        "push rax", "push rbx", "push rcx", "push rdx",
        "push rsi", "push rdi", "push rbp",
        "push r8", "push r9", "push r10", "push r11",
        "push r12", "push r13", "push r14", "push r15",

        // Debug: VGA marker
        "mov rax, 0xB8000",
        "mov word ptr [rax], 0x4F21",

        // Call handler
        "call {handler}",

        // Restore registers
        "pop r15", "pop r14", "pop r13", "pop r12",
        "pop r11", "pop r10", "pop r9", "pop r8",
        "pop rbp", "pop rdi", "pop rsi",
        "pop rdx", "pop rcx", "pop rbx", "pop rax",

        // Send EOI to CORRECT address (0xB0, not 0x40!)
        "mov rax, 0xFEE000B0",
        "mov dword ptr [rax], 0",

        "iretq",
        handler = sym crate::keyboard::keyboard_irq_handler
    );
}
```

### sti Instruction (runtime.rs:684)

```rust
// --- 5️⃣ ENABLE CPU INTERRUPTS (CRITICAL!) ---
core::arch::asm!("sti", options(nostack, preserves_flags);
```

---

## What HAS Been Verified Working

1. ✅ **System boots to UEFI shell** - No crashes, no triple faults
2. ✅ **Framebuffer works** - Dracula theme displays correctly
3. ✅ **Shell is functional** - Built-in commands work (help, clear, echo, etc.)
4. ✅ **Polling fallback works** - Keyboard input via direct port 0x60/0x64
5. ✅ **PS/2 controller is alive** - Status port 0x64 shows data ready
6. ✅ **IOAPIC is configured** - Green line shows unmasked redirection entry
7. ✅ **LAPIC is enabled** - MSR 0x1B bit 11 set, SVR programmed
8. ✅ **IDT is loaded** - Vector 0x41 entry exists
9. ✅ **BSP APIC ID is read** - Blue dot confirms ID=0

---

## What is NOT Working

1. ❌ **IRQ1 never fires** - No '!' at VGA 0xB8000
2. ❌ **IRQ stub never executes** - No pixel at top-right corner
3. ❌ **INPUT_BUFFER never gets IRQ data** - read_char() always returns None
4. ❌ **System falls back to polling** - [POLLING] message appears

---

## Remaining Possible Causes

Given that all the standard fixes have been applied, the issue may be:

1. **Firmware-level IRQ routing issue** - UEFI firmware may be routing IRQ1 through a different path than standard IOAPIC
2. **ACPI interrupt override** - ACPI may be disabling the keyboard IRQ at firmware level
3. **Virtualization/Layer issue** - If running in a VM, the hypervisor may be filtering IRQ1
4. **Hardware incompatibility** - Some UEFI systems simply don't support PS/2 keyboard interrupts
5. **UEFI SimpleTextInput conflict** - Firmware may have the keyboard bound to UEFI console protocol

---

## Debug Files

- **[PASTE.md](rustica/docs/PASTE.md)** - Detailed debugging notes and code snippets
- **Git Repository** - https://github.com/gitrustux/rustux-efi

---

## Next Steps (Options)

### Option A: Switch to UEFI Simple Text Input
Use UEFI Simple Text Input Protocol instead of raw PS/2 interrupts. This bypasses the entire IOAPIC/LAPIC interrupt handling.

### Option B: Use Serial Console
Fall back to serial port console (COM1) for input/output, which UEFI typically supports better.

### Option C: Deeper Firmware Investigation
Read ACPI tables (MADT, FADT) to determine how firmware has configured interrupt routing.

### Option D: Alternative Keyboard Hardware
Test with USB keyboard (Phase 7A) to bypass PS/2 entirely.

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
│   └── docs/              # Documentation (BUILD.md, IMAGE.md, PLAN.md)
└── apps/                # CLI tools and GUI applications
```

---

## System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **Architecture** | x86_64 (AMD64) | x86_64 (AMD64) |
| **Boot** | UEFI 2.0 | UEFI 2.3+ |
| **RAM** | 512 MB | 1 GB |
| **Storage** | 128 MB (USB) | 4 GB |
| **Input** | PS/2 Keyboard (not working via IRQ) | PS/2 or USB HID* |

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

*Last Updated: January 24, 2025*
**Status:** Phase 6A-6C Complete | 6D Keyboard IRQ BLOCKED (polling fallback works, IRQs never fire)
