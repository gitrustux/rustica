# Rustux OS - Development Plan

**Last Updated:** 2025-01-23 - Phase 5 COMPLETE, Phase 6A-6C COMPLETE, Phase 6D-7 PLANNED
**Current Status:** 🟢 INTERACTIVE SHELL RUNNING - PS/2 keyboard, framebuffer console, Dracula theme, init → shell boot
**Kernel Location:** `/var/www/rustux.com/prod/rustux/`
**Userspace/OS Location:** `/var/www/rustux.com/prod/rustica/`
**Old Installer:** `/var/www/rustux.com/prod/installer/` (OBSOLETE - was for Linux-based OS)

---

## ⚠️ Known-Good Boot Environment (DO NOT DEVIATE)

**The kernel only boots reliably under the following configuration:**

### Required Configuration

| Component | Path/Version | Notes |
|-----------|--------------|-------|
| **QEMU** | `/usr/local/bin/qemu-system-x86_64` | 7.2.0 (source-built) |
| **Firmware** | `/usr/share/qemu/OVMF.fd` | System OVMF |
| **Machine Type** | Default | - |

### ❌ BROKEN CONFIGURATION

- System QEMU 8.x (`/usr/bin/qemu-system-x86_64` version 8.2.2)

---

## 🤫 Silent Boot Phase (CRITICAL INVARIANT)

**From `efi_main` until `ExitBootServices()` succeeds, the kernel MUST NOT:**

- Use port I/O (including debug port 0xE9)
- Use global logging macros (`println!`, `debug_print!`, etc.)
- Call UEFI console services more than once
- Use heap allocations

**Silent Boot Phase ends ONLY when:**

1. `ExitBootServices()` returns successfully
2. Page tables are stable
3. Interrupts are explicitly disabled

Then AND ONLY THEN:
- Enable port 0xE9 debug output
- Enable kernel logger

---

## 📋 Phase 4: Userspace & Process Execution ✅ COMPLETE

### Phase 4A: ELF Loading & Heap Allocator ✅
- Per-process address spaces
- 64MB heap allocator with MIN_BLOCK_SIZE=1024
- ELF binary loading with segment mapping
- VMO (Virtual Memory Object) abstraction

### Phase 4B: First CPL3 Instruction ✅
- Page table isolation (kernel vs userspace)
- Per-process PML4 creation
- PML4 ownership rules enforced
- Canary reads before CR3 load
- CR3 switching with canary verification
- IRETQ to userspace
- TSS RSP0 configuration
- User segments configured
- Silent Boot Phase enforced

### Phase 4C: Syscalls & Userspace Memory ✅
- `int 0x80` syscall interface
- Syscall dispatch table
- sys_exit() - process termination
- sys_debug_write() - kernel-mediated debug output
- Argument/return handling via interrupt frame

---

## 📋 Phase 5: Process Management & Essential Syscalls (NEXT)

### Overview

Build on Phase 4's userspace execution foundation to add:
- Proper I/O syscalls for userspace programs
- Process table and scheduler for multiple processes
- Embedded filesystem for files and programs
- Multi-process demo showing kernel capabilities

### Phase 5A: Core Syscall Interface (Week 1-2)

**Goal:** Enable userspace programs to do I/O and get process information

#### 5A.1: File Descriptor Abstraction

```rust
// src/syscall/fd.rs

/// File descriptor kinds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FdKind {
    Stdin,   // 0: Keyboard input (future)
    Stdout,  // 1: Kernel debug console (port 0xE9)
    Stderr,  // 2: Same as stdout for now
    File {   // 3+: Embedded ramdisk file
        inode: u32,
        offset: u64,
    },
    Pipe {   // Future: Inter-process communication
        read_end: bool,
        pipe_id: u32,
    },
}

/// File descriptor entry
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub kind: FdKind,
    pub flags: u32,  // O_RDONLY, O_WRONLY, O_RDWR, etc.
}

/// Per-process file descriptor table
pub struct FileDescriptorTable {
    fds: [Option<FileDescriptor>; 256],
    next_fd: u8,
}

impl FileDescriptorTable {
    pub const fn new() -> Self {
        Self {
            fds: [None; 256],
            next_fd: 3,  // Start after stdin/stdout/stderr
        }
    }

    pub fn alloc(&mut self, kind: FdKind, flags: u32) -> Option<u8> {
        if self.next_fd >= 256 {
            return None;
        }
        let fd = self.next_fd;
        self.fds[fd as usize] = Some(FileDescriptor { kind, flags });
        self.next_fd += 1;
        Some(fd)
    }

    pub fn get(&self, fd: u8) -> Option<&FileDescriptor> {
        self.fds.get(fd as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, fd: u8) -> Option<&mut FileDescriptor> {
        self.fds.get_mut(fd as usize)?.as_mut()
    }

    pub fn close(&mut self, fd: u8) -> Option<FileDescriptor> {
        self.fds.get_mut(fd as usize)?.take()
    }
}
```

#### 5A.2: I/O Syscalls

```rust
// src/syscall/mod.rs

/// Write to file descriptor
/// sys_write(fd: u8, buf: *const u8, len: usize) -> isize
fn sys_write(args: SyscallArgs) -> SyscallRet {
    let fd = args.arg_u8(0);
    let buf = args.arg_ptr(1) as *const u8;
    let len = args.arg(2);

    let process = current_process();
    let fd_table = &process.fd_table;

    let desc = match fd_table.get(fd) {
        Some(d) => d,
        None => return error_to_ret_isize(Errno::EBADF),
    };

    match desc.kind {
        FdKind::Stdout | FdKind::Stderr => {
            // Write to kernel debug console (port 0xE9)
            unsafe {
                for i in 0..len {
                    let c = *(buf.add(i));
                    core::arch::asm!("out dx, al",
                        in("dx") 0xE9u16,
                        in("al") c,
                        options(nomem, nostack)
                    );
                }
            }
            ok_to_ret_isize(len as isize)
        }
        FdKind::Stdin => error_to_ret_isize(Errno::EBADF),  // Can't write to stdin
        FdKind::File { inode, offset } => {
            // File writing (Phase 5C)
            error_to_ret_isize(Errno::EROFS)  // Read-only for now
        }
        FdKind::Pipe { .. } => error_to_ret_isize(Errno::ENOSYS),
    }
}

/// Read from file descriptor
/// sys_read(fd: u8, buf: *mut u8, len: usize) -> isize
fn sys_read(args: SyscallArgs) -> SyscallRet {
    let fd = args.arg_u8(0);
    let buf = args.arg_mut_ptr(1) as *mut u8;
    let len = args.arg(2);

    let process = current_process();
    let fd_table = &process.fd_table;

    let desc = match fd_table.get(fd) {
        Some(d) => d,
        None => return error_to_ret_isize(Errno::EBADF),
    };

    match desc.kind {
        FdKind::Stdin => {
            // Keyboard input (future)
            error_to_ret_isize(Errno::ENOSYS)
        }
        FdKind::Stdout | FdKind::Stderr => error_to_ret_isize(Errno::EBADF),
        FdKind::File { .. } => {
            // File reading (Phase 5C)
            error_to_ret_isize(Errno::ENOSYS)
        }
        FdKind::Pipe { .. } => error_to_ret_isize(Errno::ENOSYS),
    }
}
```

#### 5A.3: Memory Management Syscalls

