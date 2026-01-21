# Rustica OS - Development Plan

**Last Updated:** 2025-01-20 - UEFI/QEMU Environment Fully Resolved ✅
**Current Focus:** Phase 4A - Userspace Execution
**Strategy:** Heap allocator fixes completed, validation and parallel work
**Kernel Location:** `/var/www/rustux.com/prod/rustux/`

---

## ✅ UEFI/QEMU Environment - RESOLVED (2025-01-20)

**Issue:** System's OVMF firmware files were corrupted (all zeros), QEMU 8.2.2 had compatibility issues

**Resolution:**
- Purged and reinstalled `ovf` package
- Fixed OVMF firmware corruption - files now have valid _FVH signatures
- Found working QEMU configuration: `-machine pc,accel=tcg` (not q35)
- Added `-machine acpi=off,hpet=off` to avoid QEMU mutex crashes

**Verified Working Configuration:**
```bash
qemu-system-x86_64 \
  -bios /usr/share/ovmf/OVMF.fd \
  -drive file=image.img,format=raw \
  -nographic \
  -m 512M \
  -machine type=pc,accel=tcg \
  -machine acpi=off,hpet=off
```

**Architecture-Specific Firmware Requirements:**
| Architecture | Firmware | Notes |
|--------------|----------|-------|
| **amd64** | `/usr/share/ovmf/OVMF.fd` | EDK2 UEFI firmware (verified working) |
| **arm64** | EDK2 AArch64 firmware | Different package: `edk2-arm64` or QEMU's `-bios` parameter |
| **riscv64** | OpenSBI | Uses OpenSBI as firmware, not UEFI |

**Key Learnings:**
- UEFI loader is correct ✅
- Boot path is real, not emulator artifact ✅
- Previous crashes were environmental, not kernel logic ✅
- Nothing UEFI-related needs to be revisited for amd64 ✅

---

## 🔄 Two-Phase PMM Strategy

### Architectural Context

**Problem:** Original bitmap PMM had bugs preventing multi-page allocation.

**Proper kernel architecture progression:**
1. **Boot PMM** - Simple, dumb, reliable (first thing after UEFI)
2. **Early Kernel PMM** - Vec-based, linear scan (✅ COMPLETE - where we are now)
3. **Final PMM** - Bitmap / buddy allocator (optimization, not foundation)

---

## ✅ Phase A: Vec-Based PMM - COMPLETE (2025-01-19)

**Status:** Vec-based PMM implementation complete and working

**Completed (2025-01-19 session):**
- ✅ Backed up bitmap PMM to `src/mm/pmm_bitmap.rs.bak`
- ✅ Replaced `src/mm/pmm.rs` with Vec-based implementation (~650 lines)
- ✅ Page array with state enum: `Free | Allocated | Reserved`
- ✅ Linear scan allocation (O(N) where N = total pages)
- ✅ Added `pmm_reserve_pages()` for heap reservation
- ✅ Increased boot allocator buffer to 2MB
- ✅ Fixed userspace test execution order (after PMM init)
- ✅ PMM reports **32,256 free pages** (126MB of memory)

**Test Output (UEFI mode):**
```
[INIT] PMM init complete, free pages: 7E00 (32,256 decimal)
[KERNEL] Heap test passed
```

**Remaining Issues:**
- ⚠️ Heap allocator hangs on `Vec::push()` and `Vec::with_capacity()`
- ⚠️ This is a separate subsystem issue from PMM

**Original bitmap PMM saved to:** `src/mm/pmm_bitmap.rs.bak` (for Phase B)

---

## 🔵 Phase B: Reintroduce Bitmap PMM (LATER)

**Trigger:** Userspace works and is stable

**Action Items:**
1. Reintroduce bitmap PMM as `pmm_v2`
2. Validate against known-good Vec PMM
3. Add unit tests for allocation patterns
4. Add randomized allocation/free stress tests
5. Feature flag: `#[cfg(feature = "pmm_bitmap")]`

**Purpose:** Optimization, not foundation

