use std::io::{self, Write};

// Examples of ANSI escape codes from: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797

#[allow(dead_code)]
use std::ffi::{c_char, c_uint};

pub struct Wh {
    pub width: usize,
    pub height: usize,
}

impl Wh {
    pub fn from_c(c_vals: WidthHeight) -> Self {
        let width = c_vals.width as usize;
        let height = c_vals.height as usize;
        Self { width, height }
    }
}

#[repr(C)]
pub struct WidthHeight {
    width: c_uint,
    height: c_uint,
}

impl WidthHeight {
    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_height(&self) -> u32 {
        self.height as u32
    }
}

extern "C" {
    #[cfg(target_os = "linux")]
    fn set_raw_term();
    #[cfg(target_os = "linux")]
    fn set_cooked_term();
    fn get_ch() -> c_char;
    fn get_term_size() -> WidthHeight;
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

pub fn term_size() -> Wh {
    let from_c = unsafe { get_term_size() };

    Wh::from_c(from_c)
}

pub fn move_cursor_home() {
    print!("\u{001b}[H");
    io::stdout().flush().unwrap();
}

pub fn clear_screen() {
    print!("\u{001b}[2J");
    io::stdout().flush().unwrap();
}

pub fn move_cursor_to(column: usize, row: usize) {
    print!("\u{001b}[{};{}H", row, column);
    io::stdout().flush().unwrap();
}

pub fn print_flush(message: &str) {
    print!("{message}");
    io::stdout().flush().unwrap();
}