```rust
/// Map memory into process address space
/// sys_mmap(addr: Option<usize>, len: usize, prot: u32, flags: u32) -> *mut u8
fn sys_mmap(args: SyscallArgs) -> SyscallRet {
    let addr = args.arg_opt(0);
    let len = args.arg(1);
    let prot = args.arg_u32(2);  // PROT_READ, PROT_WRITE, PROT_EXEC
    let flags = args.arg_u32(3); // MAP_PRIVATE, MAP_SHARED, MAP_ANONYMOUS

    let process = current_process();
    let vmo = process.vmo_create_aligned(len, 4096)?;

    // Map into user address space
    let vaddr = if let Some(addr) = addr {
        process.vmo_map_at(vmo, addr, prot)?
    } else {
        process.vmo_map(vmo, prot)?
    };

    ok_to_ret_usize(vaddr)
}

/// Unmap memory from process address space
/// sys_munmap(addr: usize, len: usize) -> i32
fn sys_munmap(args: SyscallArgs) -> SyscallRet {
    let addr = args.arg(0);
    let len = args.arg(1);

    let process = current_process();
    process.vmo_unmap(addr, len)?;
    ok_to_ret_i32(0)
}
```

#### 5A.4: Process Info Syscalls

```rust
/// Get current process ID
/// sys_getpid() -> u32
fn sys_getpid(_args: SyscallArgs) -> SyscallRet {
    let process = current_process();
    ok_to_ret_u32(process.pid)
}

/// Get parent process ID
/// sys_getppid() -> u32
fn sys_getppid(_args: SyscallArgs) -> SyscallRet {
    let process = current_process();
    ok_to_ret_u32(process.ppid)
}

/// Yield CPU to scheduler
/// sys_yield() -> i32
fn sys_yield(_args: SyscallArgs) -> SyscallRet {
    scheduler::yield_cpu();
    ok_to_ret_i32(0)
}

/// Get process information
/// sys_process_info(pid: u32, info: *mut ProcessInfo) -> i32
#[repr(C)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub state: u32,
    pub rsp: u64,
    pub rip: u64,
}

fn sys_process_info(args: SyscallArgs) -> SyscallRet {
    let pid = args.arg_u32(0);
    let info_ptr = args.arg_mut_ptr(1) as *mut ProcessInfo;

    let process_table = PROCESS_TABLE.lock();
    let process = process_table.get(pid)
        .ok_or(Errno::ESRCH)?;

    unsafe {
        *info_ptr = ProcessInfo {
            pid: process.pid,
            ppid: process.ppid,
            state: process.state as u32,
            rsp: process.rsp,
            rip: process.rip,
        };
    }

    ok_to_ret_i32(0)
}
```

**Deliverables:**
- [x] File descriptor abstraction (FdKind, FileDescriptor, FileDescriptorTable) - `src/syscall/fd.rs`
- [x] sys_write implementation - writes to port 0xE9 debug console
- [x] sys_read implementation - returns EOF for now
- [ ] sys_mmap/sys_munmap implementation - deferred to future phase
- [x] sys_getpid implementation - returns PID 1 placeholder
- [x] sys_getppid implementation - returns PPID 0 placeholder
- [x] sys_yield implementation - CPU yield stub
- [ ] sys_process_info implementation - deferred to future phase
- [x] Userspace test programs using new syscalls - `test-userspace/hello.c`

**Status:** ✅ Phase 5A COMPLETE - Core syscalls implemented and kernel boots successfully

### Phase 5B: Process Table & Scheduler (Week 3-4)

**Goal:** Support multiple processes with round-robin scheduling

#### 5B.1: Process Table Implementation

```rust
// src/process/table.rs

use crate::memory::{PhysAddr, VirtAddr};
use crate::arch::amd64::registers::{Cr3, rflags};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Zombie,
    Dead,
}

/// Saved CPU state during context switch
#[repr(C)]
pub struct SavedState {
    // General-purpose registers
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub r8:  u64, pub r9:  u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,

    // Control registers
    pub cr3: u64,
    pub rflags: u64,

    // Instruction pointers
    pub rip: u64,

    // Segment selectors
    pub cs: u64, pub ss: u64,

    // FPU state (512 bytes for FXSAVE)
    pub fpu: [u8; 512],
}

/// Process descriptor
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub state: ProcessState,

    // Address space
    pub page_table: PhysAddr,
    pub kernel_stack: VirtAddr,
    pub user_stack: VirtAddr,
    pub heap_base: VirtAddr,
    pub heap_size: usize,

    // Execution state
    pub saved_state: SavedState,

    // Syscall return handling
    pub syscall_ret: u64,

    // File descriptors
    pub fd_table: crate::syscall::fd::FileDescriptorTable,

    // Time accounting
    pub cpu_time: u64,
    pub sched_time: u64,
}

pub struct ProcessTable {
    processes: [Option<Process>; 256],
    current: Option<u32>,
    next_pid: u32,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            processes: [None; 256],
            current: None,
            next_pid: 1,  // PID 0 is kernel
        }
    }

    pub fn current(&self) -> Option<&Process> {
        self.current.and_then(|pid| self.processes[pid as usize].as_ref())
    }

    pub fn current_mut(&mut self) -> Option<&mut Process> {
        self.current.and_then(move |pid| self.processes[pid as usize].as_mut())
    }

    pub fn get(&self, pid: u32) -> Option<&Process> {
        self.processes.get(pid as usize)?.as_ref()
    }

    pub fn alloc_pid(&mut self) -> Option<u32> {
        if self.next_pid >= 256 {
            return None;
        }
        let pid = self.next_pid;
        self.next_pid += 1;
        Some(pid)
    }

    pub fn insert(&mut self, process: Process) {
        let pid = process.pid;
        self.processes[pid as usize] = Some(process);
    }

    pub fn set_current(&mut self, pid: u32) {
        self.current = Some(pid);
    }
}

// Global process table
use spin::Mutex;
pub static PROCESS_TABLE: Mutex<ProcessTable> = Mutex::new(ProcessTable::new());
```

#### 5B.2: Context Switching

```assembly
# src/arch/amd64/switch.S

# void context_switch(SavedState *prev, SavedState *next, u64 next_cr3)
# Switches from one process to another
.global context_switch
context_switch:
    # Save current state to prev
    mov %rax, (%rdi)
    mov %rbx, 8(%rdi)
    mov %rcx, 16(%rdi)
    mov %rdx, 24(%rdi)
    mov %rsi, 32(%rdi)
    mov %rbp, 40(%rdi)
    mov %rsp, 48(%rdi)     # Save RSP
    leaq 60(%rdi), %rax    # Calculate RDI after push
    mov %r8, 56(%rdi)
    mov %r9, 64(%rdi)
    mov %r10, 72(%rdi)
    mov %r11, 80(%rdi)
    mov %r12, 88(%rdi)
    mov %r13, 96(%rdi)
    mov %r14, 104(%rdi)
    mov %r15, 112(%rdi)

    # Save CR3
    mov %cr3, %rax
    mov %rax, 120(%rdi)

    # Save RFLAGS
    pushfq
    popq %rax
    mov %rax, 128(%rdi)

    # Save RIP
    movq $1f, %rax
    mov %rax, 136(%rdi)

    # Save segments
    mov %cs, %ax
    movzwq %ax, %rax
    mov %rax, 144(%rdi)
    mov %ss, %ax
    movzwq %ax, %rax
    mov %rax, 152(%rdi)

    # Save FPU state
    fxsave 160(%rdi)

    # Load next CR3
    mov %rdx, %cr3

    # Restore next state
    mov (%rsi), %rax
    mov 8(%rsi), %rbx
    mov 16(%rsi), %rcx
    mov 24(%rsi), %rdx
    mov 32(%rsi), %rsi     # Overwrites RSI! Use saved value
    mov 40(%rsi), %rbp
    mov 48(%rsi), %rsp     # Restore RSP
    mov 56(%rsi), %r8
    mov 64(%rsi), %r9
    mov 72(%rsi), %r10
    mov 80(%rsi), %r11
    mov 88(%rsi), %r12
    mov 96(%rsi), %r13
    mov 104(%rsi), %r14
    mov 112(%rsi), %r15

    # Restore RFLAGS
    mov 128(%rsi), %rax
    pushq %rax
    popfq

    # Restore segments
    mov 144(%rsi), %ax
    mov %ax, %cs
    mov 152(%rsi), %ax
    mov %ax, %ss

    # Restore FPU state
    fxrstor 160(%rsi)

    # Restore RIP
    mov 136(%rsi), %rax
    jmp *%rax

1:  # Return point after context switch
    ret
```

