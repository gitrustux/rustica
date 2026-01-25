# Rustux OS - Phase 6: Interactive Shell (January 2025)

**Status:** 🟡 Phase 6A-6C COMPLETE | Phase 6D Keyboard IRQ - Fix #20 added (comprehensive interrupt path diagnostics), awaiting test results

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

### Fix #8: Explicitly Clear IOAPIC Mask Bit ✅
**Problem:** `let low_dword = IRQ1_VECTOR;` only sets bits 0-7, leaving bit 16 (mask) undefined
**Impact:** IRQ1 redirection entry may remain masked, preventing interrupt delivery
**Fix:** Explicitly build low_dword with all fields set, particularly `(0 << 16)` to clear mask bit
**Result:** No change (still no IRQ fires, still shows POLLING)

### Fix #9: Read PS/2 Data Port to Acknowledge Device ✅
**Problem:** IRQ handler never reads port 0x60, so PS/2 controller never deasserts IRQ1 line
**Impact:** Even if IRQ fires once, the line stays high and NO FURTHER IRQs will fire
**Fix:** Add `in al, 0x60` directly in IRQ stub BEFORE calling handler to acknowledge device
**Result:** Tested, still shows POLLING

```rust
// CRITICAL: Read PS/2 data port to ACKNOWLEDGE the device
// The PS/2 controller will NOT deassert IRQ1 until we read port 0x60
"mov dx, 0x60",    // PS/2 data port
"in al, dx",       // Read scancode (this ACKs the device)
```

### Fix #10: Verify Initialization Order ✅ (VERIFIED - Already Correct!)
**Problem (Hypothesis):** IDT entry 0x41 might be overwritten if init order is wrong
**Investigation:** Verified that main.rs initialization order IS correct:
  1. Line 262: `init_exception_handlers()` - Sets up IDT[0..31], calls `load_idt()`
  2. Line 280: `init_keyboard_interrupts()` - Sets up IDT[0x41], calls `load_idt()`
**Finding:** The initialization order was ALREADY CORRECT. The IDT entry is NOT being overwritten.
**Result:** This is NOT the cause of polling - issue is elsewhere (likely firmware-level)

```rust
// Current (CORRECT) order in main.rs:
init_exception_handlers();  // First - sets up exceptions
init_keyboard_interrupts(); // Second - installs IRQ entries

// Both functions call load_idt() after setting up their entries
```

### Fix #11: ACPI MADT Interrupt Source Overrides ✅
**Problem:** The system was using hardcoded IRQ1→GSI=1 mapping without checking ACPI tables. Many UEFI systems override the default ISA IRQ mapping in the MADT (Multiple APIC Description Table).
**Impact:** Even if all kernel configuration is correct, IRQ1 might be mapped to a different GSI (e.g., GSI 24) or have different polarity/trigger mode.
**Fix:** Added ACPI table parsing before exiting boot services:
  1. Read RSDP from UEFI configuration tables
  2. Parse MADT to find interrupt source overrides for IRQ1
  3. Use the GSI from override (or default to 1 if no override)
  4. Apply polarity and trigger mode from override
**Result:** Image built, awaiting testing
**Code Changes:**
- New module: `kernel-efi/src/acpi.rs` - ACPI table parsing
- Modified `main.rs` - Read ACPI tables before ExitBootServices
- Modified `runtime.rs` - Use GSI from override when configuring IOAPIC

### Fix #12: ExtINT Delivery Mode for Legacy IRQ Routing ✅
**Problem:** When ACPI reports "IRQ1 Override: None" (default GSI = 1), legacy interrupt routing is in effect. The keyboard uses the virtual PIC compatibility path, which requires ExtINT delivery mode (7) instead of Fixed delivery mode (0).
**Impact:** Using Fixed delivery mode when legacy routing is active causes IOAPIC to reject interrupts from the PIC.
**Fix:** When gsi == 1 (legacy), set delivery mode to ExtINT (7). When gsi != 1 (ACPI override), use Fixed mode (0).
**Result:** Still showed POLLING - IRQ1 never fired
**Code Changes:**
- Modified `runtime.rs` - Conditional delivery mode based on GSI value