**Why This Avoids Regressions:**
- Vec PMM becomes the "known-good" baseline
- Bitmap PMM validated against it
- Can switch between implementations for testing
- No risk of breaking working userspace

**DO NOT return to bitmap debugging until Vec PMM works.**

---

## 🟡 Heap Allocator - Fixes Implemented, Validation Pending (2025-01-20)

**Status:** Critical bugs fixed, validation tests needed

**Issues Fixed:**
1. **Block Splitting Bug** ✅ FIXED:
   - Original block's size was not being updated after split
   - Fix: `(*current).size = offset + size` (only when split confirmed)
   - This prevents allocator from re-finding the same block

2. **Broken LAST_ALLOCATED Check** ✅ REMOVED:
   - Check was too aggressive, preventing valid allocations
   - `alloc_size` used block size (16MB) instead of allocation size
   - Caused false "overlap" detection

3. **Kernel Stack Switch** ✅ COMPLETE:
   - 256KB (64 pages) from PMM kernel zone
   - Non-returning jump to continuation
   - Prevents stack overflow on UEFI's 4-8KB firmware stack

**Remaining Work:**
- **Validation**: Run ELF loader multiple times to confirm no vaddr corruption
- **Free-Block Coalescing** (optional but recommended): Deallocation may not properly merge adjacent free blocks
- **Allocator Invariants** (recommended):
  ```rust
  assert!(block.size >= HEADER_SIZE);
  assert!(!overlapping_blocks());
  ```

### Validation Steps

1. Allocate/free in a loop (stress test)
2. Run ELF loader multiple times without reboot
3. Confirm vaddr values are correct (not 0x300028 corruption)

**Current Commit:** `4668ecc` - "Fix heap allocator block splitting and remove broken LAST_ALLOCATED check"

---

## 🔴 PMM Single-Page Bug (RESOLVED - Superseded by Vec PMM)

**Status:** Superseded by two-phase strategy

**Original Issue:** Bitmap allocator only allocates 1 page before failing.

**Resolution:** Replaced with Vec-based allocator (Phase A complete).

**Root Cause:** Jumped to complex allocator without proving simpler approach works.

---

## ✅ COMPLETED: Heap & VMO Corruption Fixes (2025-01-19)

**Summary:** Fixed two major corruption issues that were blocking userspace execution.

### Issue #1: VMO Corruption via Vec Moves ✅ FIXED
**Root Cause:** `Vmo` stored by value in `Vec<LoadedSegment>` caused VMO objects to move when Vec reallocated, corrupting interior pointers.

**Fix:** Changed `pub vmo: Vmo` to `pub vmo: Box<Vmo>` for stable heap addresses.

**Files Modified:** `src/exec/elf.rs`

### Issue #2: ProgramHeader Corruption via Heap Allocations ✅ FIXED
**Root Cause:** During ELF segment loading, VMO operations triggered heap allocations that corrupted the `load_segments` Vec, causing invalid values during segment creation.

**Fix:** Convert Vec to array before any VMO operations, and copy all ProgramHeader fields to local variables before heap allocations.

**Files Modified:** `src/exec/elf.rs`
- Changed `ProgramHeader` to derive `Copy`
- Added array conversion before segment loading loop
- Copy all fields (`p_offset`, `p_filesz`, `p_memsz`, `p_vaddr`, `p_flags`) before VMO operations

**Test Results (After Fixes):**
```
[ELF] Loading segment 0: offset=0x0 filesz=0x130 memsz=0x130 ✅
[ELF] Loading segment 1: offset=0x1000 filesz=0x62 memsz=0x62 ✅
[ELF] Loading segment 2: offset=0x1100 filesz=0x1c memsz=0x1c ✅
[VMO-WRITE] VMO#0 pages=1 paddr=0x1000000 ✅
[VMO-WRITE] VMO#1 pages=1 paddr=0x1001000 ✅
[VMO-WRITE] VMO#2 pages=1 paddr=0x1002000 ✅
[MAP] Checking page_entry
[MAP] Entry is None
```

**Result:** All 3 ELF segments now load correctly without corruption. Kernel progresses to address space mapping.