```rust
// src/process/switch.rs

extern "C" {
    fn context_switch(prev: *mut SavedState, next: *const SavedState, next_cr3: u64);
}

pub fn switch_to(current: &mut Process, next: &Process) {
    unsafe {
        context_switch(
            &mut current.saved_state as *mut SavedState,
            &next.saved_state as *const SavedState,
            next.page_table.as_u64(),
        );
    }
}
```

#### 5B.3: Round-Robin Scheduler

```rust
// src/scheduler/mod.rs

use crate::process::{ProcessTable, ProcessState};
use spin::Mutex;
use alloc::sync::Arc;

const TIME_SLICE_MS: u64 = 10;

pub struct Scheduler {
    pub current: Option<u32>,
    pub queue: alloc::collections::VecDeque<u32>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            current: None,
            queue: alloc::collections::VecDeque::new(),
        }
    }

    pub fn schedule(&mut self, process_table: &mut ProcessTable) {
        // Mark current as Ready if it was Running
        if let Some(pid) = self.current {
            if let Some(process) = process_table.get_mut(pid) {
                if process.state == ProcessState::Running {
                    process.state = ProcessState::Ready;
                }
            }
        }

        // Find next runnable process
        let next_pid = self.find_next_runnable(process_table);

        if let Some(pid) = next_pid {
            self.current = Some(pid);
            process_table.set_current(pid);

            let process = process_table.get_mut(pid).unwrap();
            process.state = ProcessState::Running;
        }
    }

    fn find_next_runnable(&self, process_table: &ProcessTable) -> Option<u32> {
        // Simple round-robin: iterate through all PIDs
        for pid in 1..256 {
            if let Some(process) = process_table.get(pid) {
                if process.state == ProcessState::Ready {
                    return Some(pid);
                }
            }
        }
        None
    }
}

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Timer interrupt handler
pub fn timer_tick() {
    let mut scheduler = SCHEDULER.lock();
    let mut process_table = PROCESS_TABLE.lock();

    scheduler.schedule(&mut process_table);

    // Perform context switch if needed
    if let Some(current_pid) = scheduler.current {
        if let Some(next_pid) = scheduler.find_next_runnable(&process_table) {
            if current_pid != next_pid {
                let current = process_table.get_mut(current_pid).unwrap();
                let next = process_table.get(next_pid).unwrap();

                // This will call context_switch assembly
                crate::process::switch_to(current, next);
            }
        }
    }
}

/// Yield CPU voluntarily
pub fn yield_cpu() {
    timer_tick();
}
```

#### 5B.4: Process Spawning Syscall

```rust
// src/syscall/spawn.rs

use crate::exec::process_loader::load_elf;
use crate::process::{Process, ProcessState, ProcessTable};

/// Spawn new process
/// sys_spawn(path: *const u8, argv: *const *const u8) -> u32
fn sys_spawn(args: SyscallArgs) -> SyscallRet {
    let path_ptr = args.arg_ptr(0) as *const u8;
    let argv_ptr = args.arg_ptr(1) as *const *const u8;

    // Read path string (simplified - null-terminated)
    let path = unsafe {
        let mut len = 0;
        while *(path_ptr.add(len)) != 0 {
            len += 1;
        }
        core::slice::from_raw_parts(path_ptr, len)
    };

    // Get parent process
    let parent_pid = {
        let pt = PROCESS_TABLE.lock();
        pt.current.unwrap()
    };

    // Create new address space
    let (page_table, entry_addr) = load_elf(path)?;

    // Allocate kernel stack
    let kernel_stack = crate::memory::allocate_pages(1)?;

    // Allocate user stack
    let user_stack = crate::memory::allocate_pages(4)?;
    let user_stack_top = user_stack.as_u64() + (4 * 4096);

    // Initialize process
    let pid = {
        let mut pt = PROCESS_TABLE.lock();
        let pid = pt.alloc_pid().ok_or(Errno::EAGAIN)?;

        let process = Process {
            pid,
            ppid: parent_pid,
            state: ProcessState::Ready,
            page_table,
            kernel_stack,
            user_stack,
            heap_base: VirtAddr::new(0),
            heap_size: 0,
            saved_state: SavedState::new(entry_addr, user_stack_top),
            syscall_ret: 0,
            fd_table: crate::syscall::fd::FileDescriptorTable::new(),
            cpu_time: 0,
            sched_time: 0,
        };

        pt.insert(process);
        pid
    };

    ok_to_ret_u32(pid)
}
```

**Deliverables:**
- [ ] Process table implementation (Process, ProcessTable, SavedState)
- [ ] Context switch assembly (context_switch in switch.S)
- [ ] Timer-based scheduler (timer_tick, yield_cpu)
- [ ] sys_spawn implementation
- [ ] Two-process demo (init spawns child, both run)

### Phase 5C: Embedded Filesystem (Week 5)

**Goal:** Provide files for programs to read/write

#### 5C.1: Ramdisk Structure

```rust
// src/fs/ramdisk.rs

/// Ramdisk file header (embedded at compile time)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RamdiskFile {
    pub name_offset: u32,  // Offset to name string
    pub data_offset: u32,  // Offset to file data
    pub size: u32,         // File size in bytes
    pub _pad: u32,
}

/// Ramdisk superblock
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RamdiskSuperblock {
    pub magic: u32,        // 0x52555458 ("RUTX")
    pub num_files: u32,
    pub files_offset: u32, // Offset to RamdiskFile array
}

/// Ramdisk filesystem
pub struct Ramdisk {
    pub data: &'static [u8],
    pub superblock: &'static RamdiskSuperblock,
}

impl Ramdisk {
    /// Find file by name
    pub fn find_file(&self, name: &str) -> Option<RamdiskFile> {
        let files = unsafe {
            let base = self.data.as_ptr().add(self.superblock.files_offset as usize);
            let count = self.superblock.num_files as usize;
            core::slice::from_raw_parts(base as *const RamdiskFile, count)
        };

        for &file in files {
            let file_name = unsafe {
                let base = self.data.as_ptr();
                let name_ptr = base.add(file.name_offset as usize);
                let mut len = 0;
                while *name_ptr.add(len) != 0 {
                    len += 1;
                }
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(name_ptr, len))
            };

            if file_name == name {
                return Some(file);
            }
        }
        None
    }

    /// Read file data
    pub fn read_file(&self, file: &RamdiskFile, buf: &mut [u8]) -> usize {
        let data_ptr = unsafe {
            self.data.as_ptr().add(file.data_offset as usize)
        };
        let to_copy = core::cmp::min(buf.len(), file.size as usize);
        unsafe {
            core::ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr(), to_copy);
        }
        to_copy
    }
}

// Global ramdisk instance
pub static RAMDISK: Mutex<Option<Ramdisk>> = Mutex::new(None);
```