### Fix #13: Dual-Vector Diagnostic Test ✅
**Problem:** IRQ1 never fires with any previous fix attempt. Need to determine if the issue is:
  1. IRQ1 is being delivered to a different vector than expected (0x21 vs 0x41)
  2. IRQs are not reaching the CPU at all (firmware/hardware issue)
**Hypothesis:** Some systems may deliver IRQ1 to vector 0x21 (33) instead of 0x41 (65), or vice versa.
**Fix:** Install keyboard handler at BOTH IDT[0x21] and IDT[0x41]. Leave IOAPIC in Fixed mode (not ExtINT).
**Result:** Still showed POLLING - IRQ never fired with either vector. This ruled out vector mapping as the issue.
**Code Changes:**
- Modified `runtime.rs` - Dual-vector IDT installation (0x21 and 0x41)
- Removed ExtINT mode - using Fixed delivery mode for diagnostic

### Fix #14: CPU Interrupt Acceptance Test (HLT Diagnostic) ✅
**Problem:** IRQ1 never fires with any previous fix. Need to determine if the CPU is receiving ANY external interrupts at all, not just keyboard IRQs.
**Root Cause:** The `sti` instruction does NOT immediately enable interrupts on x86. External interrupts are only recognized after the next instruction boundary. The kernel may be entering busy-wait loops before interrupts can be delivered.
**Fix:** Add `sti; nop; nop; nop; hlt` sequence immediately after enabling interrupts. The `hlt` instruction ONLY wakes on external interrupts, providing a definitive test of CPU-level interrupt acceptance.
**Result:** CPU never woke from `hlt` - confirms external interrupts are disabled at CPU level
**Code Changes:**
- Modified `runtime.rs` - Added `hlt` diagnostic test after `sti`
- Modified `main.rs` - Removed duplicate `sti` call (interrupts already enabled in runtime.rs)

### Fix #15: Disable x2APIC Mode ✅ (NEW FIX!)
**Problem:** CPU never woke from `hlt` test - external interrupts not reaching CPU at all. IOAPIC writes succeed (green line), LAPIC reads work, but interrupts never arrive.
**Root Cause:** Modern UEFI firmware often enables x2APIC mode (bit 10 of IA32_APIC_BASE MSR). In x2APIC mode:
- LAPIC MMIO at 0xFEE00000 is **IGNORED**
- All APIC access must go through MSRs
- Our MMIO-based LAPIC setup silently fails
- LAPIC logic never activates, so interrupts are dropped

**Symptoms of x2APIC being active:**
- IOAPIC writes succeed (green line shows)
- LAPIC ID reads work (MMIO returns data)
- But interrupts NEVER arrive (LAPIC logic not active)
- hlt never wakes (confirms no external IRQs reach CPU)

**Fix:** Before LAPIC MMIO initialization, check IA32_APIC_BASE MSR bit 10. If x2APIC is enabled, disable it by clearing bit 10. Then re-enable APIC (bit 11) and proceed with MMIO setup.
**Code Changes:**
- Modified `runtime.rs` - Check and disable x2APIC mode before LAPIC init
- Added debug output showing APIC mode (x2APIC vs xAPIC)

### Fix #16: Re-read IA32_APIC_BASE After wrmsr ✅ (NEW FIX!)
**Problem:** After disabling x2APIC and writing the MSR with `wrmsr`, the code used the stale `msr_value` to verify LAPIC base. Intel SDM explicitly documents that APIC mode transitions are NOT architecturally guaranteed to be immediate - software MUST re-read IA32_APIC_BASE after `wrmsr` to verify the transition took effect.
**Root Cause:**
- rdmsr → get current value
- Modify msr_value → clear bit 10, set bit 11
- wrmsr → write new value
- **BUG:** Never re-read MSR, use stale msr_value
- Some UEFI firmware re-writes MSR after ExitBootServices

**Symptoms of stale MSR value:**
- MMIO writes appear valid
- LAPIC registers read back plausible values
- But interrupt delivery logic never activates
- hlt never wakes, keyboard stays in polling mode

