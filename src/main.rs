use std::env;
use std::fs::{self, File};
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
    document: &Document,
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

    clear_editor_window(editor_right_edge, document, cursor);

    display_document(
        document,
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
fn debug_log_document(document: &Document, log_file: &mut File) {
    document.lines.iter().for_each(|l| {
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
const G_LOWER: u8 = 103;
const I_LOWER: u8 = 105;
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

    // Prep the screen to draw the editor and the document to the screen, switching to alt buffer to not erase entire screen

    switch_to_alt_buf();
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

        debug_log_document(&document, &mut log_file);
        log_file
            .write(format!("{:?}", document).as_bytes())
            .unwrap();

        // Move cursor to home to print file name
        move_cursor_home();

        // Print scratch to screen instead of file name
        print!("scratch");
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
                        cursor.move_to_end_line(
                            &document,
                            editor_left_edge,
                            editor_width,
                            editor_top,
                        );
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
                            editor_top,
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
                            editor_top,
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
                        editor_top,
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
                    cursor.move_to_end_line(&document, editor_left_edge, editor_width, editor_top);

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
                    cursor.move_to_editor_left(editor_left_edge);
                }

                todo!("Check over deleting a full line from document");
            }
            // Enter insert mode
            I_LOWER if mode == Modes::Normal => {
                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, mode_row, &mut cursor);

                // Create a new gap buffer from the string at the current cursor position
                gap_buf = GapBuf::from_str(
                    document.get_str_at_cursor(cursor.row),
                    cursor.get_position_in_line(&document, editor_left_edge, editor_width),
                );
            }
            O_LOWER if mode == Modes::Normal => {
                // Create a new empty line
                let mut new_line = Line::new();

                // Collect the current line's indices
                let curr_line_inds = document.get_line_at_cursor(cursor.row).0;

                // Change mode to insert
                change_mode(&mut mode, Modes::Insert, mode_row, &mut cursor);

                // Add the last index of the current line incremented to the new line's index list
                new_line
                    .0
                    .push(curr_line_inds[curr_line_inds.len() - 1] + 1);

                // Move to the beginning of the next possible line
                cursor.move_to_end_line(&document, editor_left_edge, editor_width, editor_top);
                cursor.move_down();
                cursor.move_to_editor_left(editor_left_edge);

                // Add the new line to the document
                document.add_line_at_row(new_line, cursor.row);

                // Crate an empty gap buffer since the line will be empty guaranteed
                gap_buf = GapBuf::new();

                // Reset view
                reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
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

                // todo!("Continue reimplementing deleting characters and check formatting");

                if cursor.get_column_in_editor(editor_left_edge) > 1 || cursor_pos == 1 {
                    // If the cursor is one space away from being on top of the first column of characters (i.e. the cursor is within the line)

                    // Remove the next character in the gap buffer
                    gap_buf.pop();

                    // Only move the cursor to the left
                    cursor.move_left();

                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
                } else if cursor_pos / editor_width != 0 {
                    // If the cursor is not in the first row of the line

                    // Remove the next character in the gap buffer
                    gap_buf.pop();

                    // Move the cursor to the previous row
                    cursor.move_up();

                    // Move the cursor to the end of the previous row
                    cursor.move_to_editor_right(editor_right_edge);

                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    // Reset the view
                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
                } else if cursor_pos == 0 && cursor.row != editor_top {
                    // If the cursor is at the first positon of the line and it is not in the first line of the document (note: the cursor's row field is not subtracted by 2 during checking because editor_top starts at the same index that cursor's row starts at)

                    // Get the current line's string
                    let curr_str = document.get_str_at_cursor(cursor.row);

                    // Remove the current line from the document
                    document.remove_line_from_doc(cursor.row);

                    // Move to the previous line
                    cursor.move_up();

                    // Move to the end of the previous line
                    cursor.move_to_end_line(&document, editor_left_edge, editor_width, editor_top);

                    // Set the previous line's string value to its current string appended with the contents of the current line string
                    document.set_line_at_cursor(
                        cursor.row,
                        document.get_str_at_cursor(cursor.row) + &curr_str,
                        editor_width,
                    );

                    // Create a new gap buffer based on the new string at the cursor position
                    gap_buf = GapBuf::from_str(
                        document.get_str_at_cursor(cursor.row),
                        cursor.get_position_in_line(&document, editor_left_edge, editor_width),
                    );

                    // Reset the view
                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
                }
            }
            // Insert a new line character to break line while in insert mode
            c if mode == Modes::Insert && c != RETURN => {
                if cursor.get_column_in_editor(editor_left_edge) < editor_width {
                    // If adding a new character on the current row will not move past the editor's right edge

                    // Add the character
                    gap_buf.insert(c as char);

                    // Move the cursor to the right
                    cursor.move_right();

                    // Set the current line's string content to the gap buffer
                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    // Reset the view
                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
                } else {
                    // If inserting a character will go beyond the editor's right edge (i.e. if the character should begin a new row)

                    // Insert the character into the gap buffer
                    gap_buf.insert(c as char);

                    // Set the current line's string content to the gap buffer
                    document.set_line_at_cursor(cursor.row, gap_buf.to_string(), editor_width);

                    // Move the cursor to the new row
                    cursor.move_down();

                    // Move the cursor to the left edge of the editor
                    cursor.move_to_editor_left(editor_left_edge);

                    // Move the cursor to the right to provide space for the character that was inserted
                    cursor.move_right();

                    // Reset the view
                    reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
                }
            }
            // Insert a character while in insert mode
            c if mode == Modes::Insert && c as char == '\n' => {
                // Collect the two sides of the gap buffer
                let (lhs, rhs) = gap_buf.collect_to_pieces();

                // Set the current line to the left hand side of the gap buffer
                document.set_line_at_cursor(cursor.row, lhs, editor_right_edge);

                // Move to the start of the new line to be created from the right hand side of the gap buffer
                cursor.move_to_end_line(&document, editor_left_edge, editor_width, editor_top);
                cursor.move_down();
                cursor.move_to_editor_left(editor_left_edge);

                // This ind_counter variable is created in such a way as to conform with the Line struct's from_str method requiring a mutable reference to a usize variable
                // this will be addressed later
                #[allow(unused_mut)]
                let mut ind_counter = cursor.row - 2;

                let new_line = Line::from_str(rhs, &mut ind_counter, editor_width);

                document.add_line_at_row(new_line, cursor.row);

                gap_buf = GapBuf::from_line(
                    document.get_line_at_cursor(cursor.row),
                    cursor.get_position_in_line(&document, editor_left_edge, editor_width),
                );

                reset_editor_view(&document, editor_left_edge, editor_right_edge, &mut cursor);
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
            RETURN if mode == Modes::Command => {
                let mut input = buf
                    .as_str()
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .into_iter();

                if let Some(command) = input.next() {
                    match command {
                        "w" => {
                            if let Some(file_name) = input.next() {
                                match fs::rename(&document.file_name, file_name) {
                                    _ => (),
                                }

                                let mut out_file = File::create(file_name).unwrap();

                                out_file.write(document.to_string().as_bytes()).unwrap();

                                document.file_name = file_name.to_string();

                                cursor.move_to(0, 0);

                                let curr_row = cursor.row;
                                let curr_col = cursor.column;

                                print!("{: >1$}", "", dimensions.width);

                                cursor.move_to(0, 0);

                                print!("{}", document.file_name);

                                cursor.move_to(curr_row, curr_col);
                            } else {
                                let mut out_file = File::create(&document.file_name).unwrap();

                                out_file.write(document.to_string().as_bytes()).unwrap();
                            }

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
                            if let Some(file_name) = input.next() {
                                match fs::rename(&document.file_name, file_name) {
                                    _ => (),
                                }

                                let mut out_file = File::create(file_name).unwrap();

                                out_file.write(document.to_string().as_bytes()).unwrap();

                                document.file_name = file_name.to_string();

                                cursor.move_to(0, 0);

                                let curr_row = cursor.row;
                                let curr_col = cursor.column;

                                print!("{: >1$}", "", dimensions.width);

                                cursor.move_to(0, 0);

                                print!("{}", document.file_name);

                                cursor.move_to(curr_row, curr_col);
                            } else {
                                let mut out_file = File::create(&document.file_name).unwrap();

                                out_file.write(document.to_string().as_bytes()).unwrap();
                            }

                            break;
                        }
                        _ => {
                            move_cursor_to(0, command_row);
                            print!("{: <1$}", "invalid command", dimensions.width);

                            change_mode(&mut mode, Modes::Normal, mode_row, &mut cursor);

                            cursor.revert_pos();

                            buf.clear();
                        }
                    }
                }
            }
            // Delete character while in command mode
            BCKSP if mode == Modes::Command => {
                if buf.len() > 0 {
                    // If the buffer is not empty

                    // Remove the last character of the command buffer
                    buf.pop();

                    // Move to the bottom row of the terminal and just after the colon
                    cursor.move_to(dimensions.height, 2);

                    // Visually blank out the bottom row
                    print!("{: >1$}", "", dimensions.width - 1);

                    // Move the cursor to just after the colon
                    cursor.move_to(dimensions.height, 2);

                    // Reprint the buffer
                    print!("{buf}");

                    // Move cursor to just after the original buffer minus the last character
                    cursor.move_to(dimensions.height, editor_left_edge + buf.len());
                }
            }
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

    return_to_normal_buf();
}
