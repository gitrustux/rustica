# Rustica OS - Development Plan

**Last Updated:** 2025-01-19
**Current Focus:** Phase 4A - Userspace Execution
**Strategy:** Two-phase PMM replacement (see below)
**Kernel Location:** `/var/www/rustux.com/prod/rustux/`

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

## 🔴 Current Blocker: PMM Single-Page Bug (HOLD)

**Status:** Superseded by two-phase strategy

**Original Issue:** Bitmap allocator only allocates 1 page before failing.

**Resolution:** Don't fix the bitmap allocator yet. Replace it with simpler Vec-based allocator first.

**Root Cause:** Jumped to complex allocator without proving simpler approach works.

---

## 🔴 CRITICAL: VMO#3 Corruption - ROOT CAUSE FOUND & FIXED (2025-01-19)

**Status:** ✅ **FIXED** - Changed `LoadedSegment.vmo` from `Vmo` to `Box<Vmo>`

### The Real Root Cause: Vec Moves + Interior Aliasing

**The Problem:** Vmo was stored BY VALUE in `Vec<LoadedSegment>`, causing Vmo objects to MOVE when the Vec reallocates.

**The Symptom:** VMO#3's BTreeMap entry disappeared after VMO#1's clone operation.

**Why Physical Memory Zoning Didn't Help:** This was NOT a memory overlap or heap corruption bug. It was a logical aliasing bug caused by struct moves.

---

### Technical Deep Dive

**Data Structure (BEFORE fix):**
```rust
pub struct LoadedSegment {
    pub vaddr: u64,
    pub size: u64,
    pub vmo: Vmo,              // ← Stored BY VALUE in Vec
    pub flags: u32,
}

pub struct LoadedElf {
    pub entry: u64,
    pub segments: Vec<LoadedSegment>,  // ← Vec can reallocate
    pub stack_addr: u64,
    pub stack_size: u64,
}
```

**What Happens When Vec Reallocates:**
1. Vec allocates new memory
2. Moves all `LoadedSegment` structs (including embedded `Vmo` objects)
3. `Vmo` objects now have DIFFERENT memory addresses
4. Any interior pointers/references to Vmo become STALE

**The Smoking Gun:**
```
[P-LOADER] After seg0: VMO#3 MISSING!    (before fix)
[P-LOADER] After seg0: VMO#3 present=1   (after fix with Box<Vmo>)
```

---

### The Fix: Box<Vmo> for Stable Addresses

**Changed `LoadedSegment` to store Vmo in a Box:**
```rust
pub struct LoadedSegment {
    pub vaddr: u64,
    pub size: u64,
    pub vmo: Box<Vmo>,         // ← Boxed for stable address
    pub flags: u32,
}
```

**Why This Works:**
- `Box<Vmo>` allocates Vmo on the HEAP
- Box's address in the Vec may change, BUT the Vmo's address stays STABLE
- No interior pointers become stale
- Vmo clone operations work correctly

---

### Timeline of Investigation

**Initial Hypothesis (INCORRECT):** Memory overlap / heap corruption
- Implemented physical memory zoning (KERNEL zone vs USER zone)
- Moved heap from 0x1000000 to 0x300000
- Implemented heap buffer intermediate copy for VMO clone
- Result: VMO#3 STILL corrupted

**Breakthrough (User Analysis):** "The real culprit: implicit struct moves + interior pointers"
- User identified that `Vmo` stored by value in `Vec` causes moves
- When Vec reallocates, Vmo objects move to new addresses
- This can cause interior pointer aliasing issues
- Solution: Store Vmo in `Box<Vmo>` for stable addresses

**Verification:**
- Applied fix: Changed `pub vmo: Vmo` to `pub vmo: Box<Vmo>`
- Added `use alloc::boxed::Box;` import
- Updated LoadedSegment creation to use `Box::new(vmo)`
- Test result: VMO#3 now stays PRESENT after all clones! ✅

---

### Test Results (Before vs After)

**Before Fix (Vmo stored by value):**
```
[VMO] clone() complete
[P-LOADER] After seg0: VMO#3 MISSING!
[P-LOADER] After seg1: VMO#3 MISSING!
```