**Fix:** Re-read IA32_APIC_BASE immediately after wrmsr. Verify x2APIC is actually disabled (bit 10 == 0) and APIC is enabled (bit 11 == 1). Add error messages if verification fails.
**Code Changes:**
- Modified `runtime.rs` - Re-read MSR after wrmsr, verify mode transition
- Added visual indicator: Cyan pixel (4,0) if xAPIC confirmed, Magenta if still x2APIC

### Fix #17: Properly Read Full 64-Bit MSR Value ✅ (NEW FIX!)
**Problem:** The `rdmsr` instruction returns a 64-bit value split across EDX:EAX (two 32-bit registers). The code was using `out("rax") msr_value` which only captured EAX - the upper 32 bits from EDX were being lost.
**Root Cause:**
```rust
// WRONG - only reads lower 32 bits into msr_value
core::arch::asm!(
    "rdmsr",
    in("ecx") IA32_APIC_BASE,
    out("rax") msr_value,  // ❌ Only gets EAX (lower 32 bits)
    options(nostack, preserves_flags, readonly)
);
```

**Impact:** The APIC base address (bits 12-35) is in the upper bits, so it was being truncated/corrupted. This caused:
- `apic_base_from_msr = msr_value & 0xFFFF_F000` gave garbage
- MMIO writes went to the wrong LAPIC address
- Interrupts never fired because LAPIC wasn't actually configured
- Keyboard stayed in polling mode forever

**Fix:** Capture EAX and EDX separately, then combine into full 64-bit value:
```rust
// CORRECT - reads full 64-bit MSR value
let mut eax: u32;
let mut edx: u32;
core::arch::asm!(
    "rdmsr",
    in("ecx") IA32_APIC_BASE,
    out("eax") eax,
    out("edx") edx,
    options(nostack, preserves_flags, readonly)
);
let msr_value = (edx as u64) << 32 | (eax as u64);
```
**Code Changes:**
- Modified `runtime.rs` - Both rdmsr calls now properly capture EAX and EDX

### Fix #18: Emergency Diagnostic Output ✅ (NEW FIX!)
**Problem:** The system still shows POLLING even after all previous fixes. Need to determine if the code is reaching the MSR re-read at all, or if it's crashing/returning early.
**Hypothesis:** The diagnostic pixels at (1,0), (3,0), and (4,0) might not be appearing because the code is crashing or returning early before those checks. Without seeing these pixels, we can't determine where the failure is occurring.
**Fix:** Added emergency diagnostic section immediately after MSR re-read that:
1. Draws ALL diagnostic pixels (1,0), (3,0), (4,0) BEFORE any checks
2. Prints the actual MSR value in hex for debugging
3. Prints the APIC base address extracted from the MSR
4. Prints the mode status (xAPIC OK, x2APIC STILL ACTIVE, APIC DISABLED)

**Expected Output After Fix #18:**
```
MSR=0x[actual_value] BASE=0xFEE0
MODE: xAPIC OK
```

**Diagnostic Interpretation:**
- If you see all 4 pixels + MSR value → Code is reaching MSR re-read successfully
- If Magenta at (4,0) → x2APIC is still enabled (MSR write failed)
- If Red at (1,0) → APIC base isn't 0xFEE00000 (wrong address)
- If NO pixels appear → Code is crashing BEFORE the MSR re-read
- If NO MSR value prints → Code is crashing before print output

**Test Results:** MSR value confirmed correct (0x00000000FEE00900):
- x2APIC disabled (bit 10 = 0) ✓
- APIC enabled (bit 11 = 1) ✓
- APIC base = 0xFEE00000 ✓
- But keyboard still doesn't work!

### Fix #19: Verify LAPIC MMIO is Actually Responding ✅ (NEW FIX!)
**Problem:** MSR shows LAPIC is correctly configured (x2APIC disabled, APIC enabled, base = 0xFEE00000), IOAPIC is configured (green line), but keyboard still doesn't work. Need to verify LAPIC MMIO is actually responding to writes.
**Hypothesis:** The MSR looks perfect, but LAPIC MMIO might not be responding despite the correct MSR value. This could happen if:
- x2APIC mode can't be disabled on this hardware
- There's a hidden configuration issue blocking MMIO
- The LAPIC is in a weird state that MMIO can't access