#### 5C.2: Build Script for Embedding Files

```rust
// build.rs (for embedding files)

use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=userspace/");
    println!("cargo:rerun-if-changed=files/");

    // Collect all ELF files to embed
    let files = [
        ("userspace/hello.elf", "bin/hello"),
        ("userspace/counter.elf", "bin/counter"),
        ("userspace/spinner.elf", "bin/spinner"),
        ("files/test.txt", "test.txt"),
    ];

    // Create ramdisk binary
    let output = Path::new("target/ramdisk.bin");
    let mut out = File::create(&output).unwrap();

    // Calculate offsets
    let mut offset = 16u32; // Superblock size
    let name_offset = 16 + (files.len() as u32 * 16); // After superblock + file headers

    // Write file headers
    let mut file_headers = Vec::new();
    let mut current_name = name_offset;
    let mut current_data = current_name;

    for (src_path, dst_name) in &files {
        let contents = std::fs::read(src_path).unwrap();
        let name_bytes = dst_name.as_bytes();

        file_headers.push(RamdiskFile {
            name_offset: current_name,
            data_offset: current_data,
            size: contents.len() as u32,
            _pad: 0,
        });

        current_name += name_bytes.len() as u32 + 1; // +1 for null
        current_data += contents.len() as u32;
    }

    // Write superblock
    let superblock = RamdiskSuperblock {
        magic: 0x52555458,
        num_files: files.len() as u32,
        files_offset: 16,
    };

    unsafe {
        out.write_all(core::slice::from_raw_parts(
            &superblock as *const _ as *const u8,
            core::mem::size_of::<RamdiskSuperblock>()
        )).unwrap();
    }

    // Write file headers
    for header in &file_headers {
        unsafe {
            out.write_all(core::slice::from_raw_parts(
                header as *const _ as *const u8,
                core::mem::size_of::<RamdiskFile>()
            )).unwrap();
        }
    }

    // Write names and data
    for (src_path, dst_name) in &files {
        let contents = std::fs::read(src_path).unwrap();
        out.write_all(dst_name.as_bytes()).unwrap();
        out.write_all(&[0]).unwrap(); // Null terminator
        out.write_all(&contents).unwrap();
    }

    println!("cargo:rustc-env=RAMDISK_PATH={}", output.display());
}
```

#### 5C.3: VFS Layer

```rust
// src/fs/vfs.rs

use crate::fs::ramdisk::Ramdisk;

/// VFS inode
#[derive(Debug, Clone, Copy)]
pub enum Inode {
    RamdiskFile { file_idx: u32 },
    Pipe { pipe_id: u32 },
}

/// VFS file operations
pub trait FileOps {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, Errno>;
    fn seek(&mut self, offset: i64, whence: Whence) -> Result<u64, Errno>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Whence {
    Set = 0,
    Cur = 1,
    End = 2,
}

/// Ramdisk file operations
pub struct RamdiskFileOps {
    pub file: RamdiskFile,
    pub offset: u64,
}

impl FileOps for RamdiskFileOps {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Errno> {
        let ramdisk = RAMDISK.lock().as_ref().ok_or(Errno::ENODEV)?;

        let remaining = (self.file.size as u64 - self.offset) as usize;
        let to_read = core::cmp::min(buf.len(), remaining);

        if to_read == 0 {
            return Ok(0);
        }

        // Read from ramdisk at offset
        let data_ptr = unsafe {
            ramdisk.data.as_ptr().add((self.file.data_offset + self.offset) as usize)
        };

        unsafe {
            core::ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr(), to_read);
        }

        self.offset += to_read as u64;
        Ok(to_read)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, Errno> {
        Err(Errno::EROFS) // Read-only
    }

    fn seek(&mut self, offset: i64, whence: Whence) -> Result<u64, Errno> {
        self.offset = match whence {
            Whence::Set => offset as u64,
            Whence::Cur => (self.offset as i64 + offset) as u64,
            Whence::End => (self.file.size as i64 + offset) as u64,
        };
        Ok(self.offset)
    }
}
```

#### 5C.4: File Operation Syscalls

```rust
// src/syscall/file.rs

use crate::fs::{Ramdisk, Inode, FileOps, RamdiskFileOps, Whence};
use crate::syscall::fd::{FdKind, FileDescriptor, FileDescriptorTable};

/// Open file
/// sys_open(path: *const u8, flags: u32) -> i32
fn sys_open(args: SyscallArgs) -> SyscallRet {
    let path_ptr = args.arg_ptr(0) as *const u8;
    let flags = args.arg_u32(1);

    // Read path string
    let path = unsafe {
        let mut len = 0;
        while *(path_ptr.add(len)) != 0 {
            len += 1;
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(path_ptr, len))
    };

    // Find file in ramdisk
    let ramdisk = RAMDISK.lock();
    let ramdisk = ramdisk.as_ref().ok_or(Errno::ENODEV)?;
    let file = ramdisk.find_file(path).ok_or(Errno::ENOENT)?;

    // Allocate file descriptor
    let process = current_process();
    let mut fd_table = process.fd_table.lock();

    let kind = FdKind::File {
        inode: 0, // Will be set by VFS
        offset: 0,
    };

    let fd = fd_table.alloc(kind, flags).ok_or(Errno::EMFILE)?;
    ok_to_ret_i32(fd as i32)
}

/// Close file descriptor
/// sys_close(fd: u8) -> i32
fn sys_close(args: SyscallArgs) -> SyscallRet {
    let fd = args.arg_u8(0);

    let process = current_process();
    let mut fd_table = process.fd_table.lock();

    fd_table.close(fd).ok_or(Errno::EBADF)?;
    ok_to_ret_i32(0)
}

/// Seek in file
/// sys_lseek(fd: u8, offset: i64, whence: i32) -> i64
fn sys_lseek(args: SyscallArgs) -> SyscallRet {
    let fd = args.arg_u8(0);
    let offset = args.arg_i64(1);
    let whence = args.arg_i32(2);

    let process = current_process();
    let fd_table = process.fd_table.lock();

    let desc = fd_table.get(fd).ok_or(Errno::EBADF)?;

    match desc.kind {
        FdKind::File { .. } => {
            // Handle seek via file ops
            // Implementation details depend on VFS integration
            ok_to_ret_i64(offset)
        }
        _ => error_to_ret_i64(Errno::ESPIPE),
    }
}
```

**Deliverables:**
- [ ] Ramdisk structure (RamdiskFile, RamdiskSuperblock, Ramdisk)
- [ ] Build script for embedding files (build.rs)
- [ ] VFS layer (FileOps trait, RamdiskFileOps)
- [ ] sys_open implementation
- [ ] sys_close implementation
- [ ] sys_lseek implementation
- [ ] Integration with sys_read/sys_write
- [ ] Test files and programs

