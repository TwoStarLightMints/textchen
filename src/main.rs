use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use textchen::{cursor::*, document::*, gapbuf::*, term::*};

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

fn clear_editor_window(editor_win_height: u32, cursor: &mut Cursor) {
    cursor.move_to(2, 1);

    for _ in 2..(editor_win_height - 1) {
        print!("\u{001b}[2K");
        cursor.move_down();
    }
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
    let mut log_file = File::create("log.txt").unwrap();
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

        document = Document::new(file_name, buf.clone(), test.get_width());

        println!("{document}");

        move_cursor_home();
        print!("{}", &document.file_name);
    } else {
        document = Document::new("scratch".to_string(), "".to_string(), test.get_width());
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
                if cursor.row <= document.get_number_lines() as u32 {
                    let original = document.get_str_at_cursor(cursor.row);

                    if cursor.column > document.get_str_at_cursor(cursor.row).len() as u32 {
                        cursor.move_down();
                        // If moving the cursor down moves the cursor out of bounds of the next line
                        cursor.move_to(
                            cursor.row,
                            (document.get_str_at_cursor(cursor.row).len() + 1) as u32,
                        );
                    } else {
                        cursor.move_down();
                    }

                    if original == document.get_str_at_cursor(cursor.row)
                        && (cursor.row - 2) + 1 <= document.num_rows() as u32
                    {
                        let num_moves_to_go = document.get_line_at_cursor(cursor.row).0.len();
                        for _ in 1..num_moves_to_go {
                            cursor.move_down();
                        }
                    }
                }
            }
            L_LOWER if mode == Modes::Normal => {
                // Move right
                let mut curr_line = document.get_line_at_cursor(cursor.row);

                if curr_line.0.len() == 1 {
                    if cursor.column <= curr_line.1.len() as u32 {
                        cursor.move_right()
                    }
                } else {
                    // (((document.get_str_at_cursor(cursor.row).len() as u32 / editor_right) - (cursor.row - 2)) * cursor.column) + cursor.column

                    // document.get_str_at_cursor(cursor.row).len() as u32 / editor_right : takes into account whole string
                    // cursor.row - 2 : doesn't take actual cursor position into full account
                    // cursor.column : only gives where the cursor is inside of the line

                    // document.get_line_at_cursor(cursor.row).0.iter().find(|i| *i == cursor.row - 2) * editor_right : skip x amount of lines, refer to this line as skip_amount
                    // skip_amount + cursor.column

                    if cursor.column < editor_right
                        && (document
                            .get_line_at_cursor(cursor.row)
                            .0
                            .iter()
                            .position(|i| *i == (cursor.get_row_usize() - 2))
                            .unwrap()
                            * editor_right as usize)
                            + cursor.get_column_usize()
                            <= document.get_str_at_cursor(cursor.row).len()
                    {
                        cursor.move_right()
                    } else if curr_line.1 == document.get_str_at_cursor(cursor.row + 1)
                        && (document
                            .get_line_at_cursor(cursor.row)
                            .0
                            .iter()
                            .position(|i| *i == (cursor.get_row_usize() - 2))
                            .unwrap()
                            * editor_right as usize)
                            + cursor.get_column_usize()
                            <= document.get_str_at_cursor(cursor.row).len()
                    {
                        cursor.move_down();

                        log_file.write("HERE".as_bytes()).unwrap();

                        cursor.move_to_left_border();
                    }
                }
            }
            K_LOWER if mode == Modes::Normal => {
                if cursor.row - 1 >= editor_top {
                    // Move up
                    cursor.move_up();

                    if document.get_line_at_cursor(cursor.row).0.len() > 1 {
                        let num_indices = document.get_line_at_cursor(cursor.row).0.len();

                        for _ in 1..num_indices {
                            cursor.move_up();
                        }
                    }

                    if cursor.column > document.get_str_at_cursor(cursor.row).len() as u32 {
                        // If moving the cursor down moves the cursor out of bounds of the next line
                        cursor.column = (document.get_str_at_cursor(cursor.row).len() + 1) as u32;
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

                    cursor.save_current_pos();

                    clear_editor_window(test.get_height(), &mut cursor);

                    display_document(&document, editor_top, &cursor);

                    cursor.revert_pos();
                    cursor.move_to_left_border();
                }
            }
            I_LOWER if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);

                gap_buf = GapBuf::from_str(
                    document.get_str_at_cursor(cursor.row),
                    (cursor.column - 1) as usize, // Needs to be decremented to make the space directly before the white block cursor the "target"
                );
            }
            O_LOWER if mode == Modes::Normal => {
                document.lines.push(Line::new());

                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);

                cursor.move_down();
                cursor.move_to_left_border();

                gap_buf = GapBuf::new();
            }
            ESC if mode == Modes::Insert => {
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                document.set_line_at_cursor(cursor.row, gap_buf.to_string());
            }
            ESC if mode == Modes::Command => {
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                cursor.move_to(test.get_height(), 0);
                print!("{: >1$}", "", test.get_width() as usize);

                cursor.revert_pos();
            }
            BCKSP if mode == Modes::Insert => {
                if cursor.column - 1 > 0 {
                    gap_buf.pop(); // Remove character from gap buffer
                    cursor.move_left();

                    cursor.save_current_pos();

                    cursor.move_to(cursor.row, 0);

                    // Solution found from: https://stackoverflow.com/questions/35280798/printing-a-character-a-variable-number-of-times-with-println
                    // Check if the original string or the gap buffer are longer, whichever is, use that size to print an appropriate amount of spaces to
                    // "clear" the line and make it suitable to redraw
                    if gap_buf.len() > document.get_str_at_cursor(cursor.row).len() {
                        print!("{: >1$}", "", gap_buf.len());
                    } else {
                        print!("{: >1$}", "", document.get_str_at_cursor(cursor.row).len());
                    }

                    cursor.move_to(cursor.row, 0);

                    // Draw the updated string to the screen
                    print!("{gap_buf}");

                    cursor.revert_pos();
                } else {
                    // cursor.column >= 0
                    if document.lines.len() > 1 {
                        if document.get_str_at_cursor(cursor.row).len() > 0 {
                            let curr_line = document.get_str_at_cursor(cursor.row);

                            document.lines.remove((cursor.row - 1) as usize);

                            cursor.move_up();
                            cursor.move_to(
                                cursor.row,
                                (document.get_str_at_cursor(cursor.row).len() + 1) as u32,
                            );

                            let new_line = document.get_str_at_cursor(cursor.row);

                            document.set_line_at_cursor(
                                cursor.row,
                                String::with_capacity(
                                    document.get_str_at_cursor(cursor.row).len() + curr_line.len(),
                                ),
                            );

                            document.lines[(cursor.row - 2) as usize].1 += &new_line;
                            document.lines[(cursor.row - 2) as usize].1 += &curr_line;

                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.row),
                                (cursor.column - 1) as usize,
                            );

                            cursor.save_current_pos();

                            cursor.move_to_left_border();

                            print!("{gap_buf}");

                            cursor.revert_pos();
                        } else {
                            document.lines.remove((cursor.row - 2) as usize);
                            cursor.move_up();

                            cursor.column =
                                (document.get_str_at_cursor(cursor.row).len() + 1) as u32;

                            cursor.update_pos();
                        }
                    }
                }
            }
            c if mode == Modes::Insert => {
                if cursor.column + 1 <= editor_right {
                    gap_buf.insert(c as char);
                    cursor.move_right();

                    let original_cursor_column = cursor.column;

                    move_cursor_to(0, cursor.row);

                    if gap_buf.len() > document.get_str_at_cursor(cursor.row).len() {
                        print!("{: >1$}", "", gap_buf.len());
                    } else {
                        print!("{: >1$}", "", document.get_str_at_cursor(cursor.row).len());
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
                cursor.save_current_pos();

                cursor.move_to(command_row, 1);

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
