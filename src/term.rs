use std::io::{self, Write};

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

    pub fn check_term_resize(&mut self) -> bool {
        let checker = term_size();

        if checker.width != self.width || checker.height != self.height {
            self.width = checker.width;
            self.height = checker.height;

            return true;
        }

        false
    }
}

#[repr(C)]
pub struct WidthHeight {
    width: c_uint,
    height: c_uint,
}

extern "C" {
    fn set_raw_term();
    fn set_cooked_term();
    fn c_kbhit() -> c_uint;
    fn get_term_size() -> WidthHeight;
    fn get_ch() -> c_char;
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

pub fn term_size() -> Wh {
    let from_c = unsafe { get_term_size() };

    Wh::from_c(from_c)
}

pub fn move_cursor_home() {
    print!("\u{001b}[H");
    io::stdout().flush().unwrap();
}

pub fn switch_to_alt_buf() {
    print!("\u{001b}[?1049h");
    io::stdout().flush().unwrap();
}

pub fn return_to_normal_buf() {
    print!("\u{001b}[?1049l");
    io::stdout().flush().unwrap();
}

pub fn clear_screen() {
    print!("\u{001b}[2J");
    io::stdout().flush().unwrap();
}

pub fn move_cursor_to(row: usize, column: usize) {
    print!("\u{001b}[{};{}H", row, column);
    io::stdout().flush().unwrap();
}

pub fn print_flush(message: &str) {
    print!("{message}");
    io::stdout().flush().unwrap();
}