### Phase 5D: Multi-Process Demo (Week 6)

**Goal:** Demonstrate full kernel capabilities

#### 5D.1: Test Programs

**counter.elf** - Simple counter that increments and prints:

```c
// test-userspace/counter.c

#define SYS_DEBUG_WRITE 0x50
#define SYS_YIELD 0x03

static inline long syscall3(long num, long arg1, long arg2, long arg3) {
    long ret;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "b"(arg1), "c"(arg2), "d"(arg3)
        : "memory"
    );
    return ret;
}

static void print_number(int n) {
    char buf[16];
    int i = 0;

    if (n == 0) {
        syscall3(SYS_DEBUG_WRITE, (long)"0", 1, 0);
        return;
    }

    while (n > 0) {
        buf[i++] = '0' + (n % 10);
        n /= 10;
    }

    while (i > 0) {
        syscall3(SYS_DEBUG_WRITE, (long)&buf[--i], 1, 0);
    }
}

void _start(void) {
    int count = 0;

    while (count < 10) {
        syscall3(SYS_DEBUG_WRITE, (long)"[Counter] Count: ", 16, 0);
        print_number(count);
        syscall3(SYS_DEBUG_WRITE, (long)"\n", 1, 0);

        count++;
        syscall1(SYS_YIELD, 0);
    }

    syscall1(SYS_PROCESS_EXIT, 0);
}
```

**hello.elf** - Simple greeting program:

```c
// test-userspace/hello.c

#define SYS_DEBUG_WRITE 0x50
#define SYS_PROCESS_EXIT 0x06

static inline long syscall1(long num, long arg1) {
    long ret;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "b"(arg1)
        : "memory"
    );
    return ret;
}

void _start(void) {
    const char *msg = "[Hello] Hello from userspace!\n";

    syscall1(SYS_DEBUG_WRITE, (long)msg);

    syscall1(SYS_PROCESS_EXIT, 0);
}
```

**spinner.elf** - Visual spinning indicator:

```c
// test-userspace/spinner.c

#define SYS_DEBUG_WRITE 0x50
#define SYS_YIELD 0x03
#define SYS_PROCESS_EXIT 0x06

static inline long syscall1(long num, long arg1) {
    long ret;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "b"(arg1)
        : "memory"
    );
    return ret;
}

static void putc(char c) {
    syscall1(SYS_DEBUG_WRITE, (long)&c);
}

void _start(void) {
    const char spinner[] = {'|', '/', '-', '\\'};
    int i = 0;

    while (1) {
        putc('\r');
        putc('[');
        putc(spinner[i % 4]);
        putc(']');
        putc(' ');
        putc('S');
        putc('p');
        putc('i');
        putc('n');
        putc('n');
        putc('i');
        putc('n');
        putc('g');
        putc('.');
        putc('.');

        i++;
        syscall1(SYS_YIELD, 0);

        if (i >= 40) {
            syscall1(SYS_PROCESS_EXIT, 0);
        }
    }
}
```

#### 5D.2: Init Process

```c
// test-userspace/init.c

#define SYS_SPAWN 0x01
#define SYS_GETPID 0x04
#define SYS_YIELD 0x03
#define SYS_PROCESS_EXIT 0x06
#define SYS_DEBUG_WRITE 0x50

static inline long syscall1(long num, long arg1) {
    long ret;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "b"(arg1)
        : "memory"
    );
    return ret;
}

static inline long syscall2(long num, long arg1, long arg2) {
    long ret;
    __asm__ volatile(
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "b"(arg1), "c"(arg2)
        : "memory"
    );
    return ret;
}

struct spawn_args {
    const char *path;
    const char **argv;
};

void _start(void) {
    int my_pid;
    int child1, child2, child3;
    int status;

    my_pid = syscall1(SYS_GETPID, 0);

    // Print init message
    const char *init_msg = "[Init] Starting PID 1\n";
    syscall2(SYS_DEBUG_WRITE, (long)init_msg, 20);

    // Spawn counter
    const char *counter_path = "/bin/counter";
    child1 = syscall2(SYS_SPAWN, (long)counter_path, 0);

    // Spawn hello
    const char *hello_path = "/bin/hello";
    child2 = syscall2(SYS_SPAWN, (long)hello_path, 0);

    // Spawn spinner
    const char *spinner_path = "/bin/spinner";
    child3 = syscall2(SYS_SPAWN, (long)spinner_path, 0);

    // Yield to children
    for (int i = 0; i < 50; i++) {
        syscall1(SYS_YIELD, 0);
    }

    // Exit
    syscall1(SYS_PROCESS_EXIT, 0);
}
```

#### 5D.3: Integration with Kernel

```rust
// src/main.rs - kernel_main modifications

fn kernel_main() -> Status {
    // ... existing initialization ...

    // Load init as PID 1
    let init_path = "/bin/init";
    let (page_table, entry_addr) = load_elf(init_path).expect("Failed to load init");

    // Create init process
    let init_process = Process {
        pid: 1,
        ppid: 0,
        state: ProcessState::Ready,
        page_table,
        kernel_stack: allocate_kernel_stack()?,
        user_stack: allocate_user_stack()?,
        heap_base: VirtAddr::new(0),
        heap_size: 0,
        saved_state: SavedState::new(entry_addr, user_stack_top),
        syscall_ret: 0,
        fd_table: FileDescriptorTable::new(),
        cpu_time: 0,
        sched_time: 0,
    };

    PROCESS_TABLE.lock().insert(init_process);
    SCHEDULER.lock().current = Some(1);

    // Jump to userspace
    uspace::enter_userspace(entry_addr, user_stack_top);

    Status::SUCCESS
}
```

**Expected Output:**
```
[Init] Starting PID 1
[Scheduler] Switching to PID 2
[Counter] Count: 0
[Scheduler] Switching to PID 3
[Hello] Hello from userspace!
[Scheduler] Switching to PID 4
[-] Spinning..
[Scheduler] Switching to PID 2
[Counter] Count: 1
[Scheduler] Switching to PID 3
[EXIT] PID 3 exited with code 0
[Scheduler] Switching to PID 4
[/] Spinning..
...
```

**Deliverables:**
- [ ] counter.elf program with syscalls
- [ ] hello.elf program
- [ ] spinner.elf program
- [ ] init.elf program that spawns children
- [ ] Build script integration for embedding
- [ ] Kernel modifications to load init as PID 1
- [ ] Scheduler demonstration via debug output
- [ ] Documentation of demo procedure

---

## 🎯 Phase 5 Success Criteria

===
=== PHASE 6 & 7: Updated with latest progress
===

## 📋 Phase 6: Rustica OS Migration (Userland Bring-Up) ⏳ NEXT

**Overview:** Phase 6 migrates the existing Rustica userland onto the stable Rustux kernel foundation, transforming it from a batch-processing system into an interactive operating system.

**Timeline:** 6-8 weeks
**Prerequisites:** Phase 5 complete (multi-process scheduler, syscalls)
**Goal:** Boot to an interactive Rustica CLI with preserved UX (including Dracula theme)

**Status:** Phase 6A (Input Subsystem) - Keyboard driver complete ✅

---

### Phase 6 Goals

**Migrate existing Rustica userland to the new Rustux kernel**
- Preserve existing Rustica code where possible
- Rebuild against new syscall/libc shim

