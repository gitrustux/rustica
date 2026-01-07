// Copyright 2025 The Rustux Authors
//
// Rustica Text Editor (redit)
// A modern text editor with syntax highlighting and tabs

#![no_std]

extern crate alloc;

use alloc::string::String;
use core::arch::asm;

#[no_mangle]
pub extern "C" fn main() -> i32 {
    // TODO: Implement redit text editor
    let message = "redit - Rustica Text Editor\nComing soon!";
    
    // Syscall: write(1, message, len)
    unsafe {
        let msg_ptr = message.as_ptr() as usize;
        let msg_len = message.len() as usize;
        let fd: usize = 1; // stdout
        
        asm!(
            "syscall",
            in("rax") 1, // sys_write
            in("rdi") fd,
            in("rsi") msg_ptr,
            in("rdx") msg_len,
            lateout("rcx") _,
            lateout("r11") _,
            lateout("rax") result,
        );
    }
    
    0
}
