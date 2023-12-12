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

        // Move cursor to editor home
        move_cursor_to(0, editor_top);

        // Create document struct instance from file and editor width
        document = Document::new(file_name, buf.clone(), test.get_width());

        // Display document
        println!("{document}");

        // Move cursor to home to print file name
        move_cursor_home();
        print!("{}", &document.file_name);
    } else {
        // No file name provided
        todo!("Implement scratch buffer");
        // Create new empty document with default name scratch
        document = Document::new("scratch".to_string(), "".to_string(), test.get_width());

        // Print scratch to screen instead of file name
        move_cursor_home();
        print!("[ scratch ]");
    }

    // Print the mode to the screen, in this case, the default is normal
    move_cursor_to(0, mode_row);
    print!("NOR");

    set_raw();

    // Here, cursor_x is initially set to 1 as setting it to 0 would require the user to press l multiple times to move away from the left barrier
    let mut cursor = Cursor::new(2, 1);
    move_cursor_to(cursor.column, cursor.row);

    // Initialize the gap buffer, it will be replaced later when editing actual text
    let mut gap_buf = GapBuf::new();
    // Clear the buffer
    buf.clear();

    loop {
        match get_char() as u8 {
            J_LOWER if mode == Modes::Normal => {
                // Move down
                // Check that the cursor's row field is less than or equal to the number of *Lines* not *rows* in the document
                if cursor.row <= document.get_number_lines() as u32 {
                    // Store the original string that the cursor is at now
                    let original = document.get_str_at_cursor(cursor.row);

                    if cursor.column > original.len() as u32 {
                        // If the cursor's column field is at the very end of the current line, move the cursor down and to the end of the next line
                        cursor.move_down();

                        // Move the cursor to the end of the line
                        cursor.move_to(
                            cursor.row,
                            (document.get_str_at_cursor(cursor.row).len() + 1) as u32,
                        );
                    } else if document.get_str_at_cursor(cursor.row + 1).len()
                        < cursor.get_column_usize()
                    {
                        // If the cursor is within the original line but outside of the next line
                        cursor.move_down();

                        // Move the cursor to the end of the line
                        cursor.move_to(
                            cursor.row,
                            (document.get_str_at_cursor(cursor.row).len() + 1) as u32,
                        );
                    } else {
                        // If the cursor is within the current line and the next line
                        cursor.move_down();
                    }

                    if original == document.get_str_at_cursor(cursor.row)
                        && (cursor.row - 2) + 1 <= document.num_rows() as u32
                    {
                        // If the line moved to is the line itself, and the line directly below this line is not the end of the document, skip till next full line

                        // Get the number of indices the line spans
                        let num_moves_to_go = document.get_line_at_cursor(cursor.row).0.len();

                        // The cursor was already moved down once, so skip 1 and move down the number of remaining indices
                        for _ in 1..num_moves_to_go {
                            cursor.move_down();
                        }
                    }
                }
            }
            L_LOWER if mode == Modes::Normal => {
                // Move right

                // get the current line
                let curr_line = document.get_line_at_cursor(cursor.row);

                if curr_line.0.len() == 1 {
                    // If the number of rows that the current line spans is only 1, then simply move to the right
                    if cursor.get_column_usize() <= curr_line.1.len() {
                        // Until the cursor's column field is at most the length of the text
                        cursor.move_right();
                    }
                } else {
                    // If the line spans more than one roww
                    if cursor.column < editor_right
                        && cursor.get_position_in_line(&document, editor_right as usize)
                            <= curr_line.1.len()
                    {
                        // If the cursor's column is less than the right edge of the editor, and it is still at most the length of the current line
                        cursor.move_right()
                    } else if curr_line.1 == document.get_str_at_cursor(cursor.row + 1)
                        && cursor.get_position_in_line(&document, editor_right as usize)
                            <= curr_line.1.len()
                    {
                        // Otherwise, if the current line is the same as the line in the next row, and it is still at most the length of the current line
                        // Move down
                        cursor.move_down();

                        // And move to the left edge
                        cursor.move_to_left_border();
                    }
                }
            }
            K_LOWER if mode == Modes::Normal => {
                if cursor.row - 1 >= editor_top {
                    // If moving the cursor up 1 is at most the editor top
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
                let curr_line = document.get_line_at_cursor(cursor.row);
                // Move left
                if cursor.column - 1 > 0 {
                    // If moving the cursor left does not take the cursor outside of the editor range
                    cursor.move_left()
                } else if curr_line.0.len() > 1
                    && (cursor.get_row_usize() - 2) != *curr_line.0.first().unwrap()
                {
                    // If moving the cursor left would mean wrapping to the very end of the previous row of the same line, move the cursor up and to the end of the editor screen
                    cursor.move_up();
                    cursor.move_to(cursor.row, editor_right);
                }
            }
            X_LOWER if mode == Modes::Normal => {
                todo!("Implement deleting lines for multiline Lines");
                if get_char() == 'd' {
                    // The key combination xd will delete a line
                    // Remove the line from the document
                    document.remove_index_from_line(cursor.get_row_usize());

                    // Save current position
                    cursor.save_current_pos();

                    // Clear the editor area
                    clear_editor_window(test.get_height(), &mut cursor);

                    // Display the document again
                    display_document(&document, editor_top, &cursor);

                    // Return to previous position
                    cursor.revert_pos();
                    // Move to left edge of editor
                    cursor.move_to_left_border();
                }
            }
            I_LOWER if mode == Modes::Normal => {
                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);

                // Create a new gap buffer from the string at the current cursor position
                gap_buf = GapBuf::from_str(
                    document.get_str_at_cursor(cursor.row),
                    cursor.get_position_in_line(&document, editor_right as usize) - 1, // Needs to be decremented to make the space directly before the white block cursor the "target"
                );
            }
            O_LOWER if mode == Modes::Normal => {
                // Create a new empty line
                document.lines.push(Line::new());

                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, test.get_height() - 1, 0, &cursor);

                // Move down to the new row
                cursor.move_down();
                // Move to the left edge of the editor
                cursor.move_to_left_border();

                // Crate an empty gap buffer
                gap_buf = GapBuf::new();
            }
            ESC if mode == Modes::Insert => {
                // Change mode to normal
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                // Make the edits persist in memory
                document.set_line_at_cursor(cursor.row, gap_buf.to_string());
            }
            ESC if mode == Modes::Command => {
                // Change mode to normal
                change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                // Move cursor to the command line row
                cursor.move_to(test.get_height(), 0);

                // Visually delete the contents of the row
                print!("{: >1$}", "", test.get_width() as usize);

                // The cursor position was saved when switching to command mode, so revert to that position
                cursor.revert_pos();

                // Clear the buffer
                buf.clear();
            }
            BCKSP if mode == Modes::Insert => {
                todo!("Implement recalculating the indices for multiline Lines when they turn into a lesser number of lines");
                if cursor.column - 1 > 0 {
                    // If the cursor's column after moving to the left is greater than 0
                    gap_buf.pop(); // Remove character from gap buffer
                    cursor.move_left();

                    let curr_line = document.get_line_at_cursor(cursor.row);

                    cursor.save_current_pos();

                    // Move the cursor to the beginning of the line (I use the cursor's first index here to make it independent of multiline vs single line Lines)
                    cursor.move_to(
                        // Add 2 to compensate for space from top of screen, because this returns the 0 based index of the line
                        (*curr_line.0.first().unwrap() + 2) as u32,
                        0,
                    );

                    // Solution found from: https://stackoverflow.com/questions/35280798/printing-a-character-a-variable-number-of-times-with-println
                    // Check if the original string or the gap buffer are longer, whichever is, use that size to print an appropriate amount of spaces to
                    // "clear" the line and make it suitable to redraw
                    if curr_line.0.len() == 1 {
                        print!("{: >1$}", "", editor_right as usize);
                    } else {
                        for _ in 0..curr_line.0.len() {
                            print!("{: >1$}", "", editor_right as usize);
                        }
                    }

                    cursor.move_to(cursor.row, 0);

                    // Draw the updated string to the screen
                    print!("{gap_buf}");

                    cursor.revert_pos();
                } else {
                    // cursor.column >= 0
                    let curr_line = document.get_line_at_cursor(cursor.row);

                    if document.lines.len() > 1 {
                        if curr_line.1.len() > 0 {
                            // This is the branch handling moving the contents of a string which is not fully deleted into the line above it
                            document.lines.remove((cursor.row - 2) as usize);

                            cursor.move_up();
                            cursor.move_to(
                                cursor.row,
                                (document.get_str_at_cursor(cursor.row).len() + 1) as u32,
                            );

                            let new_line = document.get_str_at_cursor(cursor.row);

                            document.set_line_at_cursor(
                                cursor.row,
                                String::with_capacity(
                                    document.get_str_at_cursor(cursor.row).len()
                                        + curr_line.1.len(),
                                ),
                            );

                            document.lines[(cursor.row - 2) as usize].1 += &new_line;
                            document.lines[(cursor.row - 2) as usize].1 += &curr_line.1;

                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.row),
                                (cursor.column - 1) as usize,
                            );

                            cursor.save_current_pos();

                            cursor.move_to_left_border();

                            print!("{gap_buf}");

                            cursor.revert_pos();
                        } else {
                            // This branch handles full deletion of a line
                            if curr_line.0.len() > 1 {
                                // If the line had spanned more than one line
                                document.remove_index_from_line(cursor.get_row_usize());
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
            }
            c if mode == Modes::Insert => {
                todo!("Implement recalculating the indices for single line Lines when they turn into a greater number of lines");
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

                // Clear the buffer to ensure the new command will be empty
                buf.clear();

                // Save cursor position to come back to
                cursor.save_current_pos();

                // Move cursor to command row
                cursor.move_to(command_row, 1);
                // Clear the line if something was already printed there
                print!("{: >1$}", "", test.get_width() as usize);
                // Move cursor to command row
                cursor.move_to(command_row, 1);
                // Print a colon
                print_flush(":");

                // Move the cursor to align with the colon
                cursor.move_right();
            }
            RETURN if mode == Modes::Command => match buf.as_str() {
                "w" => {
                    let mut out_file = File::create(&document.file_name).unwrap();

                    out_file.write(document.to_string().as_bytes()).unwrap();

                    move_cursor_to(0, command_row);
                    print!("{: >1$}", "", test.get_width() as usize);

                    change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                    cursor.revert_pos();

                    buf.clear();
                }
                "q" => {
                    clear_screen();
                    move_cursor_home();
                    break;
                }
                "wq" => {
                    let mut out_file = File::create(&document.file_name).unwrap();

                    out_file.write(document.to_string().as_bytes()).unwrap();

                    clear_screen();
                    move_cursor_home();
                    break;
                }
                _ => {
                    move_cursor_to(0, command_row);
                    print!("{: <1$}", "invalid command", test.get_width() as usize);

                    change_mode(&mut mode, Modes::Normal, test.get_height() - 1, 0, &cursor);

                    cursor.revert_pos();

                    buf.clear();
                }
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