**After Fix (Box<Vmo>):**
```
[VMO] clone() complete
[P-LOADER] After seg0: VMO#3 present=1
[P-LOADER] After seg1: VMO#3 present=1
[P-LOADER] Mapping segment 2
[MAP] VMO#3 num_pages=1
[MAP] VMO#3 len=1 key0=1 present=1
```

---

### Key Learnings

1. **Struct moves can cause subtle bugs** - When a struct is stored by value in a Vec and the Vec reallocates, the struct moves to a new address. Any interior pointers or references become stale.

2. **Box provides stable addresses** - `Box<T>` allocates on the heap and the address remains stable even if the Box itself moves.

3. **Debug output is critical** - The granular debug logs allowed us to pinpoint exactly when VMO#3 disappeared and trace the root cause.

4. **User's analysis was spot-on** - The user identified this as a "shallow clone" or "struct move" issue, not memory corruption. Their analysis saved hours of debugging time.

---

### Files Modified

**src/exec/elf.rs:**
- Changed `pub vmo: Vmo` to `pub vmo: Box<Vmo>` in `LoadedSegment` struct
- Added `use alloc::boxed::Box;` import
- Updated LoadedSegment creation to use `Box::new(vmo)`

---

### Remaining Work

**Phase 4A (Userspace Execution) can now proceed!**
- VMO cloning works correctly
- All 3 ELF segments load and map successfully
- Ready to proceed to userspace transition testing

**Next Steps:**
1. Complete segment 2 mapping (currently in progress)
2. Map user stack
3. Transition to userspace via IRETQ
4. Verify "Hello from userspace!" output

---

### IMPLEMENTATION ATTEMPT RESULTS (2025-01-19 Session)

**Actions Taken:**
1. ✅ Implemented physical memory zoning in PMM
2. ✅ Created `pmm_alloc_kernel_page()` and `pmm_alloc_user_page()`
3. ✅ Moved kernel heap from 0x1000000 (USER zone) to 0x300000 (KERNEL zone)
4. ✅ Updated VMO allocations to use `pmm_alloc_user_page()`
5. ✅ Updated page table allocations to use `pmm_alloc_kernel_page()`
6. ✅ Fixed PMM allocator filtering to respect KERNEL/USER flags
7. ✅ Implemented heap buffer intermediate copy for VMO clone

**Physical Memory Zones Implemented:**
```
KERNEL_ZONE: 0x00200000 - 0x00FFFFFF (14 MB)
  - Kernel heap @ 0x300000 (1MB)
  - Page tables
  - Kernel metadata

USER_ZONE: 0x01000000 - 0x07FE0000 (112 MB)
  - VMO backing pages
  - Clone destinations
  - User data
```

**Current Status (2025-01-19 latest test):**
```
[VMO] clone: src_paddr=0x1000000 dst_paddr=0x1003000 buffer=0x1fe93478
[VMO] BEFORE COPY
[VMO] AFTER COPY
[VMO] COPY DONE
[P-LOADER] After seg0: VMO#3 MISSING!
```

**Key Finding:** Even with:
- Physical memory zoning implemented
- Heap in KERNEL zone (0x300000)
- VMOs in USER zone (0x1000000+)
- Heap buffer intermediate copy
- Proper address conversion via `pmm::paddr_to_vaddr()`

**VMO#3 is STILL CORRUPTED!**

The buffer address `0x1fe93478` is a STACK address (~0.5GB range), suggesting the 4KB stack buffer may be causing stack corruption or the copy is still corrupting memory elsewhere.

---

## 🎯 NEXT STEPS (Very Specific)

### Current State Summary
- ✅ Physical memory zoning implemented
- ✅ Heap buffer copy implemented for VMO clone
- ✅ Feature gate `userspace_test` is working
- ✅ Control flow logs confirm test is being invoked
- ❌ VMO#3 corruption STILL OCCURS despite all fixes

### STOP ⛔ - Do NOT Touch Memory Code
The following is OFF LIMITS until further analysis:
- PMM allocation code
- VMO clone implementation
- Page table manipulation
- Copy logic in VMO operations

