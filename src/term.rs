use std::io::{self, Write};

// Examples of ANSI escape codes from: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797

#[allow(dead_code)]
use std::ffi::{c_char, c_uint};

#[repr(C)]
pub struct Wh {
    width: c_uint,
    height: c_uint,
}

impl Wh {
    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_height(&self) -> u32 {
        self.height as u32
    }
}

extern "C" {
    fn set_raw_term();
    fn set_cooked_term();
    fn get_ch() -> c_char;
    fn get_term_size() -> Wh;
}

pub fn set_raw() {
    unsafe {
        set_raw_term();
    }
}

pub fn set_cooked() {
    unsafe {
        set_cooked_term();
    }
}

pub fn get_char() -> char {
    unsafe { get_ch() as u8 as char }
}

pub fn term_size() -> Wh {
    unsafe { get_term_size() }
}

pub fn move_cursor_home() {
    // print!("\u{001b}[H");
    print!("\u{001b}[H");
    io::stdout().flush().unwrap();
}

pub fn clear_screen() {
    print!("\u{001b}[2J");
    io::stdout().flush().unwrap();
}

pub fn move_cursor_to(column: u32, row: u32) {
    print!("\u{001b}[{};{}H", row, column);
    io::stdout().flush().unwrap();
}
