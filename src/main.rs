use std::env;
use std::fs::{self, File};
use std::io::Write;
#[allow(unused_imports)]
use textchen::debug::*;
use textchen::{cursor::*, document::*, editor::*, gapbuf::*, term::*};

// ==== ASCII KEY CODE VALUES ====
const J_LOWER: u8 = 106;
const K_LOWER: u8 = 107;
const L_LOWER: u8 = 108;
const X_LOWER: u8 = 120;
const O_LOWER: u8 = 111;
const H_LOWER: u8 = 104;
const G_LOWER: u8 = 103;
const I_LOWER: u8 = 105;
const COLON: u8 = 58;
const ESC: u8 = 27;
const BCKSP: u8 = 127;
const RETURN: u8 = 10;

fn main() {
    // Used to log debug info to
    #[allow(unused_variables, unused_mut)]
    let mut log_file = File::create("log.txt").unwrap();

    // Dimensions for the terminal screen
    // Wh.width - The width of the terminal as a whole
    // Wh.height - The height of the terminal as a whole
    let mut dimensions = term_size();

    let mut editor_dim = Editor::new(term_size(), 2, dimensions.width - 2);

    // Title row is the home row
    // row: 0, column: 0

    // Get the command line arguments to the program

    let mut args = env::args();

    // Skip the first argument, this is unnecessary to the program

    let _ = args.next();

    // Prep the screen to draw the editor and the document to the screen, switching to alt buffer to not erase entire screen

    switch_to_alt_buf();
    clear_screen();

    // This cursor will be the cursor used throughout the document to draw and access elements from the
    // document

    let mut cursor = Cursor::new(editor_dim.editor_home_row, editor_dim.editor_left_edge);

    // This variable is like a more structured buffer for the whole document
    let mut document = create_document(args.next(), &editor_dim);

    move_cursor_home();
    print!("{}", &document.file_name);
    display_document(&document, &editor_dim, &mut cursor);

    // Print the mode to the screen, in this case, the default is normal
    cursor.move_to(editor_dim.mode_row, 0);
    print!("NOR");

    // Move the cursor to the editor home
    cursor.move_to(editor_dim.editor_home_row, editor_dim.editor_left_edge);

    // Initialize the gap buffer, it will be replaced later when editing actual text
    let mut gap_buf = GapBuf::new();

    // This is the buffer which will hold user commands

    let mut buf = String::new();

    // Stores the state of the mode for the program, starts with Modes::Normal
    let mut mode = Modes::Normal;

    // Set the terminal to raw input mode
    set_raw();

    // This will be the channel to receive the characters entered by the user
    let char_channel = spawn_char_channel();

    // Main loop for program
    loop {
        if dimensions.check_term_resize() {
            redraw_screen(
                &dimensions,
                &mut mode,
                &mut document,
                &mut editor_dim,
                &mut cursor,
            );
        }

        match char_channel.try_recv() {
            Ok(c) => {
                // Get a character and match it aginst some cases as a u8
                match c as u8 {
                    // Move down
                    J_LOWER if mode == Modes::Normal => {
                        // Store the position of the cursor in the original line, save on method calls

                        let cursor_pos = cursor.get_position_in_line(&document, &editor_dim);

                        if cursor.row < editor_dim.editor_height
                            && cursor.doc_row != *document.lines.last().unwrap().0.last().unwrap()
                        {
                            // If the cursor's visual row is less than the height of the editor (the editor's height refers to the number of rows *downward* that the
                            // editor's screen spans) and the cursor's row in relation to the document is not equal to the last row

                            cursor.move_down();
                        } else if cursor.doc_row
                            != *document.lines.last().unwrap().0.last().unwrap()
                        {
                            document.push_vis_down();
                            cursor.move_doc_down();

                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        }

                        if cursor_pos % editor_dim.editor_width
                            > document.get_line_at_cursor(cursor.doc_row).1.len() + 1
                        {
                            // If simply moving the cursor down to the next row will be outside of the bounds of that row's content and the line is one row long

                            // Move to the end of that new row
                            cursor.move_to_end_line(&mut document, &editor_dim);
                        }

                        same_line_different_row_bump(
                            cursor_pos,
                            &editor_dim,
                            document.get_line_at_cursor(cursor.doc_row - 1),
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &mut cursor,
                        );
                    }
                    // Move right
                    L_LOWER if mode == Modes::Normal => {
                        // Get the current line where the cursor is at

                        let curr_line = document.get_line_at_cursor(cursor.doc_row);
                        let cursor_pos = cursor.get_position_in_line(&document, &editor_dim);

                        if cursor_pos < curr_line.1.len()
                            && cursor.doc_column < editor_dim.editor_width
                        {
                            // If the cursor's position in the current line is less than the length of the total line and the cursor's column in relation to the document
                            // is less than or equal to the editor's width

                            cursor.move_right();
                        } else if cursor_pos < curr_line.1.len()
                            && curr_line.0.contains(&(cursor.doc_row + 1))
                        {
                            // If the cursor's position in the current line is less than the length of the total line and the current line's row indices contains the next
                            // cursor's row in relation to the document

                            if cursor.row < editor_dim.editor_height {
                                // If the cursor's row is less than the editor's height

                                // Move down to the next row
                                cursor.move_vis_down();
                            } else {
                                // If the cursor's row is at the editor's height

                                // Push the visible rows of the document down
                                document.push_vis_down();

                                // Reset the editor
                                reset_editor_view(&document, &editor_dim, &mut cursor);
                            }

                            // Move to the cursor visually the left edge of the editor
                            cursor.move_to_editor_left(editor_dim.editor_left_edge);
                            // Make the cursor's doc_column value 0 and then move it to the right (increment it) because the cursor needs to hover over the second character of the row
                            // in this particular case
                            cursor.move_doc_to_editor_left();

                            // Because the end of the previous line is included within the conditions of the previous if clause, move the cursor to the right of the immediate next
                            // chracter in the line
                            cursor.move_right();

                            // Set the place of the cursor within the document properly
                            cursor.move_doc_down();
                        }
                    }
                    // Move up
                    K_LOWER if mode == Modes::Normal => {
                        let cursor_pos = cursor.get_position_in_line(&document, &editor_dim);

                        if document.visible_rows.0 != 0 {
                            // If the document's visible rows does not include the first row

                            if cursor.row - 1 > editor_dim.editor_home_row {
                                // If moving the cursor visually updwards will not be the home row of the editor

                                cursor.move_up();

                                if cursor_pos
                                    > document.get_line_at_cursor(cursor.doc_row).1.len() + 1
                                {
                                    // If moving up would be outside of the bounds of the previos line

                                    cursor.move_to_end_line(&mut document, &editor_dim);
                                }
                            } else {
                                // If the cursor is visually below the editor's home row

                                cursor.move_doc_up();
                                document.push_vis_up();

                                reset_editor_view(&document, &editor_dim, &mut cursor);

                                if cursor_pos
                                    > document.get_line_at_cursor(cursor.doc_row).1.len() + 1
                                {
                                    // If moving up would be outside of the bounds of the previos line

                                    cursor.move_to_end_line(&mut document, &editor_dim);
                                }
                            }
                        } else if cursor.row != editor_dim.editor_home_row {
                            // If the cursor is not visually on the editor's home row

                            // Get the current position of the cursor
                            let cursor_pos = cursor.get_position_in_line(&document, &editor_dim);

                            cursor.move_up();

                            if document.get_line_at_cursor(cursor.doc_row).0.len() == 1
                                && cursor_pos > document.get_line_at_cursor(cursor.doc_row).1.len()
                            {
                                // If the new row is only one row long and the cursor's position is outside the bounds of the row
                                cursor.move_to_end_line(&mut document, &editor_dim);
                            }
                        }
                        same_line_different_row_bump(
                            cursor_pos,
                            &editor_dim,
                            document.get_line_at_cursor(cursor.doc_row - 1),
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &mut cursor,
                        );
                    }
                    // Move left
                    H_LOWER if mode == Modes::Normal => {
                        let cursor_pos = cursor.get_position_in_line(&document, &editor_dim);

                        if cursor.get_column_in_editor(editor_dim.editor_left_edge) > 1
                            || cursor_pos == 1
                        {
                            // If moving the cursor left does not reach the first column of the editor's field (i.e. the cursor will not be moved to the first possible column where characters can be printed to)
                            // or the cursor is at the second position of the line

                            cursor.move_left();
                        } else if cursor_pos / editor_dim.editor_width != 0 && cursor_pos != 0 {
                            // If the row in the line where the cursor is is not the first row of the line and the cursor is not at the first position of the line

                            if document.visible_rows.0 == 0
                                || cursor.row > editor_dim.editor_home_row
                            {
                                // If the document's visible rows does include the first row

                                cursor.move_vis_up();
                            } else {
                                // If the document's visible rows does not include the first row

                                document.push_vis_up();

                                reset_editor_view(&document, &editor_dim, &mut cursor);
                            }

                            cursor.move_doc_to_editor_width(editor_dim.editor_width);
                            cursor.move_to_editor_right(editor_dim.editor_right_edge);

                            cursor.move_doc_up();
                        }
                    }
                    G_LOWER if mode == Modes::Normal => {
                        change_mode(&mut mode, Modes::MoveTo, editor_dim.mode_row, &mut cursor);

                        let new_c = get_char();

                        if new_c == 'l' {
                            cursor.move_to_end_line(&mut document, &editor_dim);

                            change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);
                        } else if new_c == 'h' {
                            cursor.move_to_start_line(&mut document, &editor_dim);

                            change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);
                        } else if new_c == 'g' {
                            cursor.move_to(editor_dim.editor_home_row, editor_dim.editor_left_edge);
                            cursor.move_doc_to(0, 0);

                            document.visible_rows.0 = 0;
                            document.visible_rows.1 = editor_dim.editor_height;

                            reset_editor_view(&document, &editor_dim, &mut cursor);

                            change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);
                        } else if new_c == 'e' {
                            cursor.move_to(editor_dim.editor_height, editor_dim.editor_left_edge);
                            cursor
                                .move_doc_to(*document.lines.last().unwrap().0.last().unwrap(), 0);

                            document.visible_rows.0 =
                                (document.num_rows() + 1) - editor_dim.editor_height;
                            document.visible_rows.1 = document.num_rows();

                            reset_editor_view(&document, &editor_dim, &mut cursor);

                            change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);
                        } else {
                            change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);
                        }
                    }
                    X_LOWER if mode == Modes::Normal => {
                        // todo!("Reimplement for scrolling");
                        if get_char() == 'd' {
                            cursor.move_to_start_line(&mut document, &editor_dim);

                            // The key combination xd will delete a line
                            // Remove the line from the document
                            // document.remove_index_from_line(cursor.row);
                            document.remove_line_from_doc(cursor.doc_row, editor_dim.editor_width);

                            if document.num_rows() > 0 {
                                if cursor.doc_row > 0 {
                                    cursor.move_doc_up();

                                    if cursor.row == editor_dim.editor_home_row {
                                        // Move the cursor to the previous row
                                        cursor.move_to_start_line(&mut document, &editor_dim);
                                    } else {
                                        cursor.move_vis_up();
                                        cursor.move_to_start_line(&mut document, &editor_dim);
                                    }
                                }

                                if document.visible_rows.0 != 0
                                    && cursor.row == editor_dim.editor_home_row
                                {
                                    let curr_line_inds =
                                        document.get_line_at_cursor(cursor.doc_row).0.clone();

                                    while curr_line_inds[0] != document.visible_rows.0 {
                                        document.push_vis_up();
                                    }
                                }
                            }

                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        }
                    }
                    // Enter insert mode
                    I_LOWER if mode == Modes::Normal => {
                        // Change mode to insert
                        change_mode(&mut mode, Modes::Insert, editor_dim.mode_row, &mut cursor);

                        if document.lines.len() > 0 {
                            // Create a new gap buffer from the string at the current cursor position
                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.doc_row).to_owned(),
                                cursor.get_position_in_line(&document, &editor_dim),
                            );
                        } else {
                            gap_buf = GapBuf::new();

                            document.add_scratch_line();
                        }
                    }
                    // Create a new empty line
                    O_LOWER if mode == Modes::Normal => {
                        let mut new_line = Line::new();

                        // Change mode to insert
                        change_mode(&mut mode, Modes::Insert, editor_dim.mode_row, &mut cursor);

                        // Add the last index of the current line incremented to the new line's index list
                        new_line.0.push(cursor.doc_row + 1);

                        // Move to the beginning of the next possible line
                        cursor.move_to_end_line(&mut document, &editor_dim);

                        if cursor.row < editor_dim.editor_height {
                            // If the cursor's row is less than the editor's height

                            // Move down to the next row
                            cursor.move_vis_down();
                        } else {
                            // If the cursor's row is at the editor's height

                            // Push the visible rows of the document down
                            document.push_vis_down();
                        }

                        cursor.move_to_editor_left(editor_dim.editor_left_edge);
                        cursor.move_doc_to_editor_left();
                        cursor.move_doc_down();

                        // Add the new line to the document
                        document.add_line_at_row(new_line, cursor.doc_row);

                        // Crate an empty gap buffer since the line will be empty guaranteed
                        gap_buf = GapBuf::new();

                        // Reset view
                        reset_editor_view(&document, &editor_dim, &mut cursor);
                    }
                    // Exit insert mode
                    ESC if mode == Modes::Insert => {
                        // Change mode to normal
                        change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);

                        // Set the the to the string representation of the current gap buffer, reculculating the row indices for the line
                        document.set_line_at_cursor(
                            cursor.doc_row,
                            gap_buf.to_string(),
                            editor_dim.editor_width,
                        );
                    }
                    // Cancel entering a command
                    ESC if mode == Modes::Command => {
                        // Change mode to normal
                        change_mode(&mut mode, Modes::Normal, editor_dim.mode_row, &mut cursor);

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
                        let cursor_pos = cursor.get_position_in_line(&document, &editor_dim);

                        if cursor.doc_column > 1 || cursor_pos == 1 {
                            // If the cursor is one space away from being on top of the first column of characters (i.e. the cursor is within the line)

                            let num_leading_spaces = (document
                                .get_line_at_cursor(cursor.doc_row)
                                .1
                                .chars()
                                .take_while(|c| *c == ' ')
                                .count()
                                / 4)
                                * 4;

                            if num_leading_spaces
                                == cursor.get_position_in_line(&document, &editor_dim)
                                && num_leading_spaces % 4 == 0
                            {
                                // If the number of leading spaces is equivalent to the cursor's current position and
                                // the number of leading spaces is divisible by 4

                                gap_buf.pop_tab();

                                for _ in 0..4 {
                                    cursor.move_left();
                                }
                            } else {
                                // Remove the next character in the gap buffer
                                gap_buf.pop();

                                cursor.move_left();
                            }

                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor_dim.editor_width,
                            );

                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        } else if cursor_pos / editor_dim.editor_width != 0 {
                            // If the cursor is not in the first row of the line

                            // Remove the next character in the gap buffer
                            gap_buf.pop();

                            if document.visible_rows.0 == 0
                                || cursor.row > editor_dim.editor_home_row
                            {
                                // If the document's visible rows does include the first row

                                // Move the cursor to the previous row
                                cursor.move_vis_up();
                            } else {
                                // If the document's visible rows does not include the first row

                                document.push_vis_up();
                            }

                            // Move the cursor to the end of the previous row
                            cursor.move_to_editor_right(editor_dim.editor_right_edge);

                            cursor.move_doc_up();
                            cursor.move_doc_to_editor_width(editor_dim.editor_width);

                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor_dim.editor_width,
                            );

                            // Reset the view
                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        } else if cursor_pos == 0 && cursor.row != editor_dim.editor_home_row {
                            // If the cursor is at the first positon of the line and it is not in the first line of the document (note: cursor's doc row field is not used during checking because editor_top starts at the same index that cursor's row starts at)

                            // Get the current line's string
                            let curr_str = document.get_str_at_cursor(cursor.doc_row).to_owned();

                            // Remove the current line from the document
                            document.remove_line_from_doc(cursor.doc_row, editor_dim.editor_width);

                            // Move to the previous line
                            cursor.move_up();

                            // Move to the end of the previous line
                            cursor.move_to_end_line(&mut document, &editor_dim);

                            document.append_to_line(
                                cursor.doc_row,
                                &curr_str,
                                editor_dim.editor_width,
                            );

                            // Create a new gap buffer based on the new string at the cursor position
                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.doc_row).to_owned(),
                                cursor.get_position_in_line(&document, &editor_dim),
                            );

                            // Reset the view
                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        } else if cursor_pos == 0 && document.visible_rows.0 != 0 {
                            // If the cursor is at the first positon of the line and the first visible row is not the first row of the document

                            // Get the current line's string
                            let curr_str = document.get_str_at_cursor(cursor.doc_row).to_owned();

                            // Remove the current line from the document
                            document.remove_line_from_doc(cursor.doc_row, editor_dim.editor_width);

                            document.push_vis_up();

                            // Move to the previous line
                            cursor.move_doc_up();

                            // Move to the end of the previous line
                            cursor.move_to_end_line(&mut document, &editor_dim);

                            document.append_to_line(
                                cursor.doc_row,
                                &curr_str,
                                editor_dim.editor_width,
                            );

                            // Create a new gap buffer based on the new string at the cursor position
                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.doc_row).to_owned(),
                                cursor.get_position_in_line(&document, &editor_dim),
                            );

                            // Reset the view
                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        }
                    }
                    // Insert a new line character to break line while in insert mode
                    c if mode == Modes::Insert
                        && (c as char == ' ' || !(c as char).is_whitespace()) =>
                    {
                        // Here, c can only be a non whitespace character except for space
                        if cursor.doc_column < editor_dim.editor_width {
                            // If adding a new character on the current row will not move past the editor's right edge

                            // Add the character
                            gap_buf.insert(c as char);

                            // Move the cursor to the right
                            cursor.move_vis_right();
                            cursor.move_doc_right();

                            // Set the current line's string content to the gap buffer
                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor_dim.editor_width,
                            );

                            // Reset the view
                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        } else {
                            // If inserting a character will go beyond the editor's right edge (i.e. if the character should begin a new row)

                            // Insert the character into the gap buffer
                            gap_buf.insert(c as char);

                            // Set the current line's string content to the gap buffer
                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor_dim.editor_width,
                            );

                            if cursor.row < editor_dim.editor_height {
                                // If the cursor's row is less than the editor's height

                                // Move down to the next row
                                cursor.move_vis_down();
                            } else {
                                document.push_vis_down();
                            }

                            // Move the cursor to the left edge of the editor
                            cursor.move_to_editor_left(editor_dim.editor_left_edge);

                            // Move the cursor to the right to provide space for the character that was inserted
                            cursor.move_vis_right();

                            cursor.move_doc_down();
                            cursor.move_doc_to_editor_left();
                            cursor.move_doc_right();

                            // Reset the view
                            reset_editor_view(&document, &editor_dim, &mut cursor);
                        }
                    }
                    // Insert a character while in insert mode
                    c if mode == Modes::Insert && c == RETURN => {
                        debug_log_cursor(&cursor, &mut log_file);
                        // Collect the two sides of the gap buffer
                        let (lhs, mut rhs) = gap_buf.collect_to_pieces();

                        let num_spaces = (document
                            .get_line_at_cursor(cursor.doc_row)
                            .1
                            .chars()
                            .take_while(|c| *c == ' ')
                            .count()
                            / 4)
                            * 4;

                        rhs = (0..num_spaces).into_iter().map(|_| ' ').collect::<String>() + &rhs;

                        // Set the current line to the left hand side of the gap buffer
                        document.set_line_at_cursor(
                            cursor.doc_row,
                            lhs,
                            editor_dim.editor_right_edge,
                        );

                        // Move to the start of the new line to be created from the right hand side of the gap buffer
                        cursor.move_to_end_line(&mut document, &editor_dim);

                        if cursor.row < editor_dim.editor_height {
                            // If the cursor's row is less than the editor's height

                            // Move down to the next row
                            cursor.move_vis_down();
                        } else {
                            // If the cursor's row is at the editor's height

                            // Push the visible rows of the document down
                            document.push_vis_down();
                        }

                        cursor.move_doc_down();
                        cursor.move_to_editor_left(editor_dim.editor_left_edge);
                        cursor.move_doc_to_editor_left();

                        // This ind_counter variable is created in such a way as to conform with the Line struct's from_str method requiring a mutable reference to a usize variable
                        // this will be addressed later
                        #[allow(unused_mut)]
                        let mut ind_counter = cursor.doc_row;

                        let new_line =
                            Line::from_str(rhs, &mut ind_counter, editor_dim.editor_width);

                        document.add_line_at_row(new_line, cursor.doc_row);

                        gap_buf = GapBuf::from_line(
                            document.get_line_at_cursor(cursor.doc_row),
                            num_spaces,
                        );

                        cursor.move_to_pos(
                            num_spaces,
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &editor_dim,
                        );

                        reset_editor_view(&document, &editor_dim, &mut cursor);

                        debug_log_cursor(&cursor, &mut log_file);
                    }
                    c if mode == Modes::Insert && c as char == '\t' => {
                        // For now, a tab is represented as four spaces

                        for _ in 0..4 {
                            gap_buf.insert(' ');
                        }

                        let curr_pos = cursor.get_position_in_line(&document, &editor_dim);

                        document.set_line_at_cursor(
                            cursor.doc_row,
                            gap_buf.to_string(),
                            editor_dim.editor_width,
                        );

                        cursor.move_to_pos(
                            curr_pos + 4,
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &editor_dim,
                        );

                        reset_editor_view(&document, &editor_dim, &mut cursor);
                    }
                    // Enter command mode
                    COLON if mode == Modes::Normal => {
                        // Change to command mode
                        change_mode(&mut mode, Modes::Command, editor_dim.mode_row, &mut cursor);

                        // Clear the buffer to ensure the new command will be empty
                        buf.clear();

                        // Save cursor position to come back to
                        cursor.save_current_pos();

                        // Move cursor to command row
                        cursor.move_to(editor_dim.command_row, 1);

                        // Clear the line if something was already printed there
                        print!("{: >1$}", "", dimensions.width);

                        // Move cursor to command row
                        cursor.move_to(editor_dim.command_row, 1);

                        // Print a colon
                        print_flush(":");

                        // Move the cursor to align with the colon
                        cursor.move_vis_right();
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

                                        cursor.save_current_pos();

                                        print!("{: >1$}", "", dimensions.width);

                                        cursor.move_to(0, 0);

                                        print!("{}", document.file_name);

                                        cursor.revert_pos();
                                    } else {
                                        let mut out_file =
                                            File::create(&document.file_name).unwrap();

                                        out_file.write(document.to_string().as_bytes()).unwrap();
                                    }

                                    cursor.move_to(editor_dim.command_row, 0);
                                    print!("{: >1$}", "", dimensions.width);

                                    change_mode(
                                        &mut mode,
                                        Modes::Normal,
                                        editor_dim.mode_row,
                                        &mut cursor,
                                    );

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

                                        cursor.save_current_pos();

                                        print!("{: >1$}", "", dimensions.width);

                                        cursor.move_to(0, 0);

                                        print!("{}", document.file_name);

                                        cursor.revert_pos();
                                    } else {
                                        let mut out_file =
                                            File::create(&document.file_name).unwrap();

                                        out_file.write(document.to_string().as_bytes()).unwrap();
                                    }

                                    break;
                                }
                                _ => {
                                    move_cursor_to(editor_dim.command_row, 0);
                                    print!("{: <1$}", "invalid command", dimensions.width);

                                    change_mode(
                                        &mut mode,
                                        Modes::Normal,
                                        editor_dim.mode_row,
                                        &mut cursor,
                                    );

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
                            cursor.move_to(
                                dimensions.height,
                                editor_dim.editor_left_edge + buf.len(),
                            );
                        }
                    }
                    // Insert character while in command mode
                    c if mode == Modes::Command => {
                        // Push the pressed character to the buffer
                        buf.push(c as char);

                        // Display the character to the screen, stdout will be flush on cursor move
                        print!("{}", c as char);

                        cursor.move_vis_right();
                    }

                    _ => (),
                }
            }
            _ => (),
        }
    }

    return_to_normal_buf();

    // Similar to set_raw, only used/needed on linux
    set_cooked();
}