---

## ✅ RESOLVED: VMO#0 len=0 - Stack Corruption (2025-01-19)

### Root Cause: Stack Corruption from Debug Output

**Symptom:** `VMO#0 showed len=0 (empty pages map) during address space mapping`

**Investigation revealed:**

| Initially Suspected | Actual Cause |
|-------------------|--------------|
| Heap corruption | ❌ Not - Box<Vmo> was already stable |
| VMO identity mismatch | ❌ Not - Same pointers throughout |
| PMM allocation failure | ❌ Not - Pages allocated correctly |
| **Stack corruption** | ✅ **YES** - Debug output caused struct overlap |

### The Smoking Gun: `vmo_id=524`

Adding SANITY checks immediately after each `segments.push()` revealed:

```
SANITY seg0 vmo_id=524  (524 = p_filesz!)
SANITY seg1 vmo_id=2    ✅
SANITY seg2 vmo_id=3    ✅
```

**Pattern:** `vmo_id=524` exactly equals `p_filesz=0x20C (524 decimal)`

**Diagnosis:** Struct overlap - VMO struct on stack was being overwritten by ProgramHeader data.

### Fixes Applied

#### Fix #1: Moved `Box::new(vmo)` Before VMO Operations

**Before (vulnerable to stack corruption):**
```rust
// src/exec/elf.rs
let vmo = Vmo::create(aligned_size as usize, vmo_flags)?;
// Write happens while vmo is still on stack
vmo.write(0, segment_data)?;
let boxed_vmo = Box::new(vmo);  // Boxed AFTER operations
```

**After (protected):**
```rust
let vmo = Vmo::create(aligned_size as usize, vmo_flags)?;
// CRITICAL: Immediately box the VMO before any operations
let boxed_vmo = Box::new(vmo);
// Now VMO is on heap, safe from stack corruption
boxed_vmo.write(0, segment_data)?;
```

#### Fix #2: Reduced BSS Stack Allocation

**Before (4KB stack array):**
```rust
let mut bss_data = [0u8; 4096];  // Large stack allocation
```

**After (256B chunked):**
```rust
let zero_chunk = [0u8; 256];  // Much smaller, loop for large BSS
```

#### Fix #3: Pre-allocated Vec Capacity

```rust
let mut segments = Vec::with_capacity(segment_count);  // Prevents reallocation
```

### Investigation Techniques Used

1. **Pointer identity logging** - Proved VMO instances were identical (not a clone issue)
2. **SANITY checks after push** - Revealed `vmo_id=524` pattern
3. **BEFORE RETURN checks** - Showed debug output was causing corruption
4. **Address logging** - Differentiated struct overwrite from heap corruption

### Key Insight: Clean Zeros Indicate Struct Rebuild

The pattern of seeing `id=0, pages=0` (clean defaults) rather than random corrupted values indicated:
- Struct overwrite/rebuilding, NOT heap allocator corruption
- The struct was being reconstructed or overlapped with other data

### Current Status

- ✅ SANITY checks show correct VMO IDs (1, 2, 3)
- ⚠️ Verbose debug output still causes some stack corruption
- ✅ Root cause identified: Stack corruption from debug output
- ⏳ Next: Apply minimal-debug strategy

## 🎯 Next Steps: Minimal-Debug Strategy (2025-01-19)

### The Correct Minimal-Debug Strategy

The investigation has proven that the debug output itself is causing corruption. Apply this strategy exactly.

#### 🚫 What to Remove

Inside `load_elf` and related code:

- ❌ Any per-segment `debug!`, `println!`, or `log!`
- ❌ Any formatted output using `{}` or `{:?}`
- ❌ Any debug inside SpinMutex locks
- ❌ Any debug that allocates buffers (strings, Vecs)

#### ✅ What to Keep (Safe)

**Single invariant check:**
```rust
for (i, seg) in segments.iter().enumerate() {
    if seg.vmo.id() == 0 {
        panic!("load_elf invariant violated: seg {} has null VMO", i);
    }
}
```

