// Copyright 2025 The Rustux Authors
//
// Hello World Example
// Basic Rustica application

#![no_std]

extern crate alloc;

use alloc::string::ToString;

#[no_mangle]
pub extern "C" fn main() -> i32 {
    let message = "Hello, Rustica OS!\n";

    // Write to stdout using syscall
    unsafe {
        let msg_ptr = message.as_ptr() as usize;
        let msg_len = message.len() as usize;
        let fd: usize = 1; // stdout

        core::arch::asm!(
            "syscall",
            in("rax") 1, // sys_write
            in("rdi") fd,
            in("rsi") msg_ptr,
            in("rdx") msg_len,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("rax") _,
        );
    }

    0
}
