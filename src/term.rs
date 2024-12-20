#[allow(dead_code)]
use std::ffi::{c_char, c_uint};

pub struct Wh {
    pub width: usize,
    pub height: usize,
}

impl Wh {
    pub fn new() -> Self {
        unsafe {
            Self {
                width: get_terminal_width() as usize,
                height: get_terminal_height() as usize,
            }
        }
    }
}

extern "C" {
    fn c_kbhit() -> c_uint;
    fn get_terminal_width() -> c_uint;
    fn get_terminal_height() -> c_uint;
    fn get_ch() -> c_char;
}

#[cfg(target_os = "linux")]
extern "C" {
    fn set_raw_term();
    fn set_cooked_term();
}

#[cfg(target_os = "linux")]
pub fn set_raw() {
    unsafe {
        set_raw_term();
    }
}

#[cfg(target_os = "linux")]
pub fn set_cooked() {
    unsafe {
        set_cooked_term();
    }
}

pub fn get_char() -> char {
    unsafe { get_ch() as u8 as char }
}

pub fn kbhit() -> bool {
    // If the result is 0, it should be false, so return the inverse
    unsafe { c_kbhit() != 0 }
}

pub fn move_cursor_home() -> &'static str {
    "\u{001b}[H"
}

pub fn switch_to_alt_buf() -> &'static str {
    "\u{001b}[?1049h"
}

pub fn return_to_normal_buf() -> &'static str {
    "\u{001b}[?1049l"
}

pub fn clear_line() -> &'static str {
    "\u{001b}[2J"
}

pub fn move_cursor_to(row: usize, column: usize) -> String {
    format!("\u{001b}[{};{}H", row, column)
}