**Establish a minimal POSIX-like ABI boundary**
- Clean syscall interface
- Standard file descriptor conventions (stdin/stdout/stderr)

**Bring up a CLI-capable live environment**
- Bootable live USB for testing and demos
- Interactive shell with full keyboard/display support

**Preserve shell UX continuity**
- Dracula theme retained as default
- Terminal color handling in userspace
- No regression in user experience

**Enable live USB boot for testing and demos**
- FAT32 EFI system partition
- Direct boot to Rustica CLI
- No installer required

---

### Migration Strategy

**Rustica remains pure userspace**
- No Rustica code runs in kernel mode
- All Rustica components are normal userspace processes
- Kernel provides only low-level services

**Kernel provides:**
- ELF loading (via `load_elf_process`)
- Syscalls (int 0x80 interface)
- VFS (ramdisk with file operations)
- Process + memory isolation (CR3 switching, per-process address spaces)

**Rustica apps are rebuilt against a new syscall/libc shim**
- Not the old kernel assumptions
- Clean ABI boundary
- Static linking (no dynamic loader yet)

---

### Rustica Component Mapping

Based on the current Rustica tree structure (`TREE.md`):

| Rustica Component | New Kernel Concept | Notes |
|-------------------|-------------------|-------|
| `repo/apps/cli/` | First-class userspace binaries | Rebuilt as ELF for new kernel |
| `repo/apps/libs/` | Shared userspace libraries | Static linking for now |
| `repo/apps/gui/` | Future GUI userspace | Not part of Phase 6 |
| `update-system/` | Privileged userspace service | Daemon process |
| `update-daemon/` | Long-running system process | Background service |
| `tools/` | Host-side build tooling | Unchanged (host tools) |
| `images/live/` | Phase 6 live USB target | Boot media |
| `releases/cli/*` | Kernel-compatible binary outputs | Final build artifacts |

---

### Minimum Viable Userspace Checklist

**Phase 6A – Minimum Viable CLI Environment**

- [x] `/init` userspace process (PID 1) - Phase 5D ✅
- [x] Syscalls:
  - [x] `write` – stdout/stderr output
  - [x] `read` – stdin input (blocking with keyboard) ✅
  - [x] `exit` – process termination
  - [x] `spawn` – execute programs from ramdisk
  - [ ] `wait` – wait for child processes (future)
- [x] Keyboard input (PS/2 driver) ✅
- [ ] STDOUT wired to framebuffer-backed terminal
- [ ] Single shell binary (`rustica-sh`)
- [ ] Static linking (no dynamic loader yet)

**This is the path from "kernel runs" → "I can type commands".**

---

### Shell & UX Continuity

**Preserve existing Rustica shell theming**
- Dracula theme retained as **default** (non-negotiable)
- Terminal color handling implemented in **userspace**
- No kernel-side theming logic

**This makes it clear:**
- Theme = userspace concern
- Kernel = dumb transport

**Dracula Color Palette (for reference):**
```rust
pub const DRACULA_BG: Color = Color { r: 40, g: 42, b: 54 };
pub const DRACULA_FG: Color = Color { r: 248, g: 248, b: 242 };
pub const DRACULA_PURPLE: Color = Color { r: 189, g: 147, b: 249 };
pub const DRACULA_CYAN: Color = Color { r: 139, g: 233, b: 253 };
pub const DRACULA_GREEN: Color = Color { r: 80, g: 250, b: 123 };
pub const DRACULA_ORANGE: Color = Color { r: 255, g: 184, b: 108 };
pub const DRACULA_RED: Color = Color { r: 248, g: 40, b: 62 };
pub const DRACULA_YELLOW: Color = Color { r: 235, g: 219, b: 178 };
```

---

### Architecture Note

- **Kernel (`rustux/`)**: Drivers, syscalls, scheduler, memory management
- **Userspace/OS (`rustica/`)**: Shell, CLI, built-in commands, theming

The shell lives in `rustica/` as a normal userspace program, NOT in the kernel. This keeps the kernel minimal and makes the shell replaceable.

---

### Phase 6A: Input Subsystem (Week 1-2) ✅ COMPLETE

**Goal:** Read keyboard input from hardware and expose to userspace

#### 6A.1: PS/2 Keyboard Driver ✅

**Files created:**
- `src/drivers/keyboard/mod.rs` - Keyboard driver interface ✅
- `src/drivers/keyboard/ps2.rs` - PS/2 controller implementation ✅
- `src/drivers/keyboard/layout.rs` - Scancode to ASCII conversion ✅

**Deliverables:**
- [x] PS/2 keyboard initialization ✅
- [x] Scancode processing (press/release, extended keys) ✅
- [x] Shift/Ctrl/Alt/Caps Lock handling ✅
- [x] Special key detection (arrows, home, end, etc.) ✅
- [x] Circular buffer for key events ✅

#### 6A.2: Input Syscall Implementation ✅

**Deliverables:**
- [x] sys_read() blocks waiting for keyboard input ✅
- [x] Keyboard IRQ wakes blocked processes ✅
- [x] Multiple processes can read from stdin ✅

#### 6A.3: Line Editing Support

**Note:** Line editing will be implemented in userspace as part of the shell, not in the kernel. This keeps the kernel minimal and puts editing logic where it belongs.

---

### Phase 6B: Display Subsystem (Week 3-4) ✅ COMPLETE

**Goal:** Write pixels and text to screen (VGA or UEFI framebuffer)

#### 6B.1: Framebuffer Driver ✅

**Files created:**
- `src/drivers/display/framebuffer.rs` - Framebuffer management ✅
- `src/drivers/display/font.rs` - PSF2 font support ✅

**Deliverables:**
- [x] Framebuffer initialized from UEFI GOP ✅
- [x] Pixel drawing works ✅
- [x] Rectangle fill works ✅
- [x] Screen scrolling works ✅

#### 6B.2: Text Console ✅

**Files created:**
- `src/drivers/display/console.rs` - Text console with font rendering ✅

**Deliverables:**
- [x] Simple font embedded and working ✅
- [x] Character rendering with font ✅
- [x] Text wrapping and scrolling ✅
- [x] Color support (fg/bg) ✅

#### 6B.3: Display Syscalls ✅

**Deliverables:**
- [x] Userspace programs can print text ✅
- [x] Shell output appears on screen ✅
- [x] No framebuffer access from userspace (kernel only) ✅

---

### Phase 6C: Interactive Shell (Week 5-6) ✅ COMPLETE

**Goal:** Boot directly into an interactive shell running in userspace

**Location:** `/var/www/rustux.com/prod/rustux/test-userspace/` (userspace, NOT kernel)

**Note:** The shell is implemented in C (not Rust) for simplicity in the current kernel environment. A Rust version exists in `/var/www/rustux.com/prod/rustica/shell/` for future migration.

#### 6C.1: Shell Process ✅

**Design:**
- Shell is a normal userspace process
- Launched by init (PID 1)
- Uses stdin/stdout only (no special privileges)
- Blocks on stdin for input
- Uses sys_write() for output

**Responsibilities:**
- [x] Read user input (blocking on stdin)
- [x] Parse commands
- [x] Execute programs (via sys_spawn)
- [x] Handle built-in commands

#### 6C.2: Command Parsing ✅

**Files created:**
- `test-userspace/shell/shell.c` - Shell implementation

**Implementation:**
- Space-separated arguments
- Simple command parsing
- Built-in command detection