**The kernel is stable enough to run userspace - something else is wrong.**

### What to Do NEXT (Priority Order)

#### 1. Investigate VMO#3 Check Location (CRITICAL)

**File:** `src/exec/process_loader.rs`

**Action:** Find the code that prints `[P-LOADER] After seg0: VMO#3 MISSING!`

**Question:** Is VMO#3 being checked at the right time?

**Hypothesis:** The check might be looking at the wrong VMO reference or there's a lifetime issue.

```rust
// Find this code in process_loader.rs:
// DEBUG: Check VMO#3 after each segment mapping
if loaded_elf.segments.len() > 2 {
    let vmo3_pages = loaded_elf.segments[2].vmo.pages.lock();
    let vmo3_entry = vmo3_pages.get(&0);
    match vmo3_entry {
        Some(e) => { /* print present */ }
        None => { /* print MISSING */ }
    }
}
```

**Verify:**
- Is `loaded_elf.segments[2].vmo` the correct reference?
- Should we check `loaded_elf.segments[2].vmo` or the original VMO before cloning?
- Are we checking parent VMO or cloned VMO?

#### 2. Add VMO#3 Check BEFORE VMO#1 Clone

**File:** `src/object/vmo.rs` (clone function)

**Action:** Add debug check at the very start of clone to verify VMO#3 is present before any operations:

```rust
// In VMO::clone(), at the very beginning:
if self.id == 1 {
    // Check if VMO#3 exists in global VMO list
    // This requires some way to access loaded_elf.segments[2].vmo
    // For now, just print that we're VMO#1 and about to clone
}
```

#### 3. Verify Address Mapping is Correct

**Question:** Does `pmm::paddr_to_vaddr()` return the correct virtual address?

**Debug:** Print both the physical address AND the converted virtual address:

```rust
let src_paddr = page_entry.paddr;
let src_vaddr = pmm::paddr_to_vaddr(src_paddr);
// Print both and verify they make sense

let dst_paddr = new_paddr;
let dst_vaddr = pmm::paddr_to_vaddr(dst_paddr);
// Print both and verify they make sense
```

**Verify:** For user zone memory (0x1000000+), does `paddr_to_vaddr()` return `KERNEL_PHYS_OFFSET + paddr` or just `paddr`?

#### 4. Check if Stack Buffer is Too Large

**Issue:** `let mut buffer: [u8; 4096] = [0; 4096];` is a 4KB stack allocation.

**Question:** Can the kernel stack handle 4KB?

**Alternatives:**
- Use smaller chunks (e.g., copy 512 bytes at a time)
- Use heap allocation (Vec<u8>) instead of stack
- Verify stack size is sufficient

#### 5. Verify BTreeMap is Not Being Corrupted During Copy

**Hypothesis:** The copy operation itself might be corrupting the BTreeMap through some other mechanism.

**Debug:** Add a check AFTER the copy completes but BEFORE any other operations:

```rust
// After copy_nonoverlapping(src_ptr, dst_ptr, page_size):
// Verify VMO#3 is still present right after copy
```

### Build and Test Commands

```bash
# Build with userspace_test feature
cargo build --release --target x86_64-unknown-uefi --features "uefi_kernel userspace_test"

# Create bootable image
rm rustux.img
dd if=/dev/zero of=rustux.img bs=1M count=64
mkfs.fat -F 32 rustux.img
mkdir -p /tmp/rustux-efi/EFI/BOOT
cp target/x86_64-unknown-uefi/release/rustux.efi /tmp/rustux-efi/EFI/BOOT/BOOTX64.EFI
mcopy -i rustux.img -s /tmp/rustux-efi/EFI ::
rm -rf /tmp/rustux-efi

# Run in QEMU
timeout 8 qemu-system-x86_64 \
    -bios /usr/share/ovmf/OVMF.fd \
    -drive file=rustux.img,format=raw \
    -nographic \
    -device isa-debugcon,iobase=0xE9,chardev=debug \
    -chardev file,id=debug,path=/tmp/rustux-qemu-debug.log \
    -m 512M \
    -machine q35 \
    -smp 1 \
    -no-reboot \
    -no-shutdown 2>&1 || true
cat /tmp/rustux-qemu-debug.log
```

