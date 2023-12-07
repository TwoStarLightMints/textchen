use std::io::{self, Write};

#[allow(dead_code)]
use std::ffi::{c_char, c_uint};

#[repr(C)]
struct Wh {
    width: c_uint,
    height: c_uint,
}

extern "C" {
    fn set_raw_term();
    fn set_cooked_term();
    fn get_ch() -> c_char;
    fn get_term_size() -> Wh;
}

fn set_raw() {
    unsafe {
        set_raw_term();
    }
}

fn set_cooked() {
    unsafe {
        set_cooked_term();
    }
}

fn get_char() -> char {
    unsafe { get_ch() as u8 as char }
}

fn move_cursor_home() {
    // print!("\u{001b}[H");
    print!("\u{001b}[H");
    io::stdout().flush().unwrap();
}

fn clear_screen() {
    print!("\u{001b}[2J");
    io::stdout().flush().unwrap();
}

fn move_cursor_to(x: u32, y: u32) {
    print!("\u{001b}[{};{}H", y, x);
    io::stdout().flush().unwrap();
}

fn change_mode(
    curr: &mut Modes,
    mode_row: u32,
    mode_column: u32,
    curr_cursor_row: u32,
    curr_cursor_column: u32,
) {
    *curr = match curr {
        Modes::Normal => Modes::Insert,
        Modes::Insert => Modes::Normal,
    };

    move_cursor_to(mode_column, mode_row);

    match curr {
        Modes::Normal => print!("NOR"),
        Modes::Insert => print!("INS"),
    };

    io::stdout().flush().unwrap();

    move_cursor_to(curr_cursor_row, curr_cursor_column);
}

// Examples of ANSI escape codes from: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
// println!("\u{001b}[{}BBrunt Mann", test.height as u32);
// println!("\u{001b}[HBrunt Mann");

enum Modes {
    Normal,
    Insert,
}

fn main() {
    let test = unsafe { get_term_size() };
    let mut mode = Modes::Normal;

    clear_screen();
    move_cursor_home();
    print!("Title");
    move_cursor_to(0, test.height as u32 - 1);
    print!("NOR");
    move_cursor_to(0, test.height as u32);
    print!("Command area");
    move_cursor_to(0, 2);

    io::stdout().flush().unwrap();

    set_raw();

    let mut cursor_x = 1;
    let mut cursor_y = 1;

    loop {
        match get_char() as u8 {
            106 => {
                // Move up
                cursor_y += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            108 => {
                // Move right
                cursor_x += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            107 => {
                // Move down
                cursor_y -= 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            104 => {
                // Move left
                cursor_x -= 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            105 | 27 => {
                change_mode(&mut mode, test.height as u32 - 1, 0, cursor_x, cursor_y);
            }
            113 => break,
            _ => (),
        }
    }

    set_cooked();
}