#### 6C.3: Built-in Commands ✅

**Implemented built-in commands:**

| Command | Behavior | Status |
|---------|----------|--------|
| `help` | List all commands | ✅ |
| `clear` | Clear screen | ✅ |
| `echo` | Print arguments | ✅ |
| `ps` | List running processes | ✅ |
| `exit` | Exit shell | ✅ |

#### 6C.4: Program Execution ✅

**Flow:**
1. Shell parses command
2. Check if built-in → execute if yes
3. Otherwise → sys_spawn("/bin/<program>")
4. Show success/error message

**This validates:**
- [x] Syscalls work correctly
- [x] Scheduler handles multiple processes
- [x] Process lifecycle is complete
- [x] Userspace is stable

#### 6C.5: Shell Theming (Dracula Theme) ✅

**Implementation:**
- ANSI color codes for terminal output
- Purple prompt (rustux>)
- Cyan command indicator (>)
- Color-coded messages (green for success, red for errors, cyan for info)

**Deliverables:**
- [x] Shell boots as PID 1 (via init)
- [x] Prompt displays in Dracula purple
- [x] Commands parse and execute correctly
- [x] Built-in commands work
- [x] External programs spawn via sys_spawn
- [x] Output displays in themed colors

---

### Phase 6 Implementation Note

**IMPORTANT: Kernel Architecture Transition (January 2025)**

A temporary monolithic UEFI kernel (`loader/kernel-efi/`) was used to validate live boot, PS/2 keyboard input, framebuffer console, and interactive shell functionality.

**Directory Structure:**
- `loader/kernel-efi/` - Monolithic UEFI transition kernel (validated Phase 6 features)
- `rustux/` - Canonical Rustux microkernel (modular architecture)
- `rustica/` - Userspace OS distribution and tools

**Why Two Kernels?**
The transition kernel (`kernel-efi/`) was a pragmatic choice for Phase 6 validation:
- Single UEFI application for direct boot testing
- All drivers in one binary for easier debugging
- Faster iteration on live USB media
- Validated hardware (PS/2 keyboard, framebuffer, UEFI)

**Future Migration (Phase 6D+):**
Phase 6D will migrate validated subsystems from the transition kernel into the canonical microkernel:
- PS/2 keyboard driver → `rustux/src/drivers/keyboard/`
- Framebuffer console → `rustux/src/drivers/display/`
- Live boot tooling → `rustux/src/boot/uefi/`

The monolithic `kernel-efi/` will be retired once migration is complete.

---

### Phase 6D: Stability & UX Guarantees (Week 7-8)

**Goal:** Ensure system is stable and usable for extended sessions

#### 6D.1: Error Handling

**Requirements:**
- No kernel panics from malformed input
- Shell survives child process crashes
- Invalid commands show error, don't crash
- Memory allocation failures handled gracefully

#### 6D.2: Non-Regression Rules

**Critical invariants that MUST be preserved:**
- Silent Boot Phase remains enforced
- Kernel never writes framebuffer after userspace starts (debug output only)
- Keyboard input never blocks kernel threads
- Scheduler always has a runnable process

#### 6D.3: Exit Criteria

**System is considered complete when:**
- [ ] System usable for 30+ minutes without crash
- [ ] No memory leaks during shell usage
- [ ] Reboot returns to shell cleanly
- [ ] All Phase 5 functionality still works
- [ ] Dracula theme displays correctly

---

### Phase 6E: Live Boot Media (Parallel, Not Blocking)

**Goal:** Create a bootable live USB for testing and demos

**Timeline:** Parallel to 6A-6D, can be done incrementally

#### 6E.1: EFI System Partition

**Requirements:**
- FAT32 formatted partition
- `EFI/BOOT/BOOTX64.EFI` → Rustux kernel
- `boot.ini` or similar for kernel arguments

#### 6E.2: Embedded Initramfs

**Approach:**
- Embed initramfs directly in kernel binary
- OR load from secondary filesystem
- Contains:
  - `/init` binary
  - `/bin/rustica-sh`
  - `/bin/*` utilities
  - `/etc/*` configuration

#### 6E.3: Direct Boot to CLI

**Boot sequence:**
1. UEFI loads BOOTX64.EFI
2. Kernel initializes
3. Init process (PID 1) launches
4. Init spawns `rustica-sh`
5. User sees interactive prompt

**No installer required** – this is for testing and demos only

#### 6E.4: Live Media Creation

**Tools to create:**
- `tools/make-live-usb.sh` – Script to create bootable image
- `tools/make-iso.sh` – Script to create ISO for QEMU testing

---

### Phase 6 Non-Goals

**These are explicitly OUT OF SCOPE for Phase 6:**

- **No GUI** – Text-mode CLI only
- **No networking** – Local interaction only (initially)
- **No package manager** – Static binaries embedded in ramdisk
- **No multi-user permissions model** – Single-user system
- **No dynamic linking** – Static binaries only
- **No hardware drivers beyond keyboard/display** – Keep it minimal

**You'll thank yourself later for keeping the scope tight.**

---

## 🎯 Phase 6 Success Criteria

Phase 6A-6C is complete when:

- [x] Keyboard input works (PS/2 driver)
- [x] Screen displays text (framebuffer + console)
- [x] Shell boots and displays Dracula-themed prompt
- [x] Built-in commands work (help, echo, ps, clear, exit)
- [x] External programs spawn and run
- [x] Line editing works (backspace)
- [x] Multiple processes can run concurrently
- [ ] System is stable for extended use (Phase 6D)

---

## 🔧 Development Quick Reference

### Build Kernel
```bash
cd /var/www/rustux.com/prod/rustux/
cargo build --release --target x86_64-unknown-uefi --features uefi_kernel
```

### Build Shell (userspace)
```bash
cd /var/www/rustux.com/prod/rustica/shell
cargo build --release --target x86_64-unknown-none
```

### Run Kernel
```bash
/usr/local/bin/qemu-system-x86_64 \
  -bios /usr/share/qemu/OVMF.fd \
  -drive format=raw,file=disk.img \
  -m 512M
```

---

## 📝 Critical Technical Decisions

### Phase 5 Summary
- Process table with 256 slots, indexed by PID
- Round-robin scheduler with context switching
- Ramdisk for embedded files
- Init process (PID 1) auto-loads on boot
- sys_spawn() for spawning from paths

### Phase 6 Architecture (Upcoming)
- **Kernel**: Keyboard driver, framebuffer, console (rustux/)
- **Userspace**: Shell, built-ins, theming (rustica/)
- **Display**: Kernel owns framebuffer, userspace writes via syscalls
- **Input**: Keyboard IRQ → buffer → sys_read() unblocks process
- **Theming**: Userspace-owned, Dracula default

---

## 📞 Resources

- **Repository:** https://github.com/gitrustux/rustux
- **Documentation:** See ARCHITECTURE.md for kernel design details
- **Issue Tracker:** https://github.com/gitrustux/rustux/issues

**Legacy Code References:**
- `/var/www/rustux.com/prod/kernel` - Old kernel (mostly migrated to rustux/)
- `/var/www/rustux.com/prod/rustica` - Userspace OS and shell (to be refactored)

---

## 📋 Phase 6D: Rustica OS Migration ⏳ NEXT

**Overview:** Reintegrate the legacy Rustica OS tree into the new kernel infrastructure.

