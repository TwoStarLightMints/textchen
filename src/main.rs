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
const O_UPPER: u8 = 'O' as u8;
const H_LOWER: u8 = 104;
const G_LOWER: u8 = 103;
const I_LOWER: u8 = 105;
const COLON: u8 = 58;
const ESC: u8 = 27;
const BCKSP: u8 = if cfg!(target_os = "linux") { 127 } else { 8 };
const RETURN: u8 = if cfg!(target_os = "linux") { 10 } else { 13 };

fn main() {
    // Used to log debug info to
    #[allow(unused_variables, unused_mut)]
    let mut log_file = File::create("log.txt").unwrap();

    // Dimensions for the terminal screen
    // Wh.width - The width of the terminal as a whole
    // Wh.height - The height of the terminal as a whole
    let mut editor = Editor::new(term_size(), 2, 2);

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

    let mut cursor = Cursor::new(editor.doc_disp_home_row, editor.doc_disp_left_edge);

    // This variable is like a more structured buffer for the whole document
    let mut document = create_document(args.next(), &editor);

    move_cursor_home();
    editor.initialize_display(&document, &mut cursor);

    // Move the cursor to the editor home
    cursor.move_to(editor.doc_disp_home_row, editor.doc_disp_left_edge);

    // Initialize the gap buffer, it will be replaced later when editing actual text
    let mut gap_buf = GapBuf::new();

    // This is the buffer which will hold user commands

    editor.change_mode(Modes::Normal, &mut cursor);

    // Set the terminal to raw input mode
    #[cfg(target_os = "linux")]
    set_raw();

    // This will be the channel to receive the characters entered by the user
    let char_channel = spawn_char_channel();

    // Main loop for program
    loop {
        if editor.term_dimensions.check_term_resize() {
            editor.redraw_screen(&mut document, &mut cursor);
        }

        match char_channel.try_recv() {
            Ok(c) => {
                // Get a character and match it aginst some cases as a u8
                match c as u8 {
                    // Move down
                    J_LOWER if editor.curr_mode == Modes::Normal => {
                        // Store the position of the cursor in the original line, save on method calls

                        let cursor_pos = cursor.get_position_in_line(&document, &editor);

                        if cursor.row < editor.doc_disp_height
                            && cursor.doc_row != *document.lines.last().unwrap().0.last().unwrap()
                        {
                            // If the cursor's visual row is less than the height of the editor (the editor's height refers to the number of rows *downward* that the
                            // editor's screen spans) and the cursor's row in relation to the document is not equal to the last row

                            cursor.move_down();

                            let curr_line = document.get_line_at_cursor(cursor.doc_row);

                            if cursor.doc_column > curr_line.1.len() % editor.doc_disp_width
                                && cursor.doc_row == *curr_line.0.last().unwrap()
                            {
                                cursor.move_to_end_line(&mut document, &editor);
                            }
                        } else if cursor.doc_row
                            != *document.lines.last().unwrap().0.last().unwrap()
                        {
                            document.push_vis_down();
                            cursor.move_doc_down();

                            editor.reset_editor_view(&document, &mut cursor);
                        }

                        if cursor_pos % editor.doc_disp_width
                            > document.get_line_at_cursor(cursor.doc_row).1.len() + 1
                        {
                            // If simply moving the cursor down to the next row will be outside of the bounds of that row's content and the line is one row long

                            // Move to the end of that new row
                            cursor.move_to_end_line(&mut document, &editor);
                        }

                        same_line_different_row_bump(
                            cursor_pos,
                            &editor,
                            document.get_line_at_cursor(cursor.doc_row - 1),
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &mut cursor,
                        );
                    }
                    // Move right
                    L_LOWER if editor.curr_mode == Modes::Normal => {
                        // Get the current line where the cursor is at

                        let curr_line = document.get_line_at_cursor(cursor.doc_row);
                        let cursor_pos = cursor.get_position_in_line(&document, &editor);

                        if cursor_pos < curr_line.1.len()
                            && cursor.doc_column < editor.doc_disp_width
                        {
                            // If the cursor's position in the current line is less than the length of the total line and the cursor's column in relation to the document
                            // is less than or equal to the editor's width

                            cursor.move_right();
                        } else if cursor_pos < curr_line.1.len()
                            && curr_line.0.contains(&(cursor.doc_row + 1))
                        {
                            // If the cursor's position in the current line is less than the length of the total line and the current line's row indices contains the next
                            // cursor's row in relation to the document

                            if cursor.row < editor.doc_disp_height {
                                // If the cursor's row is less than the editor's height

                                // Move down to the next row
                                cursor.move_vis_down();
                            } else {
                                // If the cursor's row is at the editor's height

                                // Push the visible rows of the document down
                                document.push_vis_down();

                                // Reset the editor
                                editor.reset_editor_view(&document, &mut cursor);
                            }

                            // Move to the cursor visually the left edge of the editor
                            cursor.move_to_editor_left(editor.doc_disp_left_edge);
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
                    K_LOWER if editor.curr_mode == Modes::Normal => {
                        let cursor_pos = cursor.get_position_in_line(&document, &editor);

                        if document.visible_rows.0 != 0 {
                            // If the document's visible rows does not include the first row

                            if cursor.row - 1 > editor.doc_disp_home_row {
                                // If moving the cursor visually updwards will not be the home row of the editor

                                cursor.move_up();

                                if cursor_pos
                                    > document.get_line_at_cursor(cursor.doc_row).1.len() + 1
                                {
                                    // If moving up would be outside of the bounds of the previos line

                                    cursor.move_to_end_line(&mut document, &editor);
                                }
                            } else {
                                // If the cursor is visually below the editor's home row

                                cursor.move_doc_up();
                                document.push_vis_up();

                                editor.reset_editor_view(&document, &mut cursor);

                                if cursor_pos
                                    > document.get_line_at_cursor(cursor.doc_row).1.len() + 1
                                {
                                    // If moving up would be outside of the bounds of the previos line

                                    cursor.move_to_end_line(&mut document, &editor);
                                }
                            }
                        } else if cursor.row != editor.doc_disp_home_row {
                            // If the cursor is not visually on the editor's home row

                            // Get the current position of the cursor
                            let cursor_pos = cursor.get_position_in_line(&document, &editor);

                            cursor.move_up();

                            if document.get_line_at_cursor(cursor.doc_row).0.len() == 1
                                && cursor_pos > document.get_line_at_cursor(cursor.doc_row).1.len()
                            {
                                // If the new row is only one row long and the cursor's position is outside the bounds of the row
                                cursor.move_to_end_line(&mut document, &editor);
                            }
                        }

                        same_line_different_row_bump(
                            cursor_pos,
                            &editor,
                            if cursor.doc_row > 0 {
                                document.get_line_at_cursor(cursor.doc_row - 1)
                            } else {
                                document.get_line_at_cursor(cursor.doc_row)
                            },
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &mut cursor,
                        );
                    }
                    // Move left
                    H_LOWER if editor.curr_mode == Modes::Normal => {
                        let cursor_pos = cursor.get_position_in_line(&document, &editor);

                        if cursor.get_column_in_editor(editor.doc_disp_left_edge) > 1
                            || cursor_pos == 1
                        {
                            // If moving the cursor left does not reach the first column of the editor's field (i.e. the cursor will not be moved to the first possible column where characters can be printed to)
                            // or the cursor is at the second position of the line

                            cursor.move_left();
                        } else if cursor_pos / editor.doc_disp_width != 0 && cursor_pos != 0 {
                            // If the row in the line where the cursor is is not the first row of the line and the cursor is not at the first position of the line

                            if document.visible_rows.0 == 0 || cursor.row > editor.doc_disp_home_row
                            {
                                // If the document's visible rows does include the first row

                                cursor.move_vis_up();
                            } else {
                                // If the document's visible rows does not include the first row

                                document.push_vis_up();

                                editor.reset_editor_view(&document, &mut cursor);
                            }

                            cursor.move_doc_to_editor_width(editor.doc_disp_width);
                            cursor.move_to_editor_right(editor.doc_disp_right_edge);

                            cursor.move_doc_up();
                        }
                    }
                    G_LOWER if editor.curr_mode == Modes::Normal => {
                        editor.change_mode(Modes::MoveTo, &mut cursor);

                        let new_c = get_char();

                        if new_c == 'l' {
                            cursor.move_to_end_line(&mut document, &editor);

                            editor.change_mode(Modes::Normal, &mut cursor);
                        } else if new_c == 'h' {
                            cursor.move_to_start_line(&mut document, &editor);

                            editor.change_mode(Modes::Normal, &mut cursor);
                        } else if new_c == 'g' {
                            cursor.move_to(editor.doc_disp_home_row, editor.doc_disp_left_edge);
                            cursor.move_doc_to(0, 0);

                            document.visible_rows.0 = 0;
                            document.visible_rows.1 = editor.doc_disp_height;

                            editor.reset_editor_view(&document, &mut cursor);

                            editor.change_mode(Modes::Normal, &mut cursor);
                        } else if new_c == 'e' {
                            cursor.move_to(editor.doc_disp_height, editor.doc_disp_left_edge);
                            cursor
                                .move_doc_to(*document.lines.last().unwrap().0.last().unwrap(), 0);

                            document.visible_rows.0 =
                                (document.num_rows() + 1) - editor.doc_disp_height;
                            document.visible_rows.1 = document.num_rows();

                            editor.reset_editor_view(&document, &mut cursor);

                            editor.change_mode(Modes::Normal, &mut cursor);
                        } else {
                            editor.change_mode(Modes::Normal, &mut cursor);
                        }
                    }
                    X_LOWER if editor.curr_mode == Modes::Normal => {
                        // todo!("Reimplement for scrolling");
                        if get_char() == 'd' {
                            cursor.move_to_start_line(&mut document, &editor);

                            // The key combination xd will delete a line
                            // Remove the line from the document
                            // document.remove_index_from_line(cursor.row);
                            document.remove_line_from_doc(cursor.doc_row, editor.doc_disp_width);

                            if document.num_rows() > 0 {
                                if cursor.doc_row > 0 {
                                    cursor.move_doc_up();

                                    if cursor.row == editor.doc_disp_home_row {
                                        // Move the cursor to the previous row
                                        cursor.move_to_start_line(&mut document, &editor);
                                    } else {
                                        cursor.move_vis_up();
                                        cursor.move_to_start_line(&mut document, &editor);
                                    }
                                }

                                if document.visible_rows.0 != 0
                                    && cursor.row == editor.doc_disp_home_row
                                {
                                    let curr_line_inds =
                                        document.get_line_at_cursor(cursor.doc_row).0.clone();

                                    while curr_line_inds[0] != document.visible_rows.0 {
                                        document.push_vis_up();
                                    }
                                }
                            }

                            editor.reset_editor_view(&document, &mut cursor);
                        }
                    }
                    // Enter insert mode
                    I_LOWER if editor.curr_mode == Modes::Normal => {
                        // Change mode to insert
                        editor.change_mode(Modes::Insert, &mut cursor);

                        if document.lines.len() > 0 {
                            // Create a new gap buffer from the string at the current cursor position
                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.doc_row).to_owned(),
                                cursor.get_position_in_line(&document, &editor),
                            );
                        } else {
                            gap_buf = GapBuf::new();

                            document.add_scratch_line();
                        }
                    }
                    // Create a new empty line below current position of the cursor
                    O_LOWER if editor.curr_mode == Modes::Normal => {
                        let mut new_line = Line::new();

                        // Change mode to insert
                        editor.change_mode(Modes::Insert, &mut cursor);

                        // Add the last index of the current line incremented to the new line's index list
                        new_line.0.push(cursor.doc_row + 1);

                        // Move to the beginning of the next possible line
                        cursor.move_to_end_line(&mut document, &editor);

                        if cursor.row < editor.doc_disp_height {
                            // If the cursor's row is less than the editor's height

                            // Move down to the next row
                            cursor.move_vis_down();
                        } else {
                            // If the cursor's row is at the editor's height

                            // Push the visible rows of the document down
                            document.push_vis_down();
                        }

                        cursor.move_to_editor_left(editor.doc_disp_left_edge);
                        cursor.move_doc_to_editor_left();
                        cursor.move_doc_down();

                        // Add the new line to the document
                        document.add_line_at_row(new_line, cursor.doc_row);

                        // Crate an empty gap buffer since the line will be empty guaranteed
                        gap_buf = GapBuf::new();

                        // Reset view
                        editor.reset_editor_view(&document, &mut cursor);
                    }
                    // Create new empty line at the current cursor position, push all other contents down
                    O_UPPER if editor.curr_mode == Modes::Normal => {
                        let mut new_line = Line::new();

                        // Change mode to insert
                        editor.change_mode(Modes::Insert, &mut cursor);

                        // The new line will be inserted at the current position and will not change
                        // the position of the cursor visually or within the document
                        new_line.0.push(cursor.doc_row);

                        // Move to the beginning of the current line
                        cursor.move_to_start_line(&mut document, &editor);

                        // Move the cursor visually and within the document to the leftmost position
                        cursor.move_to_editor_left(editor.doc_disp_left_edge);
                        cursor.move_doc_to_editor_left();

                        // Add the new line to the document at the cursor's current row
                        document.add_line_at_row(new_line, cursor.doc_row);

                        // Crate an empty gap buffer since the line will be empty guaranteed
                        gap_buf = GapBuf::new();

                        // Reset view
                        editor.reset_editor_view(&document, &mut cursor);
                    }
                    // Exit insert mode
                    ESC if editor.curr_mode == Modes::Insert => {
                        // Change mode to normal
                        editor.change_mode(Modes::Normal, &mut cursor);

                        // Set the the to the string representation of the current gap buffer, reculculating the row indices for the line
                        document.set_line_at_cursor(
                            cursor.doc_row,
                            gap_buf.to_string(),
                            editor.doc_disp_width,
                        );
                    }
                    // Cancel entering a command
                    ESC if editor.curr_mode == Modes::Command => {
                        // Change mode to normal
                        editor.change_mode(Modes::Normal, &mut cursor);

                        // Clear the buffer
                        editor.command_buf.clear();
                    }
                    // Delete a character while in insert mode
                    BCKSP if editor.curr_mode == Modes::Insert => {
                        let cursor_pos = cursor.get_position_in_line(&document, &editor);

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

                            if num_leading_spaces == cursor.get_position_in_line(&document, &editor)
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
                                editor.doc_disp_width,
                            );

                            editor.reset_editor_view(&document, &mut cursor);
                        } else if cursor_pos / editor.doc_disp_width != 0 {
                            // If the cursor is not in the first row of the line

                            // Remove the next character in the gap buffer
                            gap_buf.pop();

                            if document.visible_rows.0 == 0 || cursor.row > editor.doc_disp_home_row
                            {
                                // If the document's visible rows does include the first row

                                // Move the cursor to the previous row
                                cursor.move_vis_up();
                            } else {
                                // If the document's visible rows does not include the first row

                                document.push_vis_up();
                            }

                            // Move the cursor to the end of the previous row
                            cursor.move_to_editor_right(editor.doc_disp_right_edge);

                            cursor.move_doc_up();
                            cursor.move_doc_to_editor_width(editor.doc_disp_width);

                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor.doc_disp_width,
                            );

                            // Reset the view
                            editor.reset_editor_view(&document, &mut cursor);
                        } else if cursor_pos == 0 && cursor.row != editor.doc_disp_home_row {
                            // If the cursor is at the first positon of the line and it is not in the first line of the document (note: cursor's doc row field is not used during checking because editor_top starts at the same index that cursor's row starts at)

                            // Get the current line's string
                            let curr_str = document.get_str_at_cursor(cursor.doc_row).to_owned();

                            // Remove the current line from the document
                            document.remove_line_from_doc(cursor.doc_row, editor.doc_disp_width);

                            // Move to the previous line
                            cursor.move_up();

                            // Move to the end of the previous line
                            cursor.move_to_end_line(&mut document, &editor);

                            document.append_to_line(
                                cursor.doc_row,
                                &curr_str,
                                editor.doc_disp_width,
                            );

                            // Create a new gap buffer based on the new string at the cursor position
                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.doc_row).to_owned(),
                                cursor.get_position_in_line(&document, &editor),
                            );

                            // Reset the view
                            editor.reset_editor_view(&document, &mut cursor);
                        } else if cursor_pos == 0 && document.visible_rows.0 != 0 {
                            // If the cursor is at the first positon of the line and the first visible row is not the first row of the document

                            // Get the current line's string
                            let curr_str = document.get_str_at_cursor(cursor.doc_row).to_owned();

                            // Remove the current line from the document
                            document.remove_line_from_doc(cursor.doc_row, editor.doc_disp_width);

                            document.push_vis_up();

                            // Move to the previous line
                            cursor.move_doc_up();

                            // Move to the end of the previous line
                            cursor.move_to_end_line(&mut document, &editor);

                            document.append_to_line(
                                cursor.doc_row,
                                &curr_str,
                                editor.doc_disp_width,
                            );

                            // Create a new gap buffer based on the new string at the cursor position
                            gap_buf = GapBuf::from_str(
                                document.get_str_at_cursor(cursor.doc_row).to_owned(),
                                cursor.get_position_in_line(&document, &editor),
                            );

                            // Reset the view
                            editor.reset_editor_view(&document, &mut cursor);
                        }
                    }
                    // Insert a new line character to break line while in insert mode
                    c if editor.curr_mode == Modes::Insert
                        && (c as char == ' ' || !(c as char).is_whitespace()) =>
                    {
                        // Here, c can only be a non whitespace character except for space
                        if cursor.doc_column < editor.doc_disp_width {
                            // If adding a new character on the current row will not move past the editor's right edge

                            // Add the character
                            gap_buf.insert(c as char);

                            // Move the cursor to the right
                            cursor.move_vis_right();
                            cursor.move_doc_right();

                            let curr_line_ind =
                                document.get_index_at_cursor(cursor.doc_row).unwrap();

                            let num_line_rows = document.lines[curr_line_ind]
                                .rows(editor.doc_disp_width)
                                .count();

                            // Set the current line's string content to the gap buffer
                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor.doc_disp_width,
                            );

                            // Reset the view
                            if num_line_rows
                                == document.lines[curr_line_ind]
                                    .rows(editor.doc_disp_width)
                                    .count()
                            {
                                editor.print_line(&mut document, &mut cursor);
                            } else {
                                editor.reset_editor_view(&document, &mut cursor);
                            }
                        } else {
                            // If inserting a character will go beyond the editor's right edge (i.e. if the character should begin a new row)

                            // Insert the character into the gap buffer
                            gap_buf.insert(c as char);

                            let curr_line_ind =
                                document.get_index_at_cursor(cursor.doc_row).unwrap();

                            let num_line_rows = document.lines[curr_line_ind]
                                .rows(editor.doc_disp_width)
                                .count();

                            // Set the current line's string content to the gap buffer
                            document.set_line_at_cursor(
                                cursor.doc_row,
                                gap_buf.to_string(),
                                editor.doc_disp_width,
                            );

                            if cursor.row < editor.doc_disp_height {
                                // If the cursor's row is less than the editor's height

                                // Move down to the next row
                                cursor.move_vis_down();
                            } else {
                                document.push_vis_down();
                            }

                            // Move the cursor to the left edge of the editor
                            cursor.move_to_editor_left(editor.doc_disp_left_edge);

                            // Move the cursor to the right to provide space for the character that was inserted
                            cursor.move_vis_right();

                            cursor.move_doc_down();
                            cursor.move_doc_to_editor_left();
                            cursor.move_doc_right();

                            // Reset the view
                            if num_line_rows
                                == document.lines[curr_line_ind]
                                    .rows(editor.doc_disp_width)
                                    .count()
                            {
                                editor.print_line(&mut document, &mut cursor);
                            } else {
                                editor.reset_editor_view(&document, &mut cursor);
                            }
                        }
                    }
                    // Insert a character while in insert mode
                    c if editor.curr_mode == Modes::Insert && c == RETURN => {
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
                            editor.doc_disp_right_edge,
                        );

                        // Move to the start of the new line to be created from the right hand side of the gap buffer
                        cursor.move_to_end_line(&mut document, &editor);

                        if cursor.row < editor.doc_disp_height {
                            // If the cursor's row is less than the editor's height

                            // Move down to the next row
                            cursor.move_vis_down();
                        } else {
                            // If the cursor's row is at the editor's height

                            // Push the visible rows of the document down
                            document.push_vis_down();
                        }

                        cursor.move_doc_down();
                        cursor.move_to_editor_left(editor.doc_disp_left_edge);
                        cursor.move_doc_to_editor_left();

                        // This ind_counter variable is created in such a way as to conform with the Line struct's from_str method requiring a mutable reference to a usize variable
                        // this will be addressed later
                        #[allow(unused_mut)]
                        let mut ind_counter = cursor.doc_row;

                        let new_line = Line::from_str(rhs, &mut ind_counter, editor.doc_disp_width);

                        document.add_line_at_row(new_line, cursor.doc_row);

                        gap_buf = GapBuf::from_line(
                            document.get_line_at_cursor(cursor.doc_row),
                            num_spaces,
                        );

                        cursor.move_to_pos(
                            num_spaces,
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &editor,
                        );

                        editor.reset_editor_view(&document, &mut cursor);
                    }
                    c if editor.curr_mode == Modes::Insert && c as char == '\t' => {
                        // For now, a tab is represented as four spaces

                        for _ in 0..4 {
                            gap_buf.insert(' ');
                        }

                        let curr_pos = cursor.get_position_in_line(&document, &editor);

                        document.set_line_at_cursor(
                            cursor.doc_row,
                            gap_buf.to_string(),
                            editor.doc_disp_width,
                        );

                        cursor.move_to_pos(
                            curr_pos + 4,
                            document.get_line_at_cursor(cursor.doc_row),
                            &document,
                            &editor,
                        );

                        editor.reset_editor_view(&document, &mut cursor);
                    }
                    // Enter command mode
                    COLON if editor.curr_mode == Modes::Normal => {
                        // Change to command mode
                        editor.change_mode(Modes::Command, &mut cursor);

                        // Clear the buffer to ensure the new command will be empty
                        editor.command_buf.clear();

                        // Save cursor position to come back to
                        cursor.save_current_pos();

                        // Move cursor to command row at first position
                        cursor.move_to(editor.command_row, 1);
                        editor.print_char(':');

                        cursor.move_vis_right();
                    }
                    // Execute command while in command mdoe
                    RETURN if editor.curr_mode == Modes::Command => {
                        let mut input = editor
                            .command_buf
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

                                        print!("{: >1$}", "", editor.term_dimensions.width);

                                        cursor.move_to(0, 0);

                                        print!("{}", document.file_name);

                                        cursor.revert_pos();
                                    } else {
                                        let mut out_file =
                                            File::create(&document.file_name).unwrap();

                                        out_file.write(document.to_string().as_bytes()).unwrap();
                                    }

                                    cursor.move_to(editor.command_row, 0);
                                    print!("{: >1$}", "", editor.term_dimensions.width);

                                    editor.change_mode(Modes::Normal, &mut cursor);

                                    cursor.revert_pos();

                                    editor.command_buf.clear();
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

                                        print!("{: >1$}", "", editor.term_dimensions.width);

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
                                    cursor.revert_pos();
                                    editor.print_command_message("Invalid Command", &mut cursor);

                                    editor.command_buf.clear();

                                    editor.change_mode(Modes::Normal, &mut cursor);
                                }
                            }
                        }
                    }
                    // Delete character while in command mode
                    BCKSP if editor.curr_mode == Modes::Command => {
                        if editor.command_buf.len() > 0 {
                            // If the buffer is not empty

                            cursor.move_to(
                                editor.command_row,
                                editor.doc_disp_left_edge + editor.command_buf.len() - 1,
                            );

                            editor.pop_command_buf();

                            // Move cursor to just after the original buffer minus the last character
                            cursor.move_to(
                                editor.command_row,
                                editor.doc_disp_left_edge + editor.command_buf.len(),
                            );
                        }
                    }
                    // Insert character while in command mode
                    c if editor.curr_mode == Modes::Command => {
                        // Push the pressed character to the buffer
                        // Display the character to the screen
                        editor.print_char(c as char);

                        cursor.move_vis_right();
                    }

                    _ => (),
                }
            }
            _ => (),
        }
    }

    return_to_normal_buf();

    #[cfg(target_os = "linux")]
    // Similar to set_raw, only used/needed on linux
    set_cooked();
}