**One single debug print:**
- Outside loops
- Outside locks
- After all allocations are complete
- Prefer a fixed string + integers only

**Example:**
```rust
debug_raw("load_elf completed\n");
```
(No formatting if possible)

### Longer-Term Fixes (After Userspace Boots)

Once unblocked, implement these real fixes:

#### 🛡️ 1. Increase Kernel Stack Size
Current 4 KB stack is tiny. Increase to:
- 16 KB minimum
- 32 KB if possible

#### 🛡️ 2. Add Stack Guard Page
Even a single unmapped page would have caught this instantly.

#### 🛡️ 3. Replace Debug Printing
Implement proper debug infrastructure:
- Ring buffer
- Fixed-size per-CPU buffer
- Post-boot dump

#### 🛡️ 4. Ban Formatted Debug in Early Boot
**Rule:** No formatting before scheduler + full VM is up.

### Big Picture Assessment

**You are past the hard part.**

✅ ELF loader is fundamentally correct
✅ VMO ownership is correct
✅ Memory zoning works
✅ Heap is usable

**The remaining failure is tooling, not logic.**

This is exactly where real kernels hit the "debugger becomes the bug" phase.

### Files Modified

- `src/exec/elf.rs` - Box::new timing, BSS allocation, SANITY checks
- `src/exec/process_loader.rs` - Address logging after return
- `src/object/vmo.rs` - Pointer identity logging
- `src/process/address_space.rs` - Pointer identity logging

---

## 📋 Phase 4: Userspace & Process Execution

### 4A. ELF Loading & Address Space ⏳ IN PROGRESS (2025-01-20)

**Current Status:**
- ⚠️ **BLOCKER:** LoadedSegment corruption during ELF loading
- ✅ All 3 ELF segments create VMOs successfully (allocated at 0x1000000, 0x1001000, 0x1002000)
- ✅ VMO write operations complete
- ⚠️ Address space creation starts but fails during segment mapping

**Completed (2025-01-20 session):**
- ✅ Removed verbose debug output from address_space.rs (minimal-debug strategy)
- ✅ Fixed VMO clone stack overflow (replaced 4KB buffer with 256-byte chunks)
- ✅ Skipped VMO cloning in map_vmo to avoid corruption
- ✅ Identified root cause: **Stack overflow during deep call chains**

### 🔴 Stack Overflow Investigation (2025-01-20)

**Evidence Pattern:**

Multiple independent signals pointing to stack overflow:

1. **Corruption Location:** During ELF loading (deep call chains)
   - `process_loader → load_elf → parse_phdrs → create_vmo → pmm_alloc → heap_alloc → BTreeMap_insert`

2. **Corruption Pattern:** `vaddr` (offset 0) → overwritten with `flags` (offset 24)
   - Value 0x3 = PF_R | PF_W flags exactly
   - Deterministic field overwrite at consistent offset
   - Same struct fields, same offsets every time

3. **Stack Pressure Reduction Reduced Corruption:**
   - 4KB stack buffer → corruption
   - 256B chunked buffer → less corruption
   - Debug output (stack buffers + asm) → corruption
   - Removing debug output → reduced but not eliminated

4. **Not Heap Allocator Bug (evidence against):**
   - Heap bugs typically show: double free, bad coalescing, use-after-free, random corruption
   - We're seeing: deterministic field overwrite, same offsets, happens during deep operations
   - This pattern is **textbook early-kernel stack overflow**

**Test Results with Increased Stack Size:**

| Stack Size | Corruption Pattern | Analysis |
|-----------|-------------------|----------|
| 64KB (0x10000) | vaddr=0x3 | Flags value (PF_R \| PF_W) |
| 128KB (0x20000) | vaddr=0x300028 | Heap address (0x300000) + offset |
| +Box<LoadedElf> | vaddr=0x300028 | Heap corruption continues |

**Key Finding:** The changing corruption pattern suggests the stack size increase helps but **doesn't fully resolve the issue**. The linker flag approach (`-C link-arg=-stack:0x20000`) may not be reliably honored by UEFI firmware.