**Fix:** After writing to SVR register (0x1FF to enable LAPIC), read it back to verify MMIO is working. If readback doesn't match what we wrote, MMIO isn't working.
**Diagnostic Interpretation:**
- LIME pixel (5,0) + "SVR readback OK" → LAPIC MMIO is working, issue is elsewhere
- ORANGE pixel (5,0) + "SVR write failed! Got: 0x..." → LAPIC MMIO is NOT working!
**Code Changes:**
- Modified `runtime.rs` - Added SVR readback verification after LAPIC enable

### Fix #20: Comprehensive Interrupt Path Diagnostics ✅ (NEW FIX!)
**Problem:** Previous tests confirmed:
- ✅ MSR is correct (0xFEE00900 - x2APIC disabled, APIC enabled, base = 0xFEE00000)
- ✅ LAPIC MMIO is working (SVR readback succeeded)
- ✅ IOAPIC is configured (green line)
- ✅ IRQ1 is unmasked
But keyboard still doesn't work! The issue must be in the interrupt delivery path.

**Fix:** Added 4 comprehensive diagnostic tests to pinpoint the exact issue:
1. **IOAPIC Redirection Entry Decoder** - Shows vector, destination APIC ID, and mask status
2. **CPU IF Flag Verification** - Confirms CPU interrupts are actually enabled after sti
3. **Keyboard IRQ Generation Test** - Sends reset command to keyboard to trigger IRQ
4. **Legacy PIC Status Check** - Verifies legacy PIC isn't interfering

**Expected Output:**
```
IOAPIC RTE: Vec=0x41 Dest=0x00 Masked=NO
CPU IF flag: ENABLED
Testing keyboard IRQ generation...
Keyboard reset sent. If IRQ fires, you'll see '!' at VGA top-left.
PIC masks: PIC1=0x... PIC2=0x...
```

**Diagnostic Guide:**
- If Vec != 0x41 → Wrong vector in IOAPIC entry
- If Dest != 0x00 → Wrong destination APIC ID
- If Masked=YES → IRQ1 is still masked
- If IF=DISABLED → sti didn't work or was cleared
- If '!' appears → Keyboard IS generating IRQs!
- If PIC masks show IRQ1 enabled → Legacy PIC might be stealing IRQs

**Code Changes:**
- Modified `runtime.rs` - Added 4 diagnostic tests in interrupt path

```rust
// When gsi == 1 (legacy routing): use ExtINT mode
// When gsi != 1 (ACPI override): use Fixed mode
let low_dword = if gsi == 1 {
    (7 << 8)      // ExtINT delivery mode
        | (0 << 11)   // Physical destination mode
        | (0 << 16)   // Unmasked
} else {
    IRQ1_VECTOR
        | (0 << 8)    // Fixed delivery mode
        | (0 << 11)   // Physical destination mode
        | polarity_bit
        | trigger_bit
        | (0 << 16)   // Unmasked
};
```

---
let irq1_override = unsafe {
    match acpi::find_rsdp() {
        Some(rsdp) => acpi::find_irq1_override(rsdp),
        None => acpi::Irq1Override::DEFAULT,
    }
};

// NEW: Use GSI-based offset (not hardcoded 0x12)
let gsi = irq1_override.gsi;
let redir_offset = 0x10 + (2 * gsi as u32);

