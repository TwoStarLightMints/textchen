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
fn change_mode(curr: &mut Modes, new_mode: Modes, mode_row: usize, cursor: &mut Cursor) {
    //! curr - Current mode stored in the state of the application
    //! new_mode - The new mode which will be stored in the state of the application
    //! mode_row - The row at which the mode will be printed
    //! cursor - Get control of cursor
    //!
    //! Changes the current mode of the editor to a new target mode, handles changing state and drawing to screen

    *curr = new_mode;

    // This is used here instead of save position as it messes with something outside of this scope
    let curr_row = cursor.row;
    let curr_col = cursor.column;

    cursor.move_to(mode_row, 0);

    match curr {
        Modes::Normal => print!("NOR"),
        Modes::Insert => print!("INS"),
        Modes::Command => print!("COM"),
        Modes::MoveTo => print!("MOV"),
    };

    io::stdout().flush().unwrap();

    cursor.move_to(curr_row, curr_col);
}

fn display_line(
    editor_left_edge: usize,
    editor_width: usize,
    document: &Document,
    cursor: &mut Cursor,
) {
    let curr_row = cursor.row;
    let curr_col = cursor.column;

    let line = document.get_line_at_cursor(cursor.row);

    cursor.move_to_start_line(document, editor_left_edge);

    for (ind, char) in line.1.chars().enumerate() {
        print_flush(format!("{char}").as_str());

        if ind != 0 && (ind + 1) % editor_width == 0 && ind != line.1.len() - 1 {
            cursor.move_down();
            cursor.move_to_editor_left(editor_left_edge);
        }
    }

    cursor.move_to(curr_row, curr_col);
}

fn display_document(
    document: &Document,
    editor_left_edge: usize,
    editor_width: usize,
    cursor: &mut Cursor,
) {
    //! document - Document being edited
    //! editor_left_edge - This is the offset from the left side of the terminal
    //! editor_width - Size of the editor screen, calculated from the left side offset and the right side offset, pass this calculated result to the function
    //! cursor - Get control of cursor
    //!
    //! Displays the document that is currently being edited to the screen, handles drawing within given bounds

    // This is used here instead of save position as it messes with something outside of this scope
    let curr_row = cursor.row;
    let curr_col = cursor.column;

    cursor.move_to(2, editor_left_edge);

    for line in document.lines.iter() {
        for (ind, char) in line.1.chars().enumerate() {
            print_flush(format!("{char}").as_str());

            if ind != 0 && (ind + 1) % editor_width == 0 && ind != line.1.len() - 1 {
                cursor.move_down();
                cursor.move_to_editor_left(editor_left_edge);
            }
        }

        cursor.move_down();
        cursor.move_to_editor_left(editor_left_edge);
    }

    cursor.move_to(curr_row, curr_col);
}

fn clear_line(
    editor_left_edge: usize,
    editor_width: usize,
    document: &Document,
    cursor: &mut Cursor,
) {
    let curr_row = cursor.row;
    let curr_col = cursor.column;

    cursor.move_to_start_line(document, editor_left_edge);

    for _ in document.get_line_at_cursor(cursor.row).0 {
        print!("{: <1$}", "", editor_width);
    }

    cursor.move_to(curr_row, curr_col);
}

fn clear_editor_window(editor_right_edge: usize, document: &Document, cursor: &mut Cursor) {
    //! editor_right_edge - This is the offset from the right side of the terminal
    //! document - Document being edited
    //! cursor - Get control of cursor
    //!
    //! Visually clears the contents of the editor window, the rest of the screen is untouched

    // This is used here instead of save position as it messes with something outside of this scope
    let curr_row = cursor.row;
    let curr_col = cursor.column;

    cursor.move_to(2, 1);

    for _ in 0..=document.num_rows() {
        // print!("\u{001b}[2K");
        print!("{: >1$}", "", editor_right_edge);
        cursor.move_down();
    }

    cursor.move_to(curr_row, curr_col);
}

