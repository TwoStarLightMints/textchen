use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use textchen::cursor::*;
use textchen::document::*;
use textchen::gapbuf::*; // Used when editing text
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

fn display_document(doc: &Document, editor_top: u32, cursor: &Cursor) {
    move_cursor_to(0, editor_top);

    println!("{doc}");

    cursor.update_pos();
}

const J_LOWER: u8 = 106;
const K_LOWER: u8 = 107;
const L_LOWER: u8 = 108;
const H_LOWER: u8 = 104;
const X_LOWER: u8 = 120;
const O_LOWER: u8 = 111;
const I_LOWER: u8 = 105;
const COLON: u8 = 58;
const ESC: u8 = 27;
const BCKSP: u8 = 127;
const RETURN: u8 = 10;

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

        document = Document::new(file_name, buf.clone());

        println!("{document}");

        move_cursor_home();
        print!("{}", &document.file_name);
    } else {
        document = Document::new("scratch".to_string(), "".to_string());
        move_cursor_home();
        print!("[ scratch ]");
    }

    move_cursor_to(0, mode_row);
    print!("NOR");

    set_raw();

    // Here, cursor_x is initially set to 1 as setting it to 0 would require the user to press l multiple times to move away from the left barrier
    let mut cursor = Cursor::new(2, 1);
    move_cursor_to(cursor.column, cursor.row);

    let mut gap_buf = GapBuf::new();
    buf.clear();

    let mut cur_row_store: u32 = 0;
    let mut cur_column_store: u32 = 0;

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
            X_LOWER if mode == Modes::Normal => {
                if get_char() == 'd' {
                    document.lines.remove((cursor.row - 2) as usize);
                    display_document(&document, editor_top, &cursor);

                    move_cursor_to(0, (document.lines.len() + 2) as u32);
                    print!(
                        "{: <1$}",
                        "",
                        document.lines[document.lines.len() - 1].len()
                    );

                    cursor.update_pos();
                }
            }
            I_LOWER if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);
                gap_buf = GapBuf::from_str(
                    document.lines[(cursor.row - 2) as usize].clone(),
                    (cursor.column - 1) as usize, // Needs to be decremented to make the space directly before the white block cursor the "target"
                );
            }
            O_LOWER if mode == Modes::Normal => {
                document.lines.push(String::new());

                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);

                cursor.move_down();
                cursor.move_to_left_border();

                gap_buf = GapBuf::new();
            }
            ESC if mode == Modes::Insert => {
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                document.lines[(cursor.row - 2) as usize] = gap_buf.to_string();
            }
            ESC if mode == Modes::Command => {
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                move_cursor_to(0, test.get_height());
                print!("{: >1$}", "", test.get_width() as usize);

                cursor.row = cur_row_store;
                cursor.column = cur_column_store;
                cursor.update_pos();
            }
            BCKSP if mode == Modes::Insert => {
                if cursor.column - 1 > 0 {
                    gap_buf.pop(); // Remove character from gap buffer
                    cursor.move_left();

                    let cursor_original_column = cursor.column; // Store the current column of the cursor to be able to move back to it after clean up

                    move_cursor_to(0, cursor.row); // Move the cursor to the beginning of the row

                    // Solution found from: https://stackoverflow.com/questions/35280798/printing-a-character-a-variable-number-of-times-with-println
                    // Check if the original string or the gap buffer are longer, whichever is, use that size to print an appropriate amount of spaces to
                    // "clear" the line and make it suitable to redraw
                    if gap_buf.len() > document.lines[(cursor.row - 2) as usize].len() {
                        print!("{: >1$}", "", gap_buf.len());
                    } else {
                        print!(
                            "{: >1$}",
                            "",
                            document.lines[(cursor.row - 2) as usize].len()
                        );
                    }

                    move_cursor_to(0, cursor.row);

                    // Draw the updated string to the screen
                    print!("{gap_buf}");

                    move_cursor_to(cursor_original_column, cursor.row); // The cursor was moved from the inteded position, move it back
                }
            }
            c if mode == Modes::Insert => {
                if cursor.column + 1 < editor_right {
                    gap_buf.insert(c as char);
                    cursor.move_right();

                    let original_cursor_column = cursor.column;

                    move_cursor_to(0, cursor.row);

                    if gap_buf.len() > document.lines[(cursor.row - 2) as usize].len() {
                        print!("{: >1$}", "", gap_buf.len());
                    } else {
                        print!(
                            "{: >1$}",
                            "",
                            document.lines[(cursor.row - 2) as usize].len()
                        );
                    }

                    move_cursor_to(0, cursor.row);
                    print!("{gap_buf}");

                    move_cursor_to(original_cursor_column, cursor.row);
                }
            }
            COLON if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::Command, test.get_height() - 1, 0, &cursor);
                buf.clear();

                cur_column_store = cursor.column;
                cur_row_store = cursor.row;

                cursor.row = command_row;
                cursor.column = 1;

                cursor.update_pos();

                print_flush(":");

                cursor.move_right();
            }
            RETURN if mode == Modes::Command => match buf.as_str() {
                "w" => {
                    let mut out_file = File::create(&document.file_name).unwrap();

                    out_file.write(document.to_string().as_bytes()).unwrap();

                    move_cursor_to(0, command_row);
                    print!("{: >1$}", "", test.get_width() as usize);

                    change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                    cursor.row = cur_row_store;
                    cursor.column = cur_column_store;
                    cursor.update_pos();

                    buf.clear();
                }
                "q" => {
                    clear_screen();
                    move_cursor_home();
                    break;
                }
                _ => (),
            },
            c if mode == Modes::Command => {
                buf.push(c as char);
                print_flush(&format!("{}", c as char));
                cursor.move_right();
            }

            _ => (),
        }
    }

    set_cooked();
}