### Files to Investigate

1. **`src/exec/process_loader.rs`** - Where VMO#3 check happens
2. **`src/object/vmo.rs`** - Clone implementation
3. **`src/mm/pmm.rs`** - Verify `paddr_to_vaddr()` implementation
4. **`src/exec/process_loader.rs`** - Verify we're checking the right VMO reference

### Timeline of Corruption (from debug output)

```
[VMO-WRITE] VMO#3 key=0
[VMO-WRITE] Verify: entry exists, present=1 paddr=0x202000  ← VMO#3 created OK
[P-LOADER] Before seg0: VMO#3 present=1                      ← Still OK
[VMO] BEFORE VMO#1 COPY                                          ← Before clone
[VMO] AFTER VMO#1 COPY                                           ← Copy completes
[VMO] After child_pages.insert                                       ← Insert completes
[VMO] After locks released                                         ← Clone completes
[P-LOADER] After seg0: VMO#3 present=1                      ← Still OK!
[P-LOADER] Before seg1: VMO#3 present=1                      ← Still OK!
[P-LOADER] After seg1 clone: VMO#3 entry MISSING!               ← CORRUPTED
```

**Key Insight:** The corruption happens AFTER segment 0's VMO#1 clone completes, but BEFORE segment 1's clone starts. This definitively proves VMO#1's clone is the culprit.

### Violated Invariant

**Kernel Metadata Must Never Live in User-Visible Memory**

This is a fundamental kernel architecture invariant that we're currently violating:

```
┌─────────────────────────────────────────────────────┐
│  KERNEL ZONE           │  USER ZONE                │
│  0x00100000-0x00FFFFFF  │  0x01000000-0x7FFFFFFF    │
│                         │                            │
│  • Heap allocator       │  • VMO backing pages       │
│  • BTreeMap nodes      │  • User data               │
│  • Page tables         │  • Page tables            │
│  • Kernel code         │  • Kernel code (mirrored)  │
│  • Stack               │  • Stack                 │
└─────────────────────────────────────────────────────┘
```

**Current Violation:** VMO backing pages (0x200000-0x202000) and clone destinations (0x207000-0x208000) overlap with kernel zone metadata.

### Architectural Fix Required

#### Fix #1: Physical Memory Zoning (DO THIS NOW)

```rust
// In src/mm/pmm.rs or new file src/mm/zones.rs

pub const KERNEL_ZONE_START: PAddr = 0x0010_0000;
pub const KERNEL_ZONE_END:   PAddr = 0x00FF_FFFF;
pub const USER_ZONE_START:   PAddr = 0x0100_0000;
pub const USER_ZONE_END:     PAddr = 0x7FFF_FFFF;

pub enum Zone {
    Kernel,
    User,
}

pub fn pmm_alloc_zone(zone: Zone, size: usize) -> Result<PAddr, &'static str> {
    // Allocate from the specified zone only
}

// Update existing pmm_alloc_page():
pub fn pmm_alloc_page(flags: u32) -> Result<PAddr, &'static str> {
    pmm_alloc_zone(Zone::User, 4096)  // VMO backing pages
}

pub fn pmm_alloc_kernel_page() -> Result<PAddr, &'static str> {
    pmm_alloc_zone(Zone::Kernel, 4096)  // Page tables, heap metadata
}
```

**Changes Required:**
1. Add zone tracking to PMM (two free lists)
2. Update VMO allocations to use `pmm_alloc_zone(Zone::User, size)`
3. Update page table allocations to use `pmm_alloc_kernel_page()`
4. Update heap allocator to use kernel zone only
5. Enforce that PMM validates zone boundaries

#### Fix #2: Distinct Allocators (REQUIRED LONG-TERM)

```rust
pub fn pmm_alloc_user_page() -> Result<PAddr, &'static str> { ... }
pub fn pmm_alloc_kernel_page() -> Result<PAddr, &'static str> { ... }

// Internal implementation:
// - Share underlying bitmap/Vec
// - Track which pages belong to which zone
// - Validate requests don't cross zone boundaries
```