fn reset_line_view(
    editor_left_edge: usize,
    editor_width: usize,
    document: &Document,
    cursor: &mut Cursor,
) {
    let curr_row = cursor.row;
    let curr_col = cursor.column;

    clear_line(editor_left_edge, editor_width, document, cursor);

    display_line(editor_left_edge, editor_width, document, cursor);

    cursor.move_to(curr_row, curr_col);
}

fn reset_editor_view(
    doc: &Document,
    editor_left_edge: usize,
    editor_right_edge: usize,
    cursor: &mut Cursor,
) {
    //! editor_right_edge - This is the offset from the right side of the terminal
    //! editor_left_edge - This is the offset from the left side of the terminal
    //! document - Document being edited
    //! cursor - Get control of cursor
    //!
    //! Clears the editor screen and redraws the document provided, tends to be used as to refresh the screen after an edit has occurred

    clear_editor_window(editor_right_edge, doc, cursor);

    display_document(
        doc,
        editor_left_edge,
        editor_right_edge - editor_left_edge,
        cursor,
    );
}

#[allow(dead_code)]
fn debug_log_message(message: String, log_file: &mut File) {
    log_file.write(message.as_bytes()).unwrap();
}

#[allow(dead_code)]
fn debug_log_document(doc: &Document, log_file: &mut File) {
    doc.lines.iter().for_each(|l| {
        log_file
            .write(format!("Line indices: {:?}, String content: {}\n", l.0, l.1).as_bytes())
            .unwrap();
    })
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn debug_log_gapbuffer(gap_buf: &GapBuf, log_file: &mut File) {
    log_file
        .write(format!("Lhs: {:?}, Rhs: {:?}\n", gap_buf.lhs, gap_buf.rhs).as_bytes())
        .unwrap();
}

// ==== ASCII KEY CODE VALUES ====
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
// ==== ASCII KEY CODE VALUES ====

fn main() {
    // Used to log debug info to
    #[allow(unused_variables, unused_mut)]
    let mut log_file = File::create("log.txt").unwrap();

    // Dimensions for the terminal screen
    // Wh.width - The width of the terminal as a whole
    // Wh.height - The height of the terminal as a whole
    let dimensions = term_size();

    // Title row is the home row
    // row: 0, column: 0

    // The second from the first line is the first line of the editor screen

    let editor_top = 2;

    // This variable defines where to start the editor's left edge, 1 is the minimum value
    // for this

    let editor_left_edge = 2;

    // This variable defines where to end the editor's screen, while using the full value
    // of dimensions.width, I reccomend decreasing this value

    let editor_right_edge = dimensions.width - 2;

    // This variable holds the length that the editor screen spans, calculated from the
    // editor_left_edge and editor_right_edge variables
    // The width of the editor (from the left side of the terminal to at most this value), minus 2 to give space for cursor with multiline Lines

    let editor_width = editor_right_edge - editor_left_edge;

    // This variable defines the row at which the mode will be displayed to the user,
    // conventionally this is the second to last row of the terminal which is dimension.height - 1
    // Second to last line, where mode is shown

    let mode_row = dimensions.height - 1;

    // This variable is simply a label for the full height of the terminal, it is dimensions.height
    // Last line, where commands will be written to

    let command_row = dimensions.height;

    // This variable holds a tuple containing the coordinates of the editor's home,
    // this is a wrapper

    let editor_home: (usize, usize) = (editor_top, editor_left_edge);

    // Get the command line arguments to the program

    let mut args = env::args();

    // Skip the first argument, this is unnecessary to the program

    args.next();

    // This is the buffer which will hold the document's raw string content and user commands

    let mut buf = String::new();

    // This variable is like a more structured buffer for the whole document

    let mut document: Document;

    // Prep the screen to draw the editor and the document to the screen

    clear_screen();

    // This cursor will be the cursor used throughout the document to draw and access elements from the
    // document

    let mut cursor = Cursor::new(editor_home.0, editor_home.1);

    if let Some(file_name) = args.next() {
        // If a file has been provided through command line

        // Open the file
        let mut in_file = File::open(&file_name).unwrap();

        // Read the file contents into the buffer
        in_file.read_to_string(&mut buf).unwrap();

        // Create document struct instance from file contents and editor width
        document = Document::new(file_name, buf.clone(), editor_width);

        // Move cursor to home to print file name
        move_cursor_home();
        print!("{}", &document.file_name);

        // Display document
        display_document(&document, editor_left_edge, editor_width, &mut cursor);
    } else {
        // No file name provided

        // Create new empty document with default name scratch
        document = Document::new("scratch".to_string(), "".to_string(), editor_right_edge);

        // Move cursor to home to print file name
        move_cursor_home();

        // Print scratch to screen instead of file name
        print!("[ scratch ]");

        todo!("Implement scratch buffer");
    }

    // Print the mode to the screen, in this case, the default is normal
    move_cursor_to(0, mode_row);
    print!("NOR");

    // Set the terminal to raw input mode, this is only possible and needed on linux systems
    #[cfg(target_os = "linux")]
    set_raw();

    // Move the cursor to the editor home
    cursor.move_to(editor_home.0, editor_home.1);

    // Initialize the gap buffer, it will be replaced later when editing actual text
    let mut gap_buf = GapBuf::new();

    // Clear the buffer
    buf.clear();

    // Stores the state of the mode for the program, starts with Modes::Normal
    let mut mode = Modes::Normal;

    // Main loop for program
    loop {
        // Get a character and match it aginst some cases as a u8
        match get_char() as u8 {
            // Move down
            J_LOWER if mode == Modes::Normal => {
                if cursor.row <= document.num_rows() {
                    // If the cursor's row is at most equal to the number of rows in the document, see docs for differnce between rows and Lines

                    // Store the position of the cursor in the original line, save on method calls
                    let cursor_pos =
                        cursor.get_position_in_line(&document, editor_left_edge, editor_width);

                    let curr_line = document.get_line_at_cursor(cursor.row);

                    if cursor_pos > document.get_str_at_cursor(cursor.row + 1).len() {
                        // If the current position of the cursor is greater than the length of the next line

                        // Move the cursor down
                        cursor.move_down();

                        // Move to the end of the line
                        cursor.move_to_end_line(&document, editor_left_edge, editor_width);
                    } else if curr_line.0.len() > 1
                        && cursor_pos / editor_width != curr_line.0[curr_line.0.len() - 1]
                    {
                        // If the current line's number of contained indices is greater than 1 and the cursor's position divided by the editor's width (which yields the index of the current row of the cursor in the document)
                        // is not equal to the last index within the line's indices vector

                        for _ in 0..(curr_line.0.len() - (cursor_pos / editor_width)) {
                            cursor.move_down();
                        }

                        // Move to the equivalent of the current position in the next line, check method definition for explaination of the logic
                        cursor.move_to_pos_in_line(
                            &document,
                            editor_left_edge,
                            editor_width,
                            cursor_pos,
                        );
                    } else {
                        // If the current position of the cursor is within the length of the next line

                        // Move to the next line
                        cursor.move_down();

                        // Move to the equivalent of the current position in the next line, check method definition for explaination of the logic
                        cursor.move_to_pos_in_line(
                            &document,
                            editor_left_edge,
                            editor_width,
                            cursor_pos,
                        );
                    }
                }
            }
            // Move right
            L_LOWER if mode == Modes::Normal => {
                // Get the current line where the cursor is at
                let curr_line = document.get_line_at_cursor(cursor.row);
                let cursor_pos =
                    cursor.get_position_in_line(&document, editor_left_edge, editor_width);

                if cursor_pos < curr_line.1.len()
                    && (cursor_pos % editor_width != 0 || cursor_pos == 0)
                {
                    // If the cursor's position in the current line is less than the length of the total line and either the cursor's position mod the editor's width is not 0
                    // and the cursor's position is 0

                    cursor.move_right();
                } else if cursor_pos < curr_line.1.len()
                    && cursor_pos / editor_width < curr_line.0.len()
                {
                    // If the cursor's position in the current line is less than the length of the total line and the cursor's position vertically within the line is not the
                    // last possible row in the line (cursor_pos / editor_width will give the index of the current row of the cursor, therefore the length of the indices of
                    // the current line can be used as the non-inclusive max)

                    // Move down to the next row
                    cursor.move_down();

                    // Move to the left edge of the editor
                    cursor.move_to_editor_left(editor_left_edge);

                    // Because the end of the previous line is included within the conditions of the previous if clause, move the cursor to the right of the immediate next
                    // chracter in the line
                    cursor.move_right();
                }
            }
            // Move up
            K_LOWER if mode == Modes::Normal => {
                if cursor.row - 1 >= editor_top && document.get_line_at_cursor(cursor.row).0[0] != 0
                {
                    // If moving the cursor up 1 is at most the editor's top line and (if the line is a multiline) the first index in the line's row indices is not 0 (i.e. the
                    // line is not the very first line)

                    // Get the current position of the cursor
                    let cursor_pos =
                        cursor.get_position_in_line(&document, editor_left_edge, editor_width);

                    // Since the cursor's position divided by the editor width is the index of the row at which the cursor lies within the line, the cursor must move at least
                    // that many times to move out of the current line upwards, yet since the vector is 0 indexed, one must add 1 to the result
                    for _ in 0..((cursor_pos / editor_width) + 1) {
                        cursor.move_up();
                    }

                    cursor.move_to_pos_in_line(
                        &document,
                        editor_left_edge,
                        editor_width,
                        cursor_pos,
                    );
                }
            }
            // Move left
            H_LOWER if mode == Modes::Normal => {
                let cursor_pos =
                    cursor.get_position_in_line(&document, editor_left_edge, editor_width);

                if cursor.get_column_in_editor(editor_left_edge) > 1 || cursor_pos == 1 {
                    // If moving the cursor left does not reach the first column of the editor's field (i.e. the cursor will not be moved to the first possible column where characters can be printed to)
                    // or the cursor is at the second position of the line

                    cursor.move_left()
                } else if cursor_pos / editor_width != 0 && cursor_pos != 0 {
                    // If the row in the line where the cursor is is not the first row of the line and the cursor is not at the first position of the line

                    cursor.move_up();
                    cursor.move_to(cursor.row, editor_right_edge);
                }
            }
            G_LOWER if mode == Modes::Normal => {
                change_mode(&mut mode, Modes::MoveTo, mode_row, &mut cursor);

                let new_c = get_char();

                if new_c == 'l' {
                    cursor.move_to_end_line(&document, editor_left_edge, editor_width);

                    change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);
                } else if new_c == 'g' {
                    cursor.move_to(editor_home.0, editor_home.1);

                    change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);
                } else if new_c == 'e' {
                    cursor.move_to(
                        document.lines.last().unwrap().0.last().unwrap() + 2,
                        editor_left_edge,
                    );

                    change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);
                } else {
                    change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);
                }
            }
            X_LOWER if mode == Modes::Normal => {
                if get_char() == 'd' {
                    // The key combination xd will delete a line
                    // Remove the line from the document
                    document.remove_index_from_line(cursor.row);

                    // Save current position
                    cursor.save_current_pos();

                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);

                    // Return to previous position
                    cursor.revert_pos();
                    // Move to left edge of editor
                    cursor.move_to_left_border();
                }

                todo!("Check over deleting a full line from document");
            }
            // Enter insert mode
            I_LOWER if mode == Modes::Normal => {
                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, mode_row, &mut cursor);

                // Create a new gap buffer from the string at the current cursor position
                gap_buf = GapBuf::from_line(
                    document.get_line_at_cursor(cursor.row),
                    cursor.get_position_in_line(&document, editor_left_edge, editor_width),
                );
            }
            O_LOWER if mode == Modes::Normal => {
                // Create a new empty line
                let mut new_line = Line::new();
                let curr_line_inds = document.get_line_at_cursor(cursor.row).0;

                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, mode_row, &mut cursor);

                // Move down to the new row
                cursor.move_down();
                // Move to the left edge of the editor
                cursor.move_to_editor_left(editor_left_edge);

                new_line
                    .0
                    .push(curr_line_inds[curr_line_inds.len() - 1] + 1);

                document.add_line_at_row(new_line, cursor.row);

                // Crate an empty gap buffer
                gap_buf = GapBuf::new();

                reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);

                todo!("Check over inserting new line after cursor");
            }
            // Exit insert mode
            ESC if mode == Modes::Insert => {
                // Change mode to normal
                change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);

                // Set the the to the string representation of the current gap buffer, reculculating the row indices for the line
                document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);
            }
            // Cancel entering a command
            ESC if mode == Modes::Command => {
                // Change mode to normal
                change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);

                // Move cursor to the command line row
                cursor.move_to(dimensions.height, 0);

                // Visually delete the contents of the row
                print!("{: >1$}", "", dimensions.width);

                // The cursor position was saved when switching to command mode, so revert to that position
                cursor.revert_pos();

                // Clear the buffer
                buf.clear();
            }
            // Delete a character while in insert mode
            BCKSP if mode == Modes::Insert => {
                let cursor_pos =
                    cursor.get_position_in_line(&document, editor_left_edge, editor_width);

                todo!("Continue reimplementing deleting characters");

                if cursor.get_column_in_editor(editor_left_edge) > 1 || cursor_pos == 1 {
                    gap_buf.pop();
                    cursor.move_left();

                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    reset_line_view(editor_left_edge, editor_width, &document, &mut cursor);
                } else if cursor_pos / editor_width != 0 && cursor_pos != 0 {
                    gap_buf.pop();
                    cursor.move_up();
                    cursor.move_to_end_line(&document, editor_left_edge, editor_width);
                }

                // if cursor.get_column_in_editor(editor_left_edge) > 0 {
                //     // If the cursor's column after moving to the left is greater than 0
                //     gap_buf.pop(); // Remove character from gap buffer
                //     cursor.move_left();

                //     let num_rows = document.num_rows();
                //     let curr_line = document.get_line_at_cursor(cursor.row);

                //     document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                //     if num_rows > document.num_rows() && curr_line.0.len() > 1 {
                //         // If the number of rows collected before altering the current string is greater than the rows now, move cursor up and to the end of the string
                //         cursor.move_up();
                //         cursor.move_to_end_line(&document, editor_left_edge, editor_width);

                //         cursor.save_current_pos();

                //         reset_editor_view(
                //             &document,
                //             editor_left_edge,
                //             editor_right_edge,
                //             &mut cursor,
                //         );

                //         cursor.revert_pos();
                //     } else {
                //         cursor.save_current_pos();

                //         reset_editor_view(
                //             &document,
                //             editor_left_edge,
                //             editor_right_edge,
                //             &mut cursor,
                //         );

                //         cursor.revert_pos();

                //         cursor.move_to_end_line(&document, editor_left_edge, editor_width);
                //     }
                // } else {
                //     // cursor.column == 1
                //     // Append the current line to the previous line
                //     let curr_line = document.get_line_at_cursor(cursor.row);

                //     if document.lines.len() > 1 && cursor.row != editor_top {
                //         // There is more than one line remaining in the document, therefore after deleting this line there is one to take its place, and it cannot be the top line

                //         if curr_line.1.len() == 0 {
                //             document.remove_line_from_doc(cursor.row);

                //             cursor.move_up();

                //             cursor.save_current_pos();

                //             reset_editor_view(
                //                 &document,
                //                 editor_left_edge,
                //                 editor_right_edge,
                //                 &mut cursor,
                //             );

                //             cursor.revert_pos();

                //             cursor.move_to_end_line(&document, editor_left_edge, editor_width);

                //             // If the gap buffer is not reset here, the program will thing that each consecutive new line is also a blank line and delete it immaturely
                //             gap_buf = GapBuf::from_line(
                //                 document.get_line_at_cursor(cursor.row),
                //                 cursor.get_position_in_line(
                //                     &document,
                //                     editor_left_edge,
                //                     editor_width,
                //                 ),
                //             );
                //         }
                //     }
                // }
                // todo!("Take a look at deleting a character using backspace");
            }
            // Insert a new line character to break line while in insert mode
            c if mode == Modes::Insert && c as char != '\n' => {
                if cursor.get_column_in_editor(editor_left_edge) < editor_width {
                    // Inserting a character won't spill onto next line
                    // Insert the character in memory
                    gap_buf.insert(c as char);
                    // Move the cursor to where it would be normally after inserting a character
                    cursor.move_right();
                    // Save position to return back to
                    cursor.save_current_pos();

                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_right_edge);

                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);

                    // Go back to where the cursor should be after inserting
                    cursor.revert_pos();
                } else {
                    // Inserting would cause wrapping around
                    // Insert into gap buffer as normal
                    gap_buf.insert(c as char);

                    debug_log_cursor(&cursor, &mut log_file);

                    // Since there is no undoing yet, just set the current line to the gap buffer as things are inserted, gap buffer is still necessary
                    // for ease of editing
                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_right_edge);

                    // Move to next line
                    cursor.move_down();
                    // Move to the left edge of the screen
                    cursor.move_to_end_line(&document, editor_left_edge, editor_width);

                    // Clear the screen to reprint document
                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
                }

                todo!("Check inserting new character while in insert mode");
            }
            // Insert a character while in insert mode
            c if mode == Modes::Insert && c as char == '\n' => {
                let (lhs, rhs) = gap_buf.collect_to_pieces();

                document.set_line_at_cursor(cursor.row, lhs, editor_right_edge);

                let newly_made_at_row_inds = document.get_line_at_cursor(cursor.row).0;

                // newly_made_at_row_inds stores the amount of rows spanned by the lhs after inserting it properly into the document therefore to insert
                // the rhs after this newly made row, simply move the cursor down 0 to the length of the newly_made_at_row_inds vector

                for _ in
                    0..=(cursor.get_position_in_line(&document, editor_left_edge, editor_width)
                        / editor_width)
                {
                    cursor.move_down();
                }

                let mut new_line = Line::new();
                new_line
                    .0
                    .push(newly_made_at_row_inds[newly_made_at_row_inds.len() - 1]);

                log_file
                    .write(format!("New line: {:?}\n", new_line).as_bytes())
                    .unwrap();

                document.lines.insert(cursor.row - 2, new_line);

                cursor.move_to_editor_left(editor_left_edge);

                document.set_line_at_cursor(cursor.row, rhs, editor_right_edge);

                gap_buf = GapBuf::from_line(document.get_line_at_cursor(cursor.row), 0);

                reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);

                todo!("Check inserting a new line character while in insert mode");
            }
            // Enter command mode
            COLON if mode == Modes::Normal => {
                // Change to command mode
                change_mode(&mut mode, Modes::Command, mode_row, &mut cursor);

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
            // Execute command while in command mdoe
            RETURN if mode == Modes::Command => match buf.as_str() {
                "w" => {
                    let mut out_file = File::create(&document.file_name).unwrap();

                    out_file.write(document.to_string().as_bytes()).unwrap();

                    cursor.move_to(command_row, 0);
                    print!("{: >1$}", "", dimensions.width);

                    change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);

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

                    change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);

                    cursor.revert_pos();

                    buf.clear();
                }
            },
            // Insert character while in command mode
            c if mode == Modes::Command => {
                // Push the pressed character to the buffer
                buf.push(c as char);

                // Display the character to the screen, stdout will be flush on cursor move
                print!("{}", c as char);

                cursor.move_right();
            }

            _ => (),
        }
    }

    // Similar to set_raw, only used/needed on linux
    #[cfg(target_os = "linux")]
    set_cooked();
}
