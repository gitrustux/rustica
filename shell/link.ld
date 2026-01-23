/* Copyright 2025 The Rustux Authors
 *
 * Use of this source code is governed by a MIT-style
 * license that can be found in the LICENSE file or at
 * https://opensource.org/licenses/MIT
 */

/* Rustux Shell Linker Script */

ENTRY(main);

SECTIONS {
    /* Text section (code) */
    .text : {
        *(.text.init);
        *(.text .text.*);
    }

    /* Read-only data section */
    .rodata : {
        *(.rodata .rodata.*);
    }

    /* Data section (initialized data) */
    .data : {
        *(.data .data.*);
    }

    /* BSS section (uninitialized data) */
    .bss : {
        *(.bss .bss.*);
        *(COMMON);
    }

    /* Stack */
    .stack : {
        __stack_start = .;
        . = . + 0x4000; /* 16KB stack */
        __stack_end = .;
    }

    /* Discard unwind tables */
    /DISCARD/ : {
        *(.eh_frame);
        *(.note.gnu.build-id);
    }
}