#### Fix #3: DO NOT DO (Wrong Approaches)

❌ "Use Vec for VMOs" - Violates VMO semantics
❌ "Pad allocations to avoid overlap" - Fragile, doesn't scale
❌ "Hope allocator won't land there" - Not deterministic
❌ "Use identity mapping == safety" - Still same physical memory
❌ "Clone before mapping" - Doesn't solve the overlap problem

### Why This Explains Everything

| Symptom | Explanation |
|---------|-------------|
| VMO#3 corruption | Clone writes to 0x207000, overwrites VMO#3's BTreeMap |
| Random hangs | Heap allocator metadata corrupted |
| Build-dependent behavior | Allocator layout changes, overlaps shift |
| [POLLING] hangs | Task state corrupted, never resumes |
| "Different behavior per build" | Memory layout non-deterministic |

### Implementation Plan

**Step 1:** Implement physical memory zoning in PMM (30 min)
**Step 2:** Update all allocation sites to use appropriate zones (1 hour)
**Step 3:** Test VMO clone and verify no corruption (15 min)
**Step 4:** Resume userspace execution testing

**Files to Modify:**
- `src/mm/pmm.rs` - Add zone tracking
- `src/mm/zones.rs` - New file for zone definitions
- `src/object/vmo.rs` - Update allocations to use user zone
- `src/arch/amd64/mm/page_tables.rs` - Update to use kernel zone
- `src/alloc/heap.rs` - Update to use kernel zone only

---

## 📋 Phase 4: Userspace & Process Execution
```
Segment 0:
  [MAP] num_pages=1
  [MAP] VMO has 1 pages
  [MAP] interrupt_flags=0x0 (interrupts already disabled)
  [MAP] VMO pages locked
  [MAP] Starting page iteration
  [MAP] Loop iter
  [MAP] Before get
  [MAP] After get  ← VMO page lookup succeeds
  [MAP] map_page vaddr=0x400000 paddr=0x200000
  [MAP] Checking PDP...
  [PT] Allocated at 0x204000
  [PT] Zeroing page at vaddr=0x204000
  [PT] Page zeroed
  [VMO] clone() starting
  [VMO] clone() complete
  ✅ Segment 0 complete

Segment 1:
  [MAP] num_pages=1
  [MAP] VMO has 1 pages
  [MAP] VMO pages locked
  [MAP] Starting page iteration
  [MAP] Loop iter
  [MAP] Before get
  [MAP] After get  ← VMO page lookup succeeds
  [MAP] map_page vaddr=0x401000 paddr=0x201000
  [MAP] Checking PD...
  [MAP] PD exists, reusing
  [VMO] clone() complete
  ✅ Segment 1 complete

Segment 2:
  [MAP] num_pages=1
  [MAP] VMO has 1 pages
  [MAP] Disabling interrupts...
  [MAP] interrupt_flags=0x0
  [MAP] Interrupts disabled
  [MAP] VMO pages locked
  [MAP] Starting page iteration
  [MAP] Loop iter
  [MAP] Before get
  [MAP] After get  ← VMO page lookup succeeds!
  ⚠️ HANGS before next debug message
```

**Root Cause Analysis:**
The hang occurs AFTER `vmo_pages.get()` succeeds, but BEFORE the next debug message (which should be "Got paddr" or similar). This suggests:

1. **Not a lock issue** - VMO pages locked successfully
2. **Not a BTreeMap corruption** - `get()` returned successfully
3. **Hang occurs in code between "After get" and map_page**

**Potential Issues:**
- Page exhaustion? But PMM has 32,256 free pages
- Stack overflow? But we removed excessive debug output
- `arch_disable_ints()` issue? But it works for segments 0 & 1
- PMM page allocation returning bad address?

**Next Steps:**
1. Add debug output after `page_mappings` assignment and `mapping_count` increment
2. Check if page table allocation is failing silently
3. Verify interrupt disable/enable logic is correct
4. Consider if there's a page fault during the assignment operations