**Root Cause:** Stack overflow during deep call chains between process_loader, VMO operations, and heap allocations. The LoadedSegment struct on the stack gets overwritten by adjacent data.

**Recommended Execution Order:**

#### Step 1: Proper Kernel Stack Switch (RECOMMENDED) ✅ NEXT STEP
- **Problem:** Linker flag approach for UEFI stack size is unreliable
- **Solution:** Implement actual stack switch to dynamically allocated kernel stack
- **Why:** Eliminates dependency on UEFI firmware stack size
- **Implementation:**
  1. Allocate kernel stack pages (already done in `init_kernel_stack()`)
  2. Add assembly function to switch RSP to new stack
  3. Call early in boot before deep call chains

#### Step 2: Minimal Userspace Test - ELF Bypass (ALTERNATIVE)
- Hardcode: One VMO, one page, one mapping
- Jump to stub userspace loop
- Provides: Proof that VMOs + address space + context switch work
- Isolates: From ELF complexity, gives known-good baseline
- **Use this if:** Stack switch implementation is blocked

#### Step 3: Heap Allocator Investigation (ONLY IF NEEDED)
- If corruption persists after proper stack switch → then audit heap
- Add: Redzones, canaries, instrument alloc/free paths
- Do NOT do this until stack overflow is eliminated

**Why This Approach:**
- Eliminates primary suspect (stack overflow) before complex surgery
- Builds on stable foundation instead of debugging on unstable ground
- Mirrors how real kernels evolve (Linux did exactly this)
- Avoids weeks of unnecessary pain if stack was the issue all along

**Key Files to Modify:**
- `src/arch/amd64/init.rs` - Add stack switch assembly function
- `src/init.rs` - Call stack switch early in boot
- `src/exec/userspace_exec_test.rs` - Alternative: Add minimal test variant

---

### Previous Status (2025-01-19) - DEPRECATED

**Remaining Tasks (OBSOLETE - superseded by stack overflow fix):**
1. ⏳ Remove verbose debug output to eliminate residual corruption
2. ⏳ Complete segment mapping
3. ⏳ Map user stack
4. ⏳ Execute userspace transition via IRETQ
5. ⏳ Verify "Hello from userspace!" output

**Completed (2025-01-19):**
- ✅ ELF loader fully implemented and tested
- ✅ Address space framework complete with page table management
- ✅ Process loader ties ELF loading with address space creation
- ✅ Userspace entry point via IRETQ implemented
- ✅ Test infrastructure ready
- ✅ VMO stack corruption fixed (Box::new timing, reduced BSS allocation)

**Key Files:**
- `src/exec/elf.rs` - ELF parser (490 lines)
- `src/process/address_space.rs` - Address space management
- `src/exec/process_loader.rs` - Process loading
- `src/arch/amd64/uspace.rs` - Userspace transition

---

### 4B. Essential Syscalls ⏳ PENDING

**Priority:** 🔴 HIGH - Userspace needs syscalls for I/O

**Minimum Viable Set (5 syscalls):**
| Syscall | Purpose | Status |
|---------|---------|--------|
| `sys_exit` | Process termination | ⏳ TODO |
| `sys_write` | Console output | ⏳ TODO |
| `sys_read` | Console input | ⏳ TODO |
| `sys_mmap` | Memory allocation | ⏳ TODO |
| `sys_clock_gettime` | Time queries | ✅ Working |

**Location:** `src/syscall/definitions.rs`

**Tasks:**
1. Implement syscall handlers in `src/arch/amd64/syscall.rs`
2. Add syscall numbers to definitions
3. Wire up handlers in IDT
4. Test from userspace program

---

### 4C. Scheduler Start ⏳ PENDING

**Priority:** 🟠 HIGH - Needed for multi-process

**Tasks:**
1. Create init process (PID 1)
2. Load `/sbin/init` ELF
3. Add to run queue
4. Enable timer preemption
5. Implement context switch

**Location:** `src/sched/` (framework exists)

---

### 4D. Initial Ramdisk ⏳ PENDING

**Priority:** 🟠 HIGH - Needed to load programs

