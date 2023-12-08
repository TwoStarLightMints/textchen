use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use textchen::cursor::*;
use textchen::document::*;
use textchen::term::*;

// Every line is a String
// A file is a collection of lines with some whitespace
// Vec<&str>

#[derive(PartialEq, Eq)]
enum Modes {
    Normal,
    Insert,
    Command,
}

fn change_mode(
    curr: &mut Modes,
    new_mode: Modes,
    mode_row: u32,
    mode_column: u32,
    cursor: &Cursor,
) {
    *curr = new_mode;

    move_cursor_to(mode_column, mode_row);

    match curr {
        Modes::Normal => print!("NOR"),
        Modes::Insert => print!("INS"),
        Modes::Command => print!("COM"),
    };

    io::stdout().flush().unwrap();

    move_cursor_to(cursor.column, cursor.row);
}

const J_LOWER: u8 = 106;
const K_LOWER: u8 = 107;
const L_LOWER: u8 = 108;
const H_LOWER: u8 = 104;
const O_LOWER: u8 = 111;
const I_LOWER: u8 = 105;
const Q_LOWER: u8 = 113;
const COLON: u8 = 58;
const ESC: u8 = 27;
const BCKSP: u8 = 127;

fn main() {
    let test = term_size();
    let mut mode = Modes::Normal;

    // Title row is the home row, so no variable is used for this value
    let mode_row = test.get_height() - 1; // Second to last line
    let command_row = test.get_height(); // Last line
    let editor_top = 2;
    let editor_bottom = test.get_height() - 1;
    let editor_right = test.get_width();

    let mut args = env::args();
    args.next();

    let mut buf = String::new();
    let mut document: Document;

    clear_screen();

    if let Some(file_name) = args.next() {
        // If a file has been provided through command line
        let mut in_file = File::open(&file_name).unwrap();

        in_file.read_to_string(&mut buf).unwrap();

        move_cursor_to(0, editor_top);

        document = Document::new(buf.clone());

        println!("{document}");

        move_cursor_home();
        print!("{file_name}");
    } else {
        document = Document::new("".to_string());
        move_cursor_home();
        print!("[ scratch ]");
    }

    move_cursor_to(0, mode_row);
    print!("NOR");
    move_cursor_to(0, command_row);
    print!("Command area");

    set_raw();

    // Here, cursor_x is initially set to 1 as setting it to 0 would require the user to press l multiple times to move away from the left barrier
    let mut cursor = Cursor::new(2, 1);
    move_cursor_to(cursor.column, cursor.row);

    let mut insert_point: usize = 0;
    buf.clear();

    loop {
        match get_char() as u8 {
            J_LOWER if mode == Modes::Normal => {
                // Move down
                if cursor.row <= document.lines.len() as u32 {
                    cursor.move_down();

                    if cursor.column > document.lines[(cursor.row - 2) as usize].len() as u32 {
                        // If moving the cursor down moves the cursor out of bounds of the next line
                        cursor.column =
                            (document.lines[(cursor.row - 2) as usize].len() + 1) as u32;
                        cursor.update_pos();
                    }
                }
            }
            L_LOWER if mode == Modes::Normal => {
                // Move right
                if cursor.column <= document.lines[(cursor.row - 2) as usize].len() as u32 {
                    cursor.move_right()
                }
            }
            K_LOWER if mode == Modes::Normal => {
                // Move up
                cursor.move_up();

                if cursor.row - 1 >= editor_top {
                    if cursor.column > document.lines[(cursor.row - 2) as usize].len() as u32 {
                        // If moving the cursor down moves the cursor out of bounds of the next line
                        cursor.column =
                            (document.lines[(cursor.row - 2) as usize].len() + 1) as u32;
                        cursor.update_pos();
                    }
                }
            }
            H_LOWER if mode == Modes::Normal => {
                // Move left
                if cursor.column - 1 > 0 {
                    cursor.move_left()
                }
            }
            I_LOWER if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);
                insert_point = cursor.column as usize;
            }
            O_LOWER if mode == Modes::Normal => {
                document.lines.push(String::new());
                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);
            }
            ESC if mode == Modes::Insert => {
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                let original_line = document.lines[cursor.row as usize].clone();
                let mut new_line = String::with_capacity(original_line.len() + buf.len() + 1);

                new_line += &original_line;
                new_line.insert_str(insert_point, &buf);

                document.lines[cursor.row as usize] = new_line;
                buf.clear();
            }
            Q_LOWER if mode == Modes::Normal => {
                clear_screen();
                move_cursor_home();
                break;
            }
            BCKSP if mode == Modes::Insert => {
                if cursor.column - 1 > 0 {
                    cursor.move_left(); // Move the cursor on top of the character to be deleted
                    print!(" "); // Print a space on top of whatever was there, effectively "deleting" it
                    move_cursor_to(cursor.column, cursor.row); // The cursor was moved from the inteded position, move it back
                    buf.pop();
                }
            }
            c if mode == Modes::Insert => {
                if cursor.column + 1 < editor_right {
                    print!("{}", c as char);
                    cursor.move_right();
                    buf.push(c as char);
                }
            }
            COLON if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::Command, test.get_height() - 1, 0, &cursor);
            }
            c if mode == Modes::Command => {
                todo!("Implement commands");
            }
            _ => (),
        }
    }

    set_cooked();
}