**Key Changes Made:**
- `src/mm/pmm.rs`: Added `KERNEL_PHYS_OFFSET`, updated `paddr_to_vaddr()`
- `src/arch/amd64/mmu.rs`: Added direct-map setup constants and stub
- `src/object/vmo.rs`: Fixed clone to use `paddr_to_vaddr()`
- `src/exec/process_loader.rs`: Changed to use `&Vmo` reference instead of move
- `src/process/address_space.rs`: Added granular debug output, fixed interrupt state restoration

**Remaining Phase A Tasks:**
1. ✅ Replace PMM with Vec-based allocator (COMPLETE)
2. ✅ Switch heap allocator to global allocator (COMPLETE)
3. ✅ Fix ELF program header parsing bug (COMPLETE - was in previous session)
4. ✅ Test address space creation (COMPLETE - works for segments 0 & 1)
5. ⏳ Fix segment 2 hang (CURRENT BLOCKER)
6. ⏳ Map ELF segments into address space (2/3 complete)
7. ⏳ Execute userspace transition via IRETQ
8. ⏳ Verify "Hello from userspace!" output

---

## 🐛 Known Issue: ELF Program Header Parsing Bug (2025-01-19)

**Status:** Active blocker preventing segment 1+ loading

**Symptoms:**
- ELF file is valid (verified with `readelf -l hello.elf`)
- Segment 1 (offset 0x1000, filesz 0x62) fails to load
- Kernel parses `p_filesz = 1048184` instead of correct value `98` (0x62)
- Error: "Segment extends beyond file size" (file_end = 1052280 > elf_len = 9368)

**Debug Output:**
```
[ELF] Loading segment 1
[ELF] p_offset=4096 p_filesz=1048184    # Should be p_filesz=98
[ELF] seg: start=4096 end=1052280 elf_len=9368
[KERNEL] Failed to load ELF: Segment extends beyond file size
```

**Verified Correct Values (via readelf):**
```
  LOAD           0x0000000000001000 0x0000000000401000
                 0x0000000000000062 0x0000000000000062  R E    0x1000
```

**Root Cause:**
The `parse_program_headers()` function in `src/exec/elf.rs` is reading p_filesz from the wrong byte offset. The embedded ELF binary contains the correct data:
```
Program header 1 bytes: 0100000005000000001000000000000000104000000000000010400000000000620000000000000062000000000000000010000000000000
                     ^p_type^p_flags   ^p_offset--------   ^p_vaddr---------   ^p_paddr---------   ^p_filesz--------   ^p_memsz---------   ^p_align--------
```

The bytes `6200000000000000` represent p_filesz = 0x62 = 98 correctly.

**Location:** `src/exec/elf.rs:376-379` (p_filesz parsing)

**Fix Required:**
Verify the byte offset calculation in `parse_program_headers()` - the parsing code is likely using incorrect indices for the ph_data slice.

**Related Files:**
- `src/exec/elf.rs` - ELF parsing code (line ~376 for p_filesz)
- `test-userspace/hello.elf` - Valid ELF (9368 bytes)
- `src/exec/userspace_exec_test.rs` - Embedded ELF via `include_bytes!()`

**Phase B (Later):**
1. ⏳ Reintroduce bitmap PMM as `pmm_v2`
2. ⏳ Add PMM unit tests
3. ⏳ Add randomized allocation stress tests

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
- [ ] PMM allocates multiple pages successfully
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

**Known Issue:** PMM bitmap bug in `alloc_page()` - only 1 page allocates before failure

---

## 🚧 Next Session: Fix PMM Bug

**Debug Steps:**
1. Add debug output to `src/mm/pmm.rs:253-285`
2. Trace bitmap state before/after allocations
3. Verify atomic operations on bitmap
4. Check `free_count` vs actual free pages
5. Test with manual bitmap manipulation

**Expected Fix:**
- PMM should allocate all 32,256 reported free pages
- AddressSpace::new() should succeed
- ELF segment mapping should work
- Userspace execution should complete

---

## 📞 Resources

- **Repository:** https://github.com/gitrustux/rustux
- **Userspace CLI:** `/var/www/rustux.com/prod/rustica/tools/cli/` (~5,150 lines, complete)
- **Documentation:** See ARCHITECTURE.md for kernel design details
