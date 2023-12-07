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

const J_LOWER: u8 = 106;
const K_LOWER: u8 = 107;
const L_LOWER: u8 = 108;
const H_LOWER: u8 = 107;
const I_LOWER: u8 = 105;
const Q_LOWER: u8 = 113;
const ESC: u8 = 27;

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
            J_LOWER if mode == Modes::Normal => {
                // Move up
                cursor_y += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            L_LOWER if mode == Modes::Normal => {
                // Move right
                cursor_x += 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            K_LOWER if mode == Modes::Normal => {
                // Move down
                cursor_y -= 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            H_LOWER if mode == Modes::Normal => {
                // Move left
                cursor_x -= 1;
                move_cursor_to(cursor_x, cursor_y);
            }
            I_LOWER if mode == Modes::Normal => {
                change_mode(&mut mode, test.get_height() - 1, 0, cursor_x, cursor_y);
            }
            ESC if mode == Modes::Insert => {
                change_mode(&mut mode, test.get_height() - 1, 0, cursor_x, cursor_y)
            }
            Q_LOWER if mode == Modes::Normal => break,
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