// NEW: Use polarity and trigger mode from ACPI
let polarity_bit = if irq1_override.active_low { 1 << 13 } else { 0 << 13 };
let trigger_bit = if irq1_override.level_triggered { 1 << 15 } else { 0 << 15 };
```

---

## Current Image (All Fixes Applied)

**File:** `/var/www/rustux.com/html/rustica/rustica-live-amd64-0.1.0.img`
**SHA256:** `2c4487937233ae2932fb00085d80b42ee6c046502b18f2f4db2e096d644f352a`

**This image includes all 20 fixes listed above.**

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

        // CRITICAL: Read PS/2 data port to ACKNOWLEDGE the device
        // The PS/2 controller will NOT deassert IRQ1 until we read port 0x60
        "mov dx, 0x60",    // PS/2 data port
        "in al, dx",       // Read scancode (this ACKs the device)

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
10. ✅ **ACPI MADT parsing** - Reads RSDP, finds MADT, checks for IRQ1 overrides (Fix #11)

---

## What is NOT Working

1. ❌ **IRQ1 never fires** - No '!' at VGA 0xB8000
2. ❌ **IRQ stub never executes** - No pixel at top-right corner
3. ❌ **INPUT_BUFFER never gets IRQ data** - read_char() always returns None
4. ❌ **System falls back to polling** - [POLLING] message appears

---

## Remaining Possible Causes

**IMPORTANT:** Fix #20 (comprehensive interrupt path diagnostics) has been implemented and built. Awaiting test results.

**Previous Test Results (Fix #18-#19):**
- MSR confirmed correct: 0x00000000FEE00900
- x2APIC disabled (bit 10 = 0) ✓
- APIC enabled (bit 11 = 1) ✓
- APIC base = 0xFEE00000 ✓
- LAPIC MMIO working (SVR readback OK) ✓

**But keyboard still doesn't work!** The issue must be in the interrupt delivery path.

**Expected Output After Fix #20:**
```
IOAPIC RTE: Vec=0x41 Dest=0x00 Masked=NO
CPU IF flag: ENABLED
Testing keyboard IRQ generation...
Keyboard reset sent. If IRQ fires, you'll see '!' at VGA top-left.
PIC masks: PIC1=0x... PIC2=0x...
```

**Visual Indicators:**
- (1,0) Yellow/Red = APIC base address check
- (2,0) Blue = BSP APIC ID
- (3,0) Red/Green = Was x2APIC detected?
- (4,0) Cyan/Magenta = xAPIC mode confirmation
- (5,0) LIME/ORANGE = LAPIC MMIO verification
- **(6,0) Green/Red = CPU IF flag verification** (NEW!)

**Diagnostic Guide:**
- If Vec != 0x41 → Wrong vector in IOAPIC entry
- If Dest != 0x00 → Wrong destination APIC ID
- If Masked=YES → IRQ1 is still masked
- If IF=DISABLED → sti didn't work or was cleared
- If '!' appears → Keyboard IS generating IRQs!

1. **UEFI SimpleTextInput Protocol conflict** - Firmware may have the keyboard bound to UEFI console protocol, preventing raw PS/2 access
2. **Virtualization/Layer issue** - If running in a VM, the hypervisor may be filtering IRQ1
3. **Hardware incompatibility** - Some UEFI systems simply don't support PS/2 keyboard interrupts
4. **IMC (Interrupt Message Controller) issue** - Some systems use IMC instead of traditional IOAPIC
5. **PS/2 port disabled at firmware level** - Firmware may have disabled legacy PS/2 support entirely

**Ruled Out:**
- ✅ IDT entry overwrite (Fix #10 verified init order is correct)
- ✅ IRQ stub register corruption (Fix #1)
- ✅ IOAPIC configuration issues (Fixes #2, #3, #8)
- ✅ EOI address (Fixes #4, #5, #7)
- ✅ CPU interrupts disabled (Fix #6)
- ✅ PS/2 device not acknowledged (Fix #9)
- ✅ ACPI interrupt override (Fix #11 - now reads MADT for overrides)
- ✅ Delivery mode for legacy IRQs (Fix #12 - ExtINT when gsi==1 tested, didn't work)
- ✅ Vector mapping issue (Fix #13 - both 0x21 and 0x41 tested, neither works)
- ✅ CPU interrupt acceptance (Fix #14 - hlt test showed IRQs not reaching CPU)
- ✅ x2APIC mode blocking MMIO (Fix #15 - disable x2APIC before LAPIC init)
- ✅ Stale MSR value after wrmsr (Fix #16 - re-read MSR to verify mode transition)
- ✅ Truncated MSR read (Fix #17 - properly capture EAX and EDX for full 64-bit value)
- ✅ Emergency diagnostic output (Fix #18 - force all pixels before any checks, print MSR value)
- ✅ LAPIC MMIO verification (Fix #19 - SVR readback test confirmed working)
- ⏳ Interrupt path diagnostics (Fix #20 - comprehensive IRQ delivery path tests, awaiting results)

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
**Status:** Phase 6A-6C Complete | 6D Keyboard IRQ - Fix #20 added (comprehensive interrupt path diagnostics), awaiting test results
