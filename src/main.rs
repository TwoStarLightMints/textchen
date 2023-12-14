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
    MoveTo,
}
fn change_mode(
    curr: &mut Modes,
    new_mode: Modes,
    mode_row: usize,
    mode_column: usize,
    cursor: &Cursor,
) {
    *curr = new_mode;

    move_cursor_to(mode_column, mode_row);

    match curr {
        Modes::Normal => print!("NOR"),
        Modes::Insert => print!("INS"),
        Modes::Command => print!("COM"),
        Modes::MoveTo => print!("MOV"),
    };

    io::stdout().flush().unwrap();

    move_cursor_to(cursor.column, cursor.row);
}

fn display_document(doc: &Document, editor_width: usize, cursor: &mut Cursor) {
    cursor.move_to(2, 0);

    for line in doc.lines.iter() {
        for (ind, char) in line.1.chars().enumerate() {
            print_flush(format!("{char}").as_str());

            if ind != 0 && (ind + 1) % editor_width == 0 {
                cursor.move_down();
                cursor.move_to_left_border();
            }
        }

        cursor.move_down();
        cursor.move_to_left_border();
    }
}

fn clear_editor_window(
    editor_width: usize,
    editor_win_height: usize,
    document: &Document,
    cursor: &mut Cursor,
) {
    cursor.move_to(2, 1);

    for _ in 0..document.num_rows() {
        // print!("\u{001b}[2K");
        print!("{: >1$}", "", editor_width);
        cursor.move_down();
    }
}

fn reset_editor_view(
    doc: &Document,
    editor_width: usize,
    editor_height: usize,
    cursor: &mut Cursor,
) {
    clear_editor_window(editor_width, editor_height, doc, cursor);

    display_document(doc, editor_width, cursor);
}

fn debug_log_document(doc: &Document, log_file: &mut File) {
    doc.lines.iter().for_each(|l| {
        log_file
            .write(format!("Line indices: {:?}, String content: {}\n", l.0, l.1).as_bytes())
            .unwrap();
    })
}

fn debug_log_dimensions(dimensions: &Wh, log_file: &mut File) {
    log_file
        .write(
            format!(
                "Terminal width: {}, Terminal height: {}\n",
                dimensions.width, dimensions.height
            )
            .as_bytes(),
        )
        .unwrap();
}

fn debug_log_cursor(cursor: &Cursor, log_file: &mut File) {
    log_file
        .write(
            format!(
                "Cursor row: {}, Cursor row relative to document: {}, Cursor column: {}\n",
                cursor.row,
                cursor.row - 2,
                cursor.column
            )
            .as_bytes(),
        )
        .unwrap();
}

fn debug_log_gapbuffer(gap_buf: &GapBuf, log_file: &mut File) {
    log_file
        .write(format!("Lhs: {:?}, Rhs: {:?}", gap_buf.lhs, gap_buf.rhs).as_bytes())
        .unwrap();
}

const J_LOWER: u8 = 106;
const K_LOWER: u8 = 107;
const L_LOWER: u8 = 108;
const H_LOWER: u8 = 104;
const X_LOWER: u8 = 120;
const O_LOWER: u8 = 111;
const I_LOWER: u8 = 105;
const G_LOWER: u8 = 103;
const COLON: u8 = 58;
const ESC: u8 = 27;
const BCKSP: u8 = 127;
#[cfg(target_os = "linux")]
const RETURN: u8 = 10;
#[cfg(target_os = "windows")]
const RETURN: u8 = 13;

