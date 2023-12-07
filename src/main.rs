use std::io::{self, Write};
use textchen::term::*;

#[derive(PartialEq, Eq)]
enum Modes {
    Normal,
    Insert,
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

fn main() {
    let test = term_size();
    let mut mode = Modes::Normal;

    clear_screen();
    move_cursor_home();
    print!("Title");
    move_cursor_to(0, test.get_height() - 1);
    print!("NOR");
    move_cursor_to(0, test.get_height());
    print!("Command area");
    move_cursor_to(0, 2);

    io::stdout().flush().unwrap();

    set_raw();

    let mut cursor_x = 1;
    let mut cursor_y = 1;

    loop {
        match get_char() as u8 {
            106 if mode == Modes::Normal => {
                // Move up
                cursor_y += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            108 if mode == Modes::Normal => {
                // Move right
                cursor_x += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            107 if mode == Modes::Normal => {
                // Move down
                cursor_y -= 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            104 if mode == Modes::Normal => {
                // Move left
                cursor_x -= 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            105 if mode == Modes::Normal => {
                change_mode(&mut mode, test.get_height() - 1, 0, cursor_x, cursor_y);
            }
            27 if mode == Modes::Insert => {
                change_mode(&mut mode, test.get_height() - 1, 0, cursor_x, cursor_y)
            }
            113 if mode == Modes::Normal => break,
            c if mode == Modes::Insert => {
                print!("{}", c as char);
                cursor_x += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            _ => (),
        }
    }

    set_cooked();
}
