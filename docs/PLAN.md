# Rustux OS - Phase 7: Minimal GUI Implementation

**Last Updated:** 2025-01-25
**Current Status:** Phase 6A-6C COMPLETE, Phase 7A IN PROGRESS (USB HID Keyboard)
**Project Locations:**
- Kernel: `/var/www/rustux.com/prod/rustux/`
- Userspace/OS: `/var/www/rustux.com/prod/rustica/`
- UEFI Loader: `/var/www/rustux.com/prod/loader/kernel-efi/`

---

## Table of Contents

1. [Overview](#overview)
2. [Current Status](#current-status)
3. [Phase 6: Remaining Work](#phase-6-remaining-work)
4. [Phase 7: GUI Implementation](#phase-7-gui-implementation)
5. [Success Criteria](#success-criteria)
6. [Testing & Validation](#testing--validation)
7. [Development Workflow](#development-workflow)

---

## Overview

Transform Rustux from a text-based system into a graphical operating system with mouse input, windowing, and basic GUI applications - all running directly on the Rustux microkernel without any Linux dependencies.

**Timeline:** 8-10 weeks
**Prerequisites:** Phase 6 complete (interactive shell, keyboard, framebuffer console, multi-process scheduler)
**Goal:** Boot to a functional GUI desktop with window management, mouse cursor, and launchable GUI applications

**Design Philosophy:**
- Early Mac OS / AmigaOS style (single-process window manager)
- Direct framebuffer rendering (no X11/Wayland complexity)
- Dracula theme mandatory across all GUI elements
- CLI remains accessible (Ctrl+Alt+F1 to switch back)

---

## Current Status

### ✅ Completed: Phase 6A-6C (Interactive Shell)

| Component | Status | Notes |
|-----------|--------|-------|
| **Direct UEFI Boot** | ✅ Complete | No GRUB, standalone BOOTX64.EFI |
| **PS/2 Keyboard Driver** | ✅ Complete | IRQ1, scancode-to-ASCII, modifiers |
| **USB HID Keyboard** | ⚠️ In Progress | xHCI controller, polling-based (Phase 7A) |
| **Framebuffer Console** | ✅ Complete | RGB565, PSF2 font (8x16), scrolling |
| **Process Management** | ✅ Complete | 256-slot table, round-robin scheduler |
| **Syscall Interface** | ✅ Complete | read, write, spawn, exit, getpid, yield |
| **VFS + Ramdisk** | ✅ Complete | Embedded ELF binaries (init, shell, hello) |
| **Interactive Shell** | ✅ Complete | C shell with Dracula theme, built-in commands |

### ⏳ In Progress: Phase 6D-6E

| Component | Status | Notes |
|-----------|--------|-------|
| **6D: Stability & UX** | Pending | Error handling, non-regression testing |
| **6E: Live Boot Media** | Pending | FAT32 ESP, embedded initramfs |

### ⏳ Planned: Phase 7 (GUI)

| Component | Status | Notes |
|-----------|--------|-------|
| **7A: Mouse Input & Cursor** | Pending | PS/2 or USB HID mouse driver |
| **7B: Window Manager** | Pending | Window abstraction, rendering, focus |
| **7C: GUI Syscalls & Library** | Pending | librustica_gui client library |
| **7D: Desktop Shell** | Pending | Desktop background, panel, app launcher |

---

## Phase 6: Remaining Work

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

---

## Phase 7: GUI Implementation

### Phase 7A: Mouse Input & Cursor (Week 1-2)

#### 7A.1: PS/2 Mouse Driver

**Goal:** Read mouse movement and clicks from hardware

**Files to create:**
```
src/drivers/mouse/ps2_mouse.rs - PS/2 mouse controller driver
src/drivers/mouse/mod.rs - Mouse abstraction layer
```

**Implementation:**

```rust
// src/drivers/mouse/ps2_mouse.rs

const MOUSE_DATA_PORT: u16 = 0x60;
const MOUSE_STATUS_PORT: u16 = 0x64;
const MOUSE_COMMAND_PORT: u16 = 0x64;

pub struct PS2Mouse {
    x: i32,
    y: i32,
    buttons: u8,  // bit 0: left, bit 1: right, bit 2: middle
    packet_buffer: [u8; 3],
    packet_index: usize,
}

impl PS2Mouse {
    pub fn init() -> Result<Self, &'static str> {
        // Enable mouse device (0xA8 command)
        Self::send_command(0xA8)?;

        // Enable IRQ12 (mouse interrupt)
        Self::send_command(0x20)?;
        let mut status = Self::read_data()?;
        status |= 0x02;  // Enable IRQ12
        Self::send_command(0x60)?;
        Self::send_data(status)?;

        // Set defaults for mouse
        Self::send_mouse_command(0xF6)?;

        // Enable data reporting
        Self::send_mouse_command(0xF4)?;

        Ok(Self {
            x: 0,
            y: 0,
            buttons: 0,
            packet_buffer: [0; 3],
            packet_index: 0,
        })
    }

    /// Handle mouse interrupt (IRQ 12)
    pub fn handle_interrupt(&mut self) {
        if !Self::has_data() {
            return;
        }

        let byte = unsafe { x86::io::inb(MOUSE_DATA_PORT) };

        // PS/2 mouse sends 3-byte packets
        self.packet_buffer[self.packet_index] = byte;
        self.packet_index += 1;

        if self.packet_index >= 3 {
            self.process_packet();
            self.packet_index = 0;
        }
    }

    fn process_packet(&mut self) {
        let flags = self.packet_buffer[0];
        let dx = self.packet_buffer[1] as i8 as i32;
        let dy = self.packet_buffer[2] as i8 as i32;

        // Update position (invert Y for screen coordinates)
        self.x += dx;
        self.y -= dy;

        // Clamp to screen bounds
        self.x = self.x.clamp(0, 1024);  // TODO: Get actual screen width
        self.y = self.y.clamp(0, 768);   // TODO: Get actual screen height

        // Update button state
        self.buttons = flags & 0x07;
    }

    pub fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn buttons(&self) -> (bool, bool, bool) {
        (
            self.buttons & 0x01 != 0,  // Left
            self.buttons & 0x02 != 0,  // Right
            self.buttons & 0x04 != 0,  // Middle
        )
    }

    fn send_command(cmd: u8) -> Result<(), &'static str> {
        Self::wait_for_write()?;
        unsafe { x86::io::outb(MOUSE_COMMAND_PORT, cmd); }
        Ok(())
    }

    fn send_data(data: u8) -> Result<(), &'static str> {
        Self::wait_for_write()?;
        unsafe { x86::io::outb(MOUSE_DATA_PORT, data); }
        Ok(())
    }

    fn send_mouse_command(cmd: u8) -> Result<(), &'static str> {
        // Tell controller we're sending mouse command
        Self::send_command(0xD4)?;
        Self::send_data(cmd)?;

        // Wait for ACK (0xFA)
        let ack = Self::read_data()?;
        if ack != 0xFA {
            return Err("Mouse did not ACK command");
        }
        Ok(())
    }

    fn read_data() -> Result<u8, &'static str> {
        Self::wait_for_read()?;
        Ok(unsafe { x86::io::inb(MOUSE_DATA_PORT) })
    }

    fn has_data() -> bool {
        unsafe { (x86::io::inb(MOUSE_STATUS_PORT) & 0x01) != 0 }
    }

    fn wait_for_read() -> Result<(), &'static str> {
        for _ in 0..10000 {
            if Self::has_data() {
                return Ok(());
            }
        }
        Err("Mouse read timeout")
    }

    fn wait_for_write() -> Result<(), &'static str> {
        for _ in 0..10000 {
            let status = unsafe { x86::io::inb(MOUSE_STATUS_PORT) };
            if (status & 0x02) == 0 {
                return Ok(());
            }
        }
        Err("Mouse write timeout")
    }
}
```

**Interrupt handler registration:**

```rust
// src/arch/amd64/interrupts/mouse.rs

static MOUSE: SpinMutex<Option<PS2Mouse>> = SpinMutex::new(None);

extern "C" fn mouse_interrupt_handler() {
    if let Some(ref mut mouse) = *MOUSE.lock() {
        mouse.handle_interrupt();

        // Wake GUI process if blocked waiting for events
        gui::notify_mouse_event();
    }

    apic::apic_eoi();
}

pub fn init() {
    // Initialize mouse driver
    let mouse = PS2Mouse::init().expect("Failed to initialize PS/2 mouse");
    *MOUSE.lock() = Some(mouse);

    // Register IRQ 12 (mouse) as interrupt vector 44
    unsafe {
        idt::idt_set_gate(44, mouse_interrupt_handler as u64, 0x08, 0x8E);
    }

    // Enable IRQ 12 in APIC
    apic::apic_io_init(12, 44);
}

pub fn get_position() -> (i32, i32) {
    MOUSE.lock().as_ref()
        .map(|m| m.position())
        .unwrap_or((0, 0))
}

pub fn get_buttons() -> (bool, bool, bool) {
    MOUSE.lock().as_ref()
        .map(|m| m.buttons())
        .unwrap_or((false, false, false))
}
```

**Tests:**
- Initialize mouse, verify no errors
- Move mouse, verify position updates
- Click buttons, verify state changes
- Rapid movement, verify no packet loss

**Deliverable:** Mouse position and button state accessible from kernel

---

#### 7A.2: Mouse Cursor Rendering

**Goal:** Draw mouse cursor on framebuffer

**Files to create:**
```
src/drivers/display/cursor.rs - Cursor rendering
```

**Implementation:**

```rust
// src/drivers/display/cursor.rs

use crate::drivers::display::{Framebuffer, Color};

const CURSOR_WIDTH: usize = 16;
const CURSOR_HEIGHT: usize = 24;

// Arrow cursor bitmap (1 = foreground, 0 = transparent)
const CURSOR_BITMAP: [u16; CURSOR_HEIGHT] = [
    0b1000000000000000,
    0b1100000000000000,
    0b1110000000000000,
    0b1111000000000000,
    0b1111100000000000,
    0b1111110000000000,
    0b1111111000000000,
    0b1111111100000000,
    0b1111111110000000,
    0b1111111111000000,
    0b1111111111100000,
    0b1111111111110000,
    0b1111110000000000,
    0b1111000000000000,
    0b1100000000000000,
    0b1000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
];

pub struct Cursor {
    saved_bg: [u32; CURSOR_WIDTH * CURSOR_HEIGHT],
    last_x: usize,
    last_y: usize,
    visible: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            saved_bg: [0; CURSOR_WIDTH * CURSOR_HEIGHT],
            last_x: 0,
            last_y: 0,
            visible: false,
        }
    }

    /// Draw cursor at position, saving background
    pub fn draw(&mut self, fb: &mut Framebuffer, x: usize, y: usize) {
        // Erase old cursor if visible
        if self.visible {
            self.erase(fb);
        }

        // Save background
        for dy in 0..CURSOR_HEIGHT {
            for dx in 0..CURSOR_WIDTH {
                let px = x + dx;
                let py = y + dy;

                if px < fb.width && py < fb.height {
                    let idx = dy * CURSOR_WIDTH + dx;
                    self.saved_bg[idx] = fb.get_pixel(px, py);
                }
            }
        }

        // Draw cursor (Dracula colors)
        let fg = Color::from_rgb(248, 248, 242);  // Dracula foreground
        let outline = Color::from_rgb(0, 0, 0);   // Black outline

        for dy in 0..CURSOR_HEIGHT {
            let row = CURSOR_BITMAP[dy];

            for dx in 0..CURSOR_WIDTH {
                let px = x + dx;
                let py = y + dy;

                if px >= fb.width || py >= fb.height {
                    continue;
                }

                let bit = (row >> (15 - dx)) & 1;

                if bit != 0 {
                    // Draw outline
                    if dx > 0 && dy > 0 {
                        fb.put_pixel(px - 1, py, outline);
                        fb.put_pixel(px, py - 1, outline);
                    }

                    // Draw cursor
                    fb.put_pixel(px, py, fg);
                }
            }
        }

        self.last_x = x;
        self.last_y = y;
        self.visible = true;
    }

    /// Erase cursor by restoring background
    fn erase(&mut self, fb: &mut Framebuffer) {
        if !self.visible {
            return;
        }

        for dy in 0..CURSOR_HEIGHT {
            for dx in 0..CURSOR_WIDTH {
                let px = self.last_x + dx;
                let py = self.last_y + dy;

                if px < fb.width && py < fb.height {
                    let idx = dy * CURSOR_WIDTH + dx;
                    fb.put_pixel_raw(px, py, self.saved_bg[idx]);
                }
            }
        }

        self.visible = false;
    }
}
```

**Integration:**

```rust
// src/drivers/display/mod.rs

static CURSOR: SpinMutex<Cursor> = SpinMutex::new(Cursor::new());

pub fn update_cursor() {
    let (x, y) = mouse::get_position();
    let mut fb = FRAMEBUFFER.lock();
    let mut cursor = CURSOR.lock();

    cursor.draw(&mut fb, x as usize, y as usize);
}

// Call from timer interrupt
pub fn on_timer_tick() {
    update_cursor();
}
```

**Tests:**
- Move mouse, verify cursor follows smoothly
- Verify no screen tearing
- Verify background restoration works
- Test cursor at screen edges

**Deliverable:** Visible mouse cursor that tracks hardware mouse

---

#### 7A.3: Input Event Queue

**Goal:** Unified event system for mouse + keyboard

**Files to create:**
```
src/gui/events.rs - Event queue and types
```

**Implementation:**

```rust
// src/gui/events.rs

use alloc::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    MouseMove { x: i32, y: i32 },
    MouseDown { x: i32, y: i32, button: MouseButton },
    MouseUp { x: i32, y: i32, button: MouseButton },
    KeyPress { key: char, scancode: u8 },
    KeyRelease { scancode: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub struct EventQueue {
    events: VecDeque<Event>,
    max_size: usize,
}

impl EventQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, event: Event) {
        if self.events.len() >= self.max_size {
            // Drop oldest event
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

static EVENT_QUEUE: SpinMutex<EventQueue> = SpinMutex::new(EventQueue::new(256));

/// Called from mouse interrupt handler
pub fn notify_mouse_move(x: i32, y: i32) {
    EVENT_QUEUE.lock().push(Event::MouseMove { x, y });
}

pub fn notify_mouse_down(x: i32, y: i32, button: MouseButton) {
    EVENT_QUEUE.lock().push(Event::MouseDown { x, y, button });
}

pub fn notify_mouse_up(x: i32, y: i32, button: MouseButton) {
    EVENT_QUEUE.lock().push(Event::MouseUp { x, y, button });
}

/// Called from keyboard interrupt handler
pub fn notify_key_press(key: char, scancode: u8) {
    EVENT_QUEUE.lock().push(Event::KeyPress { key, scancode });
}

pub fn notify_key_release(scancode: u8) {
    EVENT_QUEUE.lock().push(Event::KeyRelease { scancode });
}

/// Get next event (non-blocking)
pub fn poll_event() -> Option<Event> {
    EVENT_QUEUE.lock().pop()
}
```

**Tests:**
- Push 300 events, verify oldest are dropped
- Poll empty queue, verify returns None
- Mouse + keyboard events interleaved correctly

**Deliverable:** Unified event system ready for GUI applications

---

### Phase 7B: GUI Server & Window Manager (Week 3-5)

#### 7B.1: Window Abstraction

**Goal:** Define window structure and management

**Files to create:**
```
src/gui/window.rs - Window data structures
src/gui/manager.rs - Window manager
```

**Implementation:**

```rust
// src/gui/window.rs

use alloc::string::String;
use crate::drivers::display::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width as i32 &&
        y >= self.y && y < self.y + self.height as i32
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32 &&
        self.x + self.width as i32 > other.x &&
        self.y < other.y + other.height as i32 &&
        self.y + self.height as i32 > other.y
    }
}

pub struct Window {
    pub id: u32,
    pub title: String,
    pub bounds: Rect,
    pub visible: bool,
    pub focused: bool,
    pub pid: u32,  // Owning process
    pub flags: WindowFlags,
}

bitflags::bitflags! {
    pub struct WindowFlags: u32 {
        const RESIZABLE = 0x01;
        const CLOSABLE = 0x02;
        const MINIMIZABLE = 0x04;
        const HAS_TITLEBAR = 0x08;
    }
}

impl Window {
    pub fn new(id: u32, title: String, bounds: Rect, pid: u32) -> Self {
        Self {
            id,
            title,
            bounds,
            visible: true,
            focused: false,
            pid,
            flags: WindowFlags::CLOSABLE | WindowFlags::HAS_TITLEBAR,
        }
    }

    /// Get titlebar rectangle
    pub fn titlebar_rect(&self) -> Rect {
        if self.flags.contains(WindowFlags::HAS_TITLEBAR) {
            Rect {
                x: self.bounds.x,
                y: self.bounds.y,
                width: self.bounds.width,
                height: 24,  // Titlebar height
            }
        } else {
            Rect { x: 0, y: 0, width: 0, height: 0 }
        }
    }

    /// Get client area (content without titlebar)
    pub fn client_rect(&self) -> Rect {
        if self.flags.contains(WindowFlags::HAS_TITLEBAR) {
            Rect {
                x: self.bounds.x,
                y: self.bounds.y + 24,
                width: self.bounds.width,
                height: self.bounds.height.saturating_sub(24),
            }
        } else {
            self.bounds
        }
    }
}
```

**Window Manager:**

```rust
// src/gui/manager.rs

use alloc::vec::Vec;
use crate::gui::{Window, Rect, Event};

pub struct WindowManager {
    windows: Vec<Window>,
    next_id: u32,
    focused_window: Option<u32>,
    dragging: Option<DragState>,
}

struct DragState {
    window_id: u32,
    start_x: i32,
    start_y: i32,
    window_start_x: i32,
    window_start_y: i32,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            focused_window: None,
            dragging: None,
        }
    }

    pub fn create_window(&mut self, title: String, bounds: Rect, pid: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let window = Window::new(id, title, bounds, pid);
        self.windows.push(window);

        // Focus new window
        self.set_focus(id);

        id
    }

    pub fn destroy_window(&mut self, id: u32) {
        self.windows.retain(|w| w.id != id);

        if self.focused_window == Some(id) {
            self.focused_window = self.windows.last().map(|w| w.id);
        }
    }

    pub fn set_focus(&mut self, id: u32) {
        // Unfocus all
        for window in &mut self.windows {
            window.focused = false;
        }

        // Focus target and bring to front
        if let Some(idx) = self.windows.iter().position(|w| w.id == id) {
            self.windows[idx].focused = true;

            // Move to end (front of Z-order)
            let window = self.windows.remove(idx);
            self.windows.push(window);

            self.focused_window = Some(id);
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::MouseDown { x, y, button } if *button == MouseButton::Left => {
                // Check titlebar clicks (for dragging)
                for window in self.windows.iter().rev() {
                    if window.titlebar_rect().contains(*x, *y) {
                        self.set_focus(window.id);

                        // Start drag
                        self.dragging = Some(DragState {
                            window_id: window.id,
                            start_x: *x,
                            start_y: *y,
                            window_start_x: window.bounds.x,
                            window_start_y: window.bounds.y,
                        });

                        return;
                    }
                }

                // Check window clicks
                for window in self.windows.iter().rev() {
                    if window.bounds.contains(*x, *y) {
                        self.set_focus(window.id);
                        return;
                    }
                }
            }

            Event::MouseUp { .. } => {
                self.dragging = None;
            }

            Event::MouseMove { x, y } => {
                if let Some(ref drag) = self.dragging {
                    // Update window position
                    let dx = *x - drag.start_x;
                    let dy = *y - drag.start_y;

                    if let Some(window) = self.windows.iter_mut().find(|w| w.id == drag.window_id) {
                        window.bounds.x = drag.window_start_x + dx;
                        window.bounds.y = drag.window_start_y + dy;
                    }
                }
            }

            _ => {}
        }
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }
}
```

**Tests:**
- Create window, verify ID returned
- Destroy window, verify removed
- Click window, verify focus changes
- Drag titlebar, verify window moves

**Deliverable:** Window manager with focus and drag support

---

#### 7B.2: Window Rendering

**Goal:** Draw windows with Dracula theme

**Implementation:**

```rust
// src/gui/renderer.rs

use crate::drivers::display::{Framebuffer, Color};
use crate::gui::{Window, WindowManager};

// Dracula theme colors
const BG_DEFAULT: Color = Color::from_rgb(40, 42, 54);
const FG_DEFAULT: Color = Color::from_rgb(248, 248, 242);
const PURPLE: Color = Color::from_rgb(189, 147, 249);
const CYAN: Color = Color::from_rgb(139, 233, 253);
const SELECTION: Color = Color::from_rgb(68, 71, 90);

pub struct Renderer {
    dirty: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self { dirty: true }
    }

    pub fn render(&mut self, fb: &mut Framebuffer, wm: &WindowManager) {
        if !self.dirty {
            return;
        }

        // Clear background
        fb.clear(BG_DEFAULT);

        // Render windows back-to-front
        for window in wm.windows() {
            if window.visible {
                self.render_window(fb, window);
            }
        }

        self.dirty = false;
    }

    fn render_window(&self, fb: &mut Framebuffer, window: &Window) {
        let bounds = &window.bounds;

        // Draw titlebar
        if window.flags.contains(WindowFlags::HAS_TITLEBAR) {
            let titlebar = window.titlebar_rect();
            let titlebar_color = if window.focused { PURPLE } else { SELECTION };

            fb.fill_rect(
                titlebar.x as usize,
                titlebar.y as usize,
                titlebar.width as usize,
                titlebar.height as usize,
                titlebar_color
            );

            // Draw title text
            fb.draw_text(
                &window.title,
                (titlebar.x + 8) as usize,
                (titlebar.y + 4) as usize,
                FG_DEFAULT,
                titlebar_color
            );

            // Draw close button (X)
            let close_x = (titlebar.x + titlebar.width as i32 - 20) as usize;
            let close_y = (titlebar.y + 4) as usize;
            fb.draw_text("×", close_x, close_y, FG_DEFAULT, titlebar_color);
        }

        // Draw client area background
        let client = window.client_rect();
        fb.fill_rect(
            client.x as usize,
            client.y as usize,
            client.width as usize,
            client.height as usize,
            Color::from_rgb(30, 32, 40)  // Slightly darker than BG_DEFAULT
        );

        // Draw window border
        self.draw_border(fb, bounds, if window.focused { CYAN } else { SELECTION });
    }

    fn draw_border(&self, fb: &mut Framebuffer, bounds: &Rect, color: Color) {
        let x = bounds.x as usize;
        let y = bounds.y as usize;
        let w = bounds.width as usize;
        let h = bounds.height as usize;

        // Top
        fb.fill_rect(x, y, w, 2, color);
        // Bottom
        fb.fill_rect(x, y + h - 2, w, 2, color);
        // Left
        fb.fill_rect(x, y, 2, h, color);
        // Right
        fb.fill_rect(x + w - 2, y, 2, h, color);
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}
```

**Tests:**
- Render single window, verify titlebar + border
- Render overlapping windows, verify Z-order correct
- Focus different windows, verify color changes
- Drag window, verify redraws correctly

**Deliverable:** Windows render with Dracula-themed titlebars and borders

---

### Phase 7C: GUI Syscalls & Client Library (Week 6-8)

#### 7C.1: GUI Syscalls

**Goal:** Expose GUI functionality to userspace

**Syscalls to implement:**

```sys_window_create(title: *const u8, title_len: usize, x: i32, y: i32, width: u32, height: u32) -> i32```

Returns window ID or -1 on error.

```rust
// src/syscall/gui.rs

pub fn sys_window_create(
    title: *const u8,
    title_len: usize,
    x: i32,
    y: i32,
    width: u32,
    height: u32
) -> i32 {
    let current = current_process();

    // Validate title pointer
    if !current.address_space.is_user_address(title as u64, title_len) {
        return -1;
    }

    // Copy title from userspace
    let mut kernel_title = String::with_capacity(title_len);
    unsafe {
        for i in 0..title_len {
            kernel_title.push(*title.add(i) as char);
        }
    }

    // Create window
    let bounds = Rect { x, y, width, height };
    let window_id = GUI_MANAGER.lock().create_window(kernel_title, bounds, current.pid);

    window_id as i32
}
```

**Additional syscalls:**
- `sys_window_destroy(window_id: u32) -> i32`
- `sys_window_draw_rect(window_id: u32, x: i32, y: i32, width: u32, height: u32, color: u32) -> i32`
- `sys_window_draw_text(window_id: u32, text: *const u8, text_len: usize, x: i32, y: i32, color: u32) -> i32`
- `sys_poll_event(event_buf: *mut Event) -> i32` - Returns 1 if event available, 0 if none, -1 on error

**Syscall numbers:**
```rust
const SYS_WINDOW_CREATE: usize = 100;
const SYS_WINDOW_DESTROY: usize = 101;
const SYS_WINDOW_DRAW_RECT: usize = 102;
const SYS_WINDOW_DRAW_TEXT: usize = 103;
const SYS_POLL_EVENT: usize = 104;
```

**Tests:**
- Create window from userspace, verify ID returned
- Draw shapes, verify they appear
- Poll events, verify mouse/keyboard events received

**Deliverable:** Userspace can create windows and draw into them

---

#### 7C.2: Client Library (librustica_gui)

**Goal:** Rust library for building GUI apps

**Files to create:**
```
userspace/librustica_gui/src/lib.rs
userspace/librustica_gui/src/window.rs
userspace/librustica_gui/src/widgets.rs
```

**Implementation:**

```rust
// userspace/librustica_gui/src/lib.rs

#![no_std]

extern crate alloc;

pub mod window;
pub mod widgets;
pub mod events;

pub use window::Window;
pub use widgets::{Button, Label};
pub use events::Event;

// Syscall wrappers
mod syscalls {
    use super::Event;

    const SYS_WINDOW_CREATE: usize = 100;
    const SYS_WINDOW_DESTROY: usize = 101;
    const SYS_WINDOW_DRAW_RECT: usize = 102;
    const SYS_WINDOW_DRAW_TEXT: usize = 103;
    const SYS_POLL_EVENT: usize = 104;

    pub fn window_create(title: &str, x: i32, y: i32, width: u32, height: u32) -> i32 {
        unsafe {
            syscall!(
                SYS_WINDOW_CREATE,
                title.as_ptr(),
                title.len(),
                x,
                y,
                width,
                height
            ) as i32
        }
    }

    pub fn window_destroy(id: u32) -> i32 {
        unsafe { syscall!(SYS_WINDOW_DESTROY, id) as i32 }
    }

    pub fn draw_rect(id: u32, x: i32, y: i32, w: u32, h: u32, color: u32) -> i32 {
        unsafe { syscall!(SYS_WINDOW_DRAW_RECT, id, x, y, w, h, color) as i32 }
    }

    pub fn draw_text(id: u32, text: &str, x: i32, y: i32, color: u32) -> i32 {
        unsafe {
            syscall!(SYS_WINDOW_DRAW_TEXT, id, text.as_ptr(), text.len(), x, y, color) as i32
        }
    }

    pub fn poll_event() -> Option<Event> {
        let mut event = core::mem::MaybeUninit::<Event>::uninit();
        let result = unsafe {
            syscall!(SYS_POLL_EVENT, event.as_mut_ptr()) as i32
        };

        if result == 1 {
            Some(unsafe { event.assume_init() })
        } else {
            None
        }
    }
}
```

**Window wrapper:**

```rust
// userspace/librustica_gui/src/window.rs

use alloc::string::String;
use crate::syscalls;

pub struct Window {
    id: u32,
    width: u32,
    height: u32,
}

impl Window {
    pub fn new(title: &str, x: i32, y: i32, width: u32, height: u32) -> Result<Self, &'static str> {
        let id = syscalls::window_create(title, x, y, width, height);

        if id < 0 {
            return Err("Failed to create window");
        }

        Ok(Self {
            id: id as u32,
            width,
            height,
        })
    }

    pub fn fill_rect(&self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        syscalls::draw_rect(self.id, x, y, w, h, color);
    }

    pub fn draw_text(&self, text: &str, x: i32, y: i32, color: u32) {
        syscalls::draw_text(self.id, text, x, y, color);
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        syscalls::window_destroy(self.id);
    }
}
```

**Widget abstractions:**

```rust
// userspace/librustica_gui/src/widgets.rs

use crate::Window;

pub struct Button {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    label: &'static str,
}

impl Button {
    pub fn new(x: i32, y: i32, width: u32, height: u32, label: &'static str) -> Self {
        Self { x, y, width, height, label }
    }

    pub fn draw(&self, window: &Window) {
        // Dracula purple button
        let bg_color = 0xBD93F9;  // Purple
        let fg_color = 0xF8F8F2;  // Foreground

        // Draw button background
        window.fill_rect(self.x, self.y, self.width, self.height, bg_color);

        // Draw label (centered)
        let text_x = self.x + (self.width as i32 / 2) - (self.label.len() as i32 * 4);
        let text_y = self.y + (self.height as i32 / 2) - 8;
        window.draw_text(self.label, text_x, text_y, fg_color);
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width as i32 &&
        y >= self.y && y < self.y + self.height as i32
    }
}

pub struct Label {
    x: i32,
    y: i32,
    text: &'static str,
}

impl Label {
    pub fn new(x: i32, y: i32, text: &'static str) -> Self {
        Self { x, y, text }
    }

    pub fn draw(&self, window: &Window) {
        let fg_color = 0xF8F8F2;  // Dracula foreground
        window.draw_text(self.text, self.x, self.y, fg_color);
    }
}
```

**Tests:**
- Create window using library, verify it appears
- Draw button, verify it renders correctly
- Click button, verify hit testing works

**Deliverable:** Rust library for building GUI applications

---

#### 7C.3: Example GUI Application

**Goal:** Demonstrate GUI capabilities

**File:** `userspace/gui-demo/src/main.rs`

```rust
#![no_std]
#![no_main]

extern crate librustica_gui;
extern crate alloc;

use librustica_gui::{Window, Button, Label, Event, MouseButton};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize heap
    init_heap();

    // Create window
    let window = Window::new("Rustux GUI Demo", 100, 100, 400, 300)
        .expect("Failed to create window");

    // Create widgets
    let label = Label::new(20, 40, "Welcome to Rustux!");
    let button = Button::new(150, 100, 100, 40, "Click Me");
    let mut clicked = false;

    // Event loop
    loop {
        // Handle events
        while let Some(event) = librustica_gui::poll_event() {
            match event {
                Event::MouseDown { x, y, button: MouseButton::Left } => {
                    // Convert to window coordinates
                    let wx = x - 100;
                    let wy = y - 124;  // Account for titlebar

                    if button.contains(wx, wy) {
                        clicked = !clicked;
                    }
                }
                _ => {}
            }
        }

        // Render
        window.fill_rect(0, 0, 400, 300, 0x282A36);  // Dracula background

        label.draw(&window);
        button.draw(&window);

        if clicked {
            let status = Label::new(20, 160, "Button was clicked!");
            status.draw(&window);
        }

        // Yield CPU
        librustica_gui::yield_cpu();
    }
}

fn init_heap() {
    // Use sys_mmap to allocate heap
    // (Implementation from Phase 5)
}
```

**Build:**
```bash
cd userspace/gui-demo
x86_64-linux-gnu-gcc -static -nostdlib -fno-stack-protector \
    -I../librustica_gui/include \
    -L../librustica_gui/target/release \
    -lrustica_gui \
    src/main.rs -o gui-demo.elf
```

**Tests:**
- Boot to GUI, verify window appears
- Click button, verify state changes
- Drag window, verify it moves
- Close window, verify app exits

**Deliverable:** Working GUI demo application

---

### Phase 7D: Desktop Shell (Week 9-10)

#### 7D.1: Desktop Background & Panel

**Goal:** Desktop environment with taskbar

**Implementation:**

```rust
// userspace/desktop/src/main.rs

#![no_std]
#![no_main]

use librustica_gui::{Window, Event};

const PANEL_HEIGHT: u32 = 32;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_heap();

    // Create desktop background (fullscreen)
    let desktop = Window::new("Desktop", 0, 0, 1024, 768)
        .expect("Failed to create desktop");

    // Draw background (Dracula BG_DEFAULT)
    desktop.fill_rect(0, 0, 1024, 768, 0x282A36);

    // Create panel (top bar)
    let panel = Window::new("Panel", 0, 0, 1024, PANEL_HEIGHT)
        .expect("Failed to create panel");

    // Draw panel (Dracula SELECTION)
    panel.fill_rect(0, 0, 1024, PANEL_HEIGHT, 0x44475A);

    // Draw clock
    panel.draw_text("12:00", 950, 8, 0xF8F8F2);

    // Draw app launcher button
    panel.draw_text("≡ Apps", 10, 8, 0xF8F8F2);

    loop {
        // Handle panel clicks
        while let Some(event) = librustica_gui::poll_event() {
            handle_panel_click(&event, &panel);
        }

        librustica_gui::yield_cpu();
    }
}

fn handle_panel_click(event: &Event, panel: &Window) {
    if let Event::MouseDown { x, y, .. } = event {
        // Check if click is on "Apps" button
        if *x >= 10 && *x <= 100 && *y >= 8 && *y <= 24 {
            launch_app_menu();
        }
    }
}

fn launch_app_menu() {
    // Spawn app launcher process
    librustica_gui::spawn("/bin/app-launcher.elf");
}
```

**Tests:**
- Boot to desktop, verify background + panel appear
- Click Apps button, verify menu launches
- Verify clock updates every minute

**Deliverable:** Basic desktop shell with panel

---

#### 7D.2: Application Launcher

**Goal:** Menu for launching GUI apps

**Implementation:**

```rust
// userspace/app-launcher/src/main.rs

#![no_std]
#![no_main]

use librustica_gui::{Window, Button, Event, MouseButton};

const APPS: &[(&str, &str)] = &[
    ("Terminal", "/bin/shell.elf"),
    ("GUI Demo", "/bin/gui-demo.elf"),
    ("Text Editor", "/bin/editor.elf"),
];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let window = Window::new("Applications", 10, 40, 200, 300)
        .expect("Failed to create launcher");

    let mut buttons = Vec::new();
    for (i, (name, _path)) in APPS.iter().enumerate() {
        let button = Button::new(10, 10 + i as i32 * 50, 180, 40, name);
        buttons.push(button);
    }

    loop {
        while let Some(event) = librustica_gui::poll_event() {
            if let Event::MouseDown { x, y, button: MouseButton::Left } = event {
                for (i, btn) in buttons.iter().enumerate() {
                    if btn.contains(x - 10, y - 64) {  // Adjust for window position
                        launch_app(APPS[i].1);
                    }
                }
            }
        }

        // Render
        window.fill_rect(0, 0, 200, 300, 0x282A36);
        for button in &buttons {
            button.draw(&window);
        }

        librustica_gui::yield_cpu();
    }
}

fn launch_app(path: &str) {
    librustica_gui::spawn(path);
}
```

**Tests:**
- Open app launcher from panel
- Click app, verify it launches
- Verify multiple apps can run simultaneously

**Deliverable:** Functional application launcher

---

## Success Criteria

Phase 7 is complete when:

### 7A: Input & Cursor
- [ ] PS/2 mouse driver functional
- [ ] Mouse cursor visible and tracks movement
- [ ] Event queue processes mouse + keyboard events

### 7B: Window Manager
- [ ] Windows can be created, destroyed, focused
- [ ] Windows can be dragged by titlebar
- [ ] Multiple overlapping windows render correctly
- [ ] Dracula theme applied to all windows

### 7C: Client Library
- [ ] Syscalls for window operations work
- [ ] librustica_gui compiles and links
- [ ] GUI demo application runs and responds to clicks

### 7D: Desktop Shell
- [ ] Desktop background renders
- [ ] Panel with app launcher works
- [ ] Applications can be launched from menu
- [ ] Multiple GUI apps run concurrently

---

## Testing & Validation

### Hardware Testing

- Primary target: Live USB boot on UEFI hardware
- Secondary: QEMU/VNC for rapid iteration
- Test on minimum 3 different machines

### Visual Testing

- All UI elements use Dracula colors
- Windows overlap correctly (Z-order)
- No screen tearing during window dragging
- Cursor doesn't leave artifacts

### Interaction Testing

- Mouse clicks register correctly
- Keyboard input works in focused window
- Window focus switches on click
- Dragging is smooth and responsive

---

## Development Workflow

### Incremental Development

1. Implement feature in kernel
2. Add corresponding syscall
3. Update librustica_gui
4. Test with simple demo app
5. Integrate into desktop shell

### Debug Strategy

- Framebuffer debug messages (colored rectangles)
- LED patterns for critical errors
- Persist logs to ramdisk file
- VNC display for QEMU testing

### Build Script Integration

```bash
# build-gui.sh
cargo build --release --target x86_64-unknown-uefi --features gui_support
cd userspace/librustica_gui && cargo build --release
cd userspace/gui-demo && make
cd userspace/desktop && make
./build-live-image.sh
```

---

## Technical Debt & Future Work

Intentionally deferred to Phase 8+:

- Double buffering (avoid flicker)
- Window resize/maximize
- Keyboard shortcuts (Alt+Tab, etc.)
- Window minimize to panel
- Right-click context menus
- Copy/paste clipboard
- USB HID mouse support

**Why defer:**
- Phase 7 establishes GUI foundation
- These are polish features, not blockers
- Focus on getting GUI working first

---

## Final Notes

This phase transforms Rustux from a text-based OS into a graphical OS, but maintains the CLI as first-class:

- **Ctrl+Alt+F1** → CLI
- **Ctrl+Alt+F2** → GUI

The design is intentionally simple:

- No IPC complexity
- No multi-process compositor
- No hardware acceleration (yet)
- Just framebuffer + mouse + windows

This gets you a working GUI in 8-10 weeks that can run real applications, with room to add sophistication in Phase 8+.

---

**Dracula Theme Colors (Reference):**

```rust
pub const DRACULA_BG: Color = Color::from_rgb(40, 42, 54);
pub const DRACULA_FG: Color = Color::from_rgb(248, 248, 242);
pub const DRACULA_PURPLE: Color = Color::from_rgb(189, 147, 249);
pub const DRACULA_CYAN: Color = Color::from_rgb(139, 233, 253);
pub const DRACULA_GREEN: Color = Color::from_rgb(80, 250, 123);
pub const DRACULA_ORANGE: Color = Color::from_rgb(255, 184, 108);
pub const DRACULA_RED: Color = Color::from_rgb(248, 40, 62);
pub const DRACULA_YELLOW: Color = Color::from_rgb(235, 219, 178);
pub const DRACULA_SELECTION: Color = Color::from_rgb(68, 71, 90);
```

---

**Repository:** https://github.com/gitrustux/rustux