fn main() {
    #[allow(unused_variables, unused_mut)]
    let mut log_file = File::create("log.txt").unwrap();

    let dimensions = term_size();

    // Title row is the home row, so no variable is used for this value
    let mode_row = dimensions.height - 1; // Second to last line, where mode is shown
    let command_row = dimensions.height; // Last line, where commands will be written to
    let editor_top = 2 as usize; // The second from first line, where the editor screen starts
    let editor_bottom = dimensions.height;
    let editor_width = dimensions.width - 2; // The width of the editor (from the left side of the terminal to at most this value), minus 2 to give space for cursor with multiline Lines

    let mut args = env::args();
    args.next(); // Skip unnecessary arg

    let mut buf = String::new(); // This buffer is used to read the document in, but then later to act as a buffer for user input
    let mut document: Document; // This is the variable that will hold this document to be edited

    clear_screen(); // Clear screen for the editor
    let mut cursor = Cursor::new(editor_top, 1); // The cursor that will be used for all drawing, start at column = 1 because otherwise it will not move correctly

    if let Some(file_name) = args.next() {
        // If a file has been provided through command line
        let mut in_file = File::open(&file_name).unwrap();

        in_file.read_to_string(&mut buf).unwrap();

        // Create document struct instance from file and editor width
        document = Document::new(file_name, buf.clone(), editor_width);

        // Move cursor to home to print file name
        move_cursor_home();
        print!("{}", &document.file_name);

        // Display document
        display_document(&document, editor_width, &mut cursor);
    } else {
        // No file name provided
        todo!("Implement scratch buffer");
        // Create new empty document with default name scratch
        document = Document::new("scratch".to_string(), "".to_string(), editor_width);

        move_cursor_home();
        // Print scratch to screen instead of file name
        print!("[ scratch ]");
    }

    // Print the mode to the screen, in this case, the default is normal
    move_cursor_to(0, mode_row);
    print!("NOR");

    #[cfg(target_os = "linux")]
    set_raw();

    // Here, cursor_x is initially set to 1 as setting it to 0 would require the user to press l multiple times to move away from the left barrier
    cursor.move_to(editor_top, 1);

    // Initialize the gap buffer, it will be replaced later when editing actual text
    let mut gap_buf = GapBuf::new();

    // Clear the buffer
    buf.clear();

    // Stores the state of the mode for the program
    let mut mode = Modes::Normal;

    log_file
        .write(format!("Number of rows{:?}", 2..document.num_rows()).as_bytes())
        .unwrap();

    loop {
        match get_char() as u8 {
            J_LOWER if mode == Modes::Normal => {
                // Move down
                // Check that the cursor's row field is less than or equal to the number of *rows* not *Lines* in the document
                if cursor.row <= document.num_rows() {
                    // Store the original string that the cursor is at now
                    let original = document.get_str_at_cursor(cursor.row);

                    if cursor.column > original.len() {
                        // If the cursor's column field is at the very end of the current line, move the cursor down and to the end of the next line
                        cursor.move_down();

                        // Move the cursor to the end of the line
                        cursor
                            .move_to(cursor.row, document.get_str_at_cursor(cursor.row).len() + 1);
                    } else if document.get_str_at_cursor(cursor.row + 1).len() < cursor.column {
                        // If the cursor is within the original line but outside of the next line
                        cursor.move_down();

                        // Move the cursor to the end of the line
                        cursor
                            .move_to(cursor.row, document.get_str_at_cursor(cursor.row).len() + 1);
                    } else {
                        // If the cursor is within the current line and the next line
                        cursor.move_down();
                    }

                    if original == document.get_str_at_cursor(cursor.row)
                        && cursor.row - 1 <= document.num_rows()
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
                    if cursor.column <= curr_line.1.len() {
                        // Until the cursor's column field is at most the length of the text
                        cursor.move_right();
                    }
                } else {
                    // If the line spans more than one roww
                    if cursor.column <= editor_width + 1
                        && cursor.get_position_in_line(&document, editor_width) < curr_line.1.len()
                    {
                        // If the cursor's column is at most one more than the right edge of the editor, and it is still less than the length of the current line
                        cursor.move_right()
                    } else if curr_line.1 == document.get_str_at_cursor(cursor.row + 1)
                        && cursor.get_position_in_line(&document, editor_width) <= curr_line.1.len()
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
                if cursor.row - 1 >= editor_top && document.get_line_at_cursor(cursor.row).0[0] != 0
                {
                    // If moving the cursor up 1 is at most the editor top
                    // Move up
                    cursor.move_up();

                    if document.get_line_at_cursor(cursor.row).0.len() > 1 {
                        let num_indices = document.get_line_at_cursor(cursor.row).0.len();

                        for _ in 1..num_indices {
                            cursor.move_up();
                        }
                    }

                    if cursor.column > document.get_str_at_cursor(cursor.row).len() {
                        // If moving the cursor down moves the cursor out of bounds of the next line
                        cursor.column = document.get_str_at_cursor(cursor.row).len() + 1;
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
                } else if curr_line.0.len() > 1 && (cursor.row - 2) != *curr_line.0.first().unwrap()
                {
                    // If moving the cursor left would mean wrapping to the very end of the previous row of the same line, move the cursor up and to the end of the editor screen
                    cursor.move_up();
                    cursor.move_to(cursor.row, editor_width);
                }
            }
            G_LOWER if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::MoveTo, mode_row, 0, &cursor);

                if get_char() == 'l' {
                    cursor.move_to_end_line(&document, editor_width);

                    change_mode(&mut mode, Modes::Normal, mode_row, 0, &cursor);
                } else {
                    change_mode(&mut mode, Modes::Normal, mode_row, 0, &cursor);
                }
            }
            X_LOWER if mode == Modes::Normal => {
                todo!("Implement deleting lines for multiline Lines");
                if get_char() == 'd' {
                    // The key combination xd will delete a line
                    // Remove the line from the document
                    document.remove_index_from_line(cursor.row);

                    // Save current position
                    cursor.save_current_pos();

                    reset_editor_view(&document, editor_width, editor_bottom, &mut cursor);

                    // Return to previous position
                    cursor.revert_pos();
                    // Move to left edge of editor
                    cursor.move_to_left_border();
                }
            }
            I_LOWER if mode == Modes::Normal => {
                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, mode_row, 0, &cursor);

                // Create a new gap buffer from the string at the current cursor position
                gap_buf = GapBuf::from_str(
                    document.get_str_at_cursor(cursor.row),
                    cursor.get_position_in_line(&document, editor_width) - 1, // Needs to be decremented to make the space directly before the white block cursor the "target"
                );
            }
            O_LOWER if mode == Modes::Normal => {
                // Create a new empty line
                let mut new_line = Line::new();
                let curr_line_inds = document.get_line_at_cursor(cursor.row).0;

                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, dimensions.height - 1, 0, &cursor);

                // Move down to the new row
                cursor.move_down();
                // Move to the left edge of the editor
                cursor.move_to_left_border();

                new_line
                    .0
                    .push(curr_line_inds[curr_line_inds.len() - 1] + 1);

                document.add_line_at_row(new_line, cursor.row);

                // Crate an empty gap buffer
                gap_buf = GapBuf::new();

                cursor.save_current_pos();

                reset_editor_view(&document, editor_width, editor_bottom, &mut cursor);

                cursor.revert_pos();
            }
            ESC if mode == Modes::Insert => {
                // Change mode to normal
                change_mode(&mut mode, Modes::Normal, dimensions.height - 1, 0, &cursor);

                // Make the edits persist in memory
                document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);
            }
            ESC if mode == Modes::Command => {
                // Change mode to normal
                change_mode(&mut mode, Modes::Normal, dimensions.height - 1, 0, &cursor);

                // Move cursor to the command line row
                cursor.move_to(dimensions.height, 0);

                // Visually delete the contents of the row
                print!("{: >1$}", "", dimensions.width);

                // The cursor position was saved when switching to command mode, so revert to that position
                cursor.revert_pos();

                // Clear the buffer
                buf.clear();
            }
            BCKSP if mode == Modes::Insert => {
                if cursor.column - 1 > 0 {
                    // If the cursor's column after moving to the left is greater than 0
                    gap_buf.pop(); // Remove character from gap buffer
                    cursor.move_left();

                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    cursor.save_current_pos();

                    reset_editor_view(&document, editor_width, editor_bottom, &mut cursor);

                    cursor.revert_pos();
                } else {
                    // cursor.column == 1
                    // Append the current line to the previous line
                    let curr_line = document.get_line_at_cursor(cursor.row);

                    // TODO: Implement checks for when deleting to beginning of first line

                    if document.lines.len() > 1 {
                        // If the document contains more than one line

                        if curr_line.1.len() > 0 && (cursor.row - 2) == curr_line.0[0] {
                            // If the current line's string's length is greater than 0 (not empty) and the cursor's row is equal to the first index of the line
                            // This is the branch handling moving the contents of a string which is not fully deleted into the line above it

                            let removal_ind = cursor.row;

                            document.remove_line_from_doc(removal_ind);

                            // Move to where the line will be appended to, note, this is assumed to be the very beginning of the line, so moving up would be guaranteed to be the previous line
                            cursor.move_up();

                            cursor.move_to_end_line(&document, editor_width);

                            cursor.save_current_pos(); // Will move back to after visual clean up

                            // Create a new string based on the line to be appended to the current line and the current line (current line here does *NOT* refer to the variable curr_line)
                            let mut new_str = String::with_capacity(
                                document.get_str_at_cursor(cursor.row).len() + curr_line.1.len(),
                            );

                            // Add the contents of the current line and the string content of the curr_line variable in that order
                            new_str += &document.get_str_at_cursor(cursor.row);
                            new_str += &curr_line.1;

                            // Go through the motions of creating a new line from the current line, see document.rs
                            document.set_line_at_cursor(cursor.row, new_str, editor_width);

                            // Create a new gap buffer from the newly created line
                            gap_buf = GapBuf::from_line(
                                document.get_line_at_cursor(cursor.row),
                                cursor.column - 1,
                            );

                            reset_editor_view(&document, editor_width, editor_bottom, &mut cursor);

                            cursor.revert_pos();
                        } else {
                            // Either the current line's content is not greater than 0 or the cursor's row is not at the first index of the line
                            if curr_line.1.len() > 0 && (cursor.row + 2) != curr_line.0[0] {
                                // The current line spans more than one line, there is more content in the string, and the cursor's row is not at the first index of the Line
                                cursor.move_up();

                                gap_buf.pop();

                                document.set_line_at_cursor(
                                    cursor.row,
                                    gap_buf.to_string(),
                                    editor_width,
                                );

                                cursor.save_current_pos();

                                reset_editor_view(
                                    &document,
                                    editor_width,
                                    editor_bottom,
                                    &mut cursor,
                                );

                                cursor.revert_pos();

                                cursor.move_to_end_line(&document, editor_width);
                            } else {
                                document.remove_line_from_doc(cursor.row);

                                cursor.move_up();

                                cursor.save_current_pos();

                                reset_editor_view(
                                    &document,
                                    editor_width,
                                    editor_bottom,
                                    &mut cursor,
                                );

                                cursor.revert_pos();

                                cursor.move_to_end_line(&document, editor_width);
                            }
                        }
                    }
                }

                debug_log_document(&document, &mut log_file);
            }
            c if mode == Modes::Insert => {
                if cursor.column <= editor_width {
                    // Inserting a character won't spill onto next line
                    // Insert the character in memory
                    gap_buf.insert(c as char);
                    // Move the cursor to where it would be normally after inserting a character
                    cursor.move_right();

                    // Save position to return back to
                    cursor.save_current_pos();

                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    reset_editor_view(&document, editor_width, editor_bottom, &mut cursor);

                    // Go back to where the cursor should be after inserting
                    cursor.revert_pos();
                } else {
                    // Inserting would cause wrapping around
                    // Insert into gap buffer as normal
                    gap_buf.insert(c as char);

                    // Since there is no undoing yet, just set the current line to the gap buffer as things are inserted, gap buffer is still necessary
                    // for ease of editing
                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    // Move to next line
                    cursor.move_down();
                    // Move to the left edge of the screen
                    cursor.move_to_left_border();
                    cursor.move_right();

                    // Save position to come back to
                    cursor.save_current_pos();

                    // Clear the screen to reprint document
                    reset_editor_view(&document, editor_width, editor_bottom, &mut cursor);

                    cursor.revert_pos();
                }
            }
            COLON if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::Command, dimensions.height - 1, 0, &cursor);

                // Clear the buffer to ensure the new command will be empty
                buf.clear();

                // Save cursor position to come back to
                cursor.save_current_pos();

                // Move cursor to command row
                cursor.move_to(command_row, 1);
                // Clear the line if something was already printed there
                print!("{: >1$}", "", dimensions.width);
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
                    print!("{: >1$}", "", dimensions.width);

                    change_mode(&mut mode, Modes::Normal, dimensions.height - 1, 0, &cursor);

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
                    print!("{: <1$}", "invalid command", dimensions.width);

                    change_mode(&mut mode, Modes::Normal, dimensions.height - 1, 0, &cursor);

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

    #[cfg(target_os = "linux")]
    set_cooked();
}