**Design:**
- Simple tar-like format for file storage
- Files embedded in kernel binary
- Basic file operations: open, close, read, stat

**Files to include:**
- `/sbin/init` - Init process
- `/bin/sh` - Shell
- `/bin/ls`, `/bin/cat`, `/bin/echo` - Core utilities

---

### 4E. Console Driver ⏳ LOW PRIORITY

**Priority:** 🟡 LOW - Debug port (0xE9) works for now

**Options:**
- VGA text mode (0xB8000)
- Serial console (COM1: 0x3F8)

**Defer until:** Syscalls and scheduler work

---

### 4F. Bootable Image ⏳ LOW PRIORITY

**Priority:** 🟢 LOW - Packaging comes LAST

**DO NOT create bootable images until:**
- ✅ ELF loading works
- ✅ Syscalls work
- ✅ Process execution works
- ✅ At least one userspace program runs

---

## 📁 Quick Reference

### Kernel Structure
```
/var/www/rustux.com/prod/rustux/
├── src/
│   ├── main.rs              # Entry point
│   ├── exec/                # ELF loader, process loader
│   │   ├── elf.rs           # ✅ ELF parsing/loading
│   │   ├── process_loader.rs # ✅ Process creation
│   │   └── userspace_exec_test.rs # ✅ Test framework
│   ├── process/             # Process management
│   │   └── address_space.rs # ✅ Address space with page tables
│   ├── arch/amd64/
│   │   ├── uspace.rs        # ✅ Userspace transition (IRETQ)
│   │   └── mexec.rs         # ✅ Alternative mexec implementation
│   ├── object/              # VMOs, handles, capabilities
│   ├── mm/
│   │   ├── pmm.rs           # ⚠️ BUG: Single-page allocation
│   │   └── allocator.rs     # ✅ Heap allocator
│   └── syscall/             # Syscall interface
└── test-userspace/
    └── hello.elf           # ✅ Test binary (9KB)
```

### Build Commands
```bash
# Build kernel
cd /var/www/rustux.com/prod/rustux
cargo build --release --target x86_64-unknown-uefi --features uefi_kernel

# Test in QEMU
./test-qemu.sh

# Build userspace test
cd test-userspace
./build.sh
```

---

## 🎯 Success Criteria

### Phase 4A Complete When:
- [x] VMO identity issue resolved ✅ (2025-01-19)
- [ ] Verbose debug output removed
- [ ] Address space creation works
- [ ] ELF segments map into address space
- [ ] Userspace transition completes
- [ ] "Hello from userspace!" appears on debug console

### Phase 4B Complete When:
- [ ] sys_write outputs to debug console
- [ ] sys_exit terminates process cleanly
- [ ] sys_mmap allocates memory
- [ ] Can call syscalls from userspace

### Phase 4 Complete When:
- [ ] Init process (PID 1) runs
- [ ] Can execute programs from initrd
- [ ] Shell runs interactively
- [ ] Multiple processes can run

---

## 📊 Completed Work Summary

### Phase 2C: Kernel Migration ✅ COMPLETE
- ~13,500 lines of kernel code migrated to new architecture
- All core modules implemented (AMD64, MM, objects, sync, syscalls)
- Kernel boots successfully in QEMU with UEFI

### Phase 4A (2025-01-18):
- ELF loader fully implemented and tested
- Address space framework complete with page table management
- Process loader ties ELF loading with address space creation
- Userspace entry point via IRETQ implemented
- Test infrastructure ready

**Key Files Created/Modified:**
- `src/exec/elf.rs` - ELF parser (490 lines)
- `src/process/address_space.rs` - Address space management
- `src/exec/process_loader.rs` - Process loading
- `src/arch/amd64/uspace.rs` - Userspace transition
- `src/init.rs` - Updated with boot allocator for PMM

---

## 📞 Resources

- **Repository:** https://github.com/gitrustux/rustux
- **Userspace CLI:** `/var/www/rustux.com/prod/rustica/tools/cli/` (~5,150 lines, complete)
- **Documentation:** See ARCHITECTURE.md for kernel design details
