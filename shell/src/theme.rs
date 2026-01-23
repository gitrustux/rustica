// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Dracula Theme for Rustica Shell
//!
//! This module provides the Dracula color theme for the shell.
//! The Dracula theme is the non-negotiable default theme for Rustica.

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// =============================================================
// DRACULA THEME COLORS
// =============================================================

/// Background color
pub const DRACULA_BG: Color = Color { r: 40, g: 42, b: 54 };

/// Foreground (text) color
pub const DRACULA_FG: Color = Color { r: 248, g: 248, b: 242 };

/// Purple - used for prompts
pub const DRACULA_PURPLE: Color = Color { r: 189, g: 147, b: 249 };

/// Cyan - used for info messages
pub const DRACULA_CYAN: Color = Color { r: 139, g: 233, b: 253 };

/// Green - used for success messages
pub const DRACULA_GREEN: Color = Color { r: 80, g: 250, b: 123 };

/// Orange - used for warnings
pub const DRACULA_ORANGE: Color = Color { r: 255, g: 184, b: 108 };

/// Red - used for errors
pub const DRACULA_RED: Color = Color { r: 248, g: 40, b: 62 };

/// Yellow - used for highlights
pub const DRACULA_YELLOW: Color = Color { r: 235, g: 219, b: 178 };

/// Pink - used for special highlights
pub const DRACULA_PINK: Color = Color { r: 255, g: 121, b: 198 };

/// Comment color (gray)
pub const DRACULA_COMMENT: Color = Color { r: 98, g: 114, b: 164 };

// =============================================================
// ANSI COLOR CODES
// =============================================================

/// ANSI escape code prefix
const ANSI_ESCAPE: &[u8] = b"\x1b[";

/// ANSI color codes for terminal output
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl AnsiColor {
    /// Get the ANSI code sequence for this color
    pub fn code(&self) -> &[u8] {
        match self {
            AnsiColor::Black => b"30m",
            AnsiColor::Red => b"31m",
            AnsiColor::Green => b"32m",
            AnsiColor::Yellow => b"33m",
            AnsiColor::Blue => b"34m",
            AnsiColor::Magenta => b"35m",
            AnsiColor::Cyan => b"36m",
            AnsiColor::White => b"37m",
            AnsiColor::Reset => b"0m",
            AnsiColor::BrightBlack => b"90m",
            AnsiColor::BrightRed => b"91m",
            AnsiColor::BrightGreen => b"92m",
            AnsiColor::BrightYellow => b"93m",
            AnsiColor::BrightBlue => b"94m",
            AnsiColor::BrightMagenta => b"95m",
            AnsiColor::BrightCyan => b"96m",
            AnsiColor::BrightWhite => b"97m",
        }
    }

    /// Write this color code to stdout
    pub fn write(&self) {
        // Write ANSI escape sequence
        for &b in ANSI_ESCAPE {
            sys_write(1, b as *const u8, 1);
        }
        for &b in self.code() {
            sys_write(1, b as *const u8, 1);
        }
    }
}

// =============================================================
// THEME CONFIGURATION
// =============================================================

/// Shell theme configuration
pub struct Theme {
    /// Prompt color (typically purple)
    pub prompt: Color,

    /// Command color (typically foreground)
    pub command: Color,

    /// Error message color (typically red)
    pub error: Color,

    /// Success message color (typically green)
    pub success: Color,

    /// Info message color (typically cyan)
    pub info: Color,

    /// Warning message color (typically orange)
    pub warning: Color,
}

/// Default Dracula theme - non-negotiable
pub const DRACULA_THEME: Theme = Theme {
    prompt: DRACULA_PURPLE,
    command: DRACULA_FG,
    error: DRACULA_RED,
    success: DRACULA_GREEN,
    info: DRACULA_CYAN,
    warning: DRACULA_ORANGE,
};

// =============================================================
// SYSCALL DECLARATIONS
// =============================================================

/// Syscall: write to file descriptor
extern "C" {
    fn sys_write(fd: u32, buf: *const u8, len: usize) -> isize;
}

/// Helper function to write a string to stdout
fn print_str(s: &str) {
    unsafe {
        for &b in s.as_bytes() {
            sys_write(1, &b as *const u8, 1);
        }
    }
}

// =============================================================
// THEME FUNCTIONS
// =============================================================

/// Set the terminal color for output
pub fn set_color(color: Color) {
    // Map our RGB colors to closest ANSI colors
    // This is a simplified mapping - in a real system we'd use RGB escape codes
    let ansi = if color == DRACULA_PURPLE || color == DRACULA_PINK {
        AnsiColor::Magenta
    } else if color == DRACULA_CYAN {
        AnsiColor::Cyan
    } else if color == DRACULA_GREEN {
        AnsiColor::Green
    } else if color == DRACULA_ORANGE || color == DRACULA_YELLOW {
        AnsiColor::Yellow
    } else if color == DRACULA_RED {
        AnsiColor::Red
    } else if color == DRACULA_BG || color == DRACULA_COMMENT {
        AnsiColor::Black
    } else {
        AnsiColor::White
    };

    ansi.write();
}

/// Reset terminal color to default
pub fn reset_color() {
    AnsiColor::Reset.write();
}

/// Print a message with the specified color
pub fn print_color(color: Color, message: &str) {
    set_color(color);
    print_str(message);
    reset_color();
}

/// Print the shell prompt with Dracula theme
pub fn print_prompt() {
    // Print prompt in Dracula purple
    set_color(DRACULA_PURPLE);
    print_str("rustux");
    reset_color();

    print_str(" ");

    set_color(DRACULA_CYAN);
    print_str(">");
    reset_color();

    print_str(" ");
}

/// Print an error message
pub fn print_error(message: &str) {
    set_color(DRACULA_RED);
    print_str("error: ");
    reset_color();
    print_str(message);
    print_str("\n");
}

/// Print a success message
pub fn print_success(message: &str) {
    set_color(DRACULA_GREEN);
    print_str("✓ ");
    reset_color();
    print_str(message);
    print_str("\n");
}

/// Print an info message
pub fn print_info(message: &str) {
    set_color(DRACULA_CYAN);
    print_str("ℹ ");
    reset_color();
    print_str(message);
    print_str("\n");
}

/// Print a warning message
pub fn print_warning(message: &str) {
    set_color(DRACULA_ORANGE);
    print_str("⚠ ");
    reset_color();
    print_str(message);
    print_str("\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn test_dracula_constants() {
        assert_eq!(DRACULA_BG, Color::new(40, 42, 54));
        assert_eq!(DRACULA_FG, Color::new(248, 248, 242));
        assert_eq!(DRACULA_PURPLE, Color::new(189, 147, 249));
    }

    #[test]
    fn test_theme_constants() {
        assert_eq!(DRACULA_THEME.prompt, DRACULA_PURPLE);
        assert_eq!(DRACULA_THEME.command, DRACULA_FG);
        assert_eq!(DRACULA_THEME.error, DRACULA_RED);
        assert_eq!(DRACULA_THEME.success, DRACULA_GREEN);
    }
}