**Timeline:** 2-3 weeks
**Prerequisites:** Phase 6C complete (interactive shell working)
**Goal:** Make Rustica OS a proper userspace distribution running on the Rustux kernel

---

### Phase 6D.1: Architecture Clarification

**Rustica OS is no longer a "kernel-bundled OS"**
- It is a **userspace distribution** running on the Rustux kernel
- Kernel provides low-level services only
- All user-facing tools live in userspace

### Phase 6D.2: Migration Strategy

**Old Rustica userland** → becomes `/usr` + `/bin`

**Old scripts** → shell builtins or userspace programs

**Old tools** → recompiled against new syscall ABI

**Directory Structure:**
```
/prod
 ├── rustux/          # kernel only (OS-agnostic)
 ├── rustica/         # userspace OS (distribution)
 │    ├── bin/         # user programs
 │    │    ├── shell
 │    │    ├── hello
 │    │    └── counter
 │    ├── etc/
 │    │    └── theme.toml   (Dracula theme config)
 │    ├── usr/         # user data
 │    └── docs/
 └── tools/           # build tools
```

### Phase 6D.3: CLI Location

**The CLI is a userspace program, NOT part of the kernel.**

This separation ensures:
- Kernel stays OS-agnostic
- CLI can be updated independently
- GUI later will sit next to CLI, not replace it
- Multiple shells can coexist

---

## 📋 Phase 6E: Live Image First Policy 🔄

**Overview:** Shift from QEMU emulation to UEFI hardware testing as primary development target.

**Timeline:** Ongoing
**Goal:** All debugging must be framebuffer-visible or persisted to disk

---

### Phase 6E.1: QEMU is Optional

**QEMU is now optional for development only.**

Primary test target is **real UEFI hardware**.

### Phase 6E.2: Debug Requirements

All debug must be:
- **Framebuffer-visible** (see VNC display)
- Or persisted to disk
- Or LED / color-code based

This aligns with the **Silent Boot Phase** discipline - no port I/O between UEFI entry and ExitBootServices.

### Phase 6E.3: Live USB Boot

**Boot flow:**
```
UEFI firmware → BOOTX64.EFI → kernel → init → shell
```

**System boots directly into:**
- Framebuffer console with Dracula theme
- Interactive shell prompt
- No installer required (for testing)

---

## 📋 Phase 7: Minimal GUI Path ⏳ PLANNED

**Overview:** Add windowed GUI capabilities to the Rustux OS.

**Estimated Timeline:** 4-6 weeks for minimal windowed GUI, 2-3 months for Wayland-like compositor

**Prerequisites:** Phase 6 complete (shell, keyboard, framebuffer, theming)

**Goal:** Provide a basic GUI with mouse input, windows, and simple applications

---

### Phase 7A: Input + Drawing Primitives (1-2 weeks)

**Kernel additions (small):**
- Mouse driver (PS/2 or USB HID)
- New syscalls: `sys_fb_map()` → map framebuffer into userspace OR `sys_draw()` primitives (safer)

**Userspace additions:**
- Mouse cursor renderer
- Event queue (MouseMove, Click, KeyPress)
- Simple drawing primitives

**Deliverables:**
- [ ] Mouse driver working
- [ ] Framebuffer mapping syscall
- [ ] Event queue implementation
- [ ] Mouse cursor visible on screen

**At this point, you can draw and click things.**

---

### Phase 7B: Single-Process GUI Server (2-3 weeks)

**Think early Mac OS, not Wayland.**

**One userspace process runs the GUI server:**
```
rustica-gui
```

**GUI Server Responsibilities:**
- Owns framebuffer
- Owns input
- Draws:
  - Windows
  - Buttons
  - Text
- Launches apps as clients

**No IPC yet** — apps call GUI APIs directly.

**This gives you:**
- Overlapping windows
- Focus management
- A mouse pointer
- Menus

---

### Phase 7C: Client Apps (1-2 weeks)

**Apps link against:**
```
librustica_gui
```

**Library exposes:**
- `Window::new()`
- `Button::new()`
- `Label::new()`
- `on_click(...)`

**Internally:**
- Draw → framebuffer
- Events → callbacks

**This is where Rust shines.**

---

### GUI Architecture Overview

**Layering:**
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

**CLI is NOT throwaway** — it becomes:
- Fallback console
- Recovery mode
- SSH-like interface later

---

## 🎯 Phase 7 Success Criteria

Phase 7 is complete when:

- [ ] Mouse input works (PS/2 or USB HID)
- [ ] Framebuffer can be mapped into userspace
- [ ] GUI server can draw windows and buttons
- [ ] Event queue processes mouse and keyboard events
- [ ] Simple client apps can be built and linked

---

## 🔧 Development Quick Reference (Updated)

### Build Kernel
```bash
cd /var/www/rustux.com/prod/rustux/
cargo build --release --target x86_64-unknown-uefi --features uefi_kernel
```

### Build Shell (userspace C)
```bash
cd /var/www/rustux.com/prod/rustux/test-userspace
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector shell.c -o shell.elf
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector init.c -o init.elf
```

### Test with QEMU (Optional)
```bash
/usr/local/bin/qemu-system-x86_64 \
  -bios /usr/share/qemu/OVMF.fd \
  -drive format=raw,file=disk.img \
  -m 512M \
  -vnc :0
```

---

## 📝 Critical Technical Decisions (Updated)

### Phase 5 Summary
- Process table with 256 slots, indexed by PID
- Round-robin scheduler with context switching
- Ramdisk for embedded files
- Init process (PID 1) auto-loads on boot
- sys_spawn() for spawning from paths

### Phase 6 Summary (Complete)
- **Input:** PS/2 keyboard driver with scancode to ASCII conversion
- **Display:** Framebuffer driver with text console (16x8 font)
- **Shell:** Interactive C shell with Dracula theme
- **Process Management:** Multi-process scheduler, VFS abstraction
- **Syscalls:** read, write, open, close, lseek, spawn, exit, getpid, getppid, yield

### Phase 7 Architecture (Planned)
- **GUI Server:** Single userspace process owning framebuffer
- **Input:** Mouse + keyboard events
- **Drawing:** Framebuffer mapping or draw syscalls
- **Apps:** Link against librustica_gui

**Key Design Decision:**
- CLI remains primary interface (recovery, debugging)
- GUI is additive, not replacement
- Theme consistency across CLI and GUI (Dracula)

---

## 📞 Resources

- **Repository:** https://github.com/gitrustux/rustux
- **Documentation:** See ARCHITECTURE.md for kernel design details
- **Issue Tracker:** https://github.com/gitrustux/rustux/issues

**Project Locations:**
- **Kernel:** `/var/www/rustux.com/prod/rustux/`
- **Userspace/OS:** `/var/www/rustux.com/prod/rustica/`
- **Legacy Kernel:** `/var/www/rustux.com/prod/kernel` (deprecated, migrated to rustux/)

---

**END OF PLAN**
Last Updated: 2025-01-23 - Phase 5 COMPLETE, Phase 6A-6C COMPLETE, Phase 6D-7 PLANNED

**About the Old Installer:**
`/var/www/rustux.com/prod/installer/` was for the OLD Linux-based Rustica OS and is now OBSOLETE.
It used GRUB + Linux kernel + initramfs. The new Rustux kernel is a standalone UEFI
application that boots directly without GRUB. Do not use the old installer for the new kernel.
