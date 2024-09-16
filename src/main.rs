use textchen::{document::*, editor::*, gapbuf::*, term::*};

// ==== ASCII KEY CODE VALUES ====
// Note: I use the ascii values as the keys so that it is more simple
// to check against special keys such as Escape, Backspace, etc.
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

const CTRL_H: u8 = 8;

fn main() {
    // Editor is the primary instance to control the editor and all its data
    let mut editor = Editor::new(2, 2);

    editor.initialize();

    // Initialize the gap buffer, it will be replaced later when editing actual text
    let mut gap_buf = GapBuf::new();

    // Main loop for program
    loop {
        editor.check_resize();

        if kbhit() {
            match get_char() as u8 {
                // Move down
                J_LOWER if editor.curr_mode == Modes::Normal => {
                    // Store the position of the cursor in the original line, save on method calls

                    if editor.get_cursor_vis_row() < editor.doc_disp_height()
                        && editor.get_cursor_doc_row() != editor.current_buffer_last_row_index()
                    {
                        // If the cursor's visual row is less than the height of the editor (the editor's height refers to the number of rows *downward* that the
                        // editor's screen spans) and the cursor's row in relation to the document is not equal to the last row

                        editor.move_cursor_down();

                        let doc_binding = editor.current_buffer();

                        let document = doc_binding.borrow();
                        let curr_line = document.get_line_at_cursor(editor.get_cursor_doc_row());

                        if editor.get_cursor_doc_col() > curr_line.1.len() % editor.doc_disp_width()
                            && editor.get_cursor_doc_row() == *curr_line.0.last().unwrap()
                        {
                            editor.move_cursor_to_end_line();
                        }
                    } else if editor.get_cursor_doc_row() != editor.current_buffer_last_row_index()
                    {
                        editor.current_buffer().borrow_mut().push_vis_down();
                        editor.move_cursor_doc_down();

                        editor.reset_editor_view();
                    }

                    if editor.get_cursor_pos_in_line() % editor.doc_disp_width()
                        > editor
                            .current_buffer()
                            .borrow()
                            .get_line_at_cursor(editor.get_cursor_doc_row())
                            .1
                            .len()
                            + 1
                    {
                        // If simply moving the cursor down to the next row will be outside of the bounds of that row's content and the line is one row long

                        // Move to the end of that new row
                        editor.move_cursor_to_end_line();
                    }

                    editor.multi_row_bump();
                }
                // Move right
                L_LOWER if editor.curr_mode == Modes::Normal => {
                    // Get the current line where the cursor is at

                    let doc_binding = editor.current_buffer();

                    let document = doc_binding.borrow();
                    let curr_line = document.get_line_at_cursor(editor.get_cursor_doc_row());
                    let cursor_pos = editor.get_cursor_pos_in_line();

                    if cursor_pos < curr_line.1.len()
                        && editor.get_cursor_doc_col() < editor.doc_disp_width()
                    {
                        // If the cursor's position in the current line is less than the length of the total line and the cursor's column in relation to the document
                        // is less than or equal to the editor's width

                        editor.move_cursor_right();
                    } else if cursor_pos < curr_line.1.len()
                        && curr_line.0.contains(&(editor.get_cursor_doc_row() + 1))
                    {
                        // If the cursor's position in the current line is less than the length of the total line and the current line's row indices contains the next
                        // cursor's row in relation to the document

                        if editor.get_cursor_vis_row() < editor.doc_disp_height() {
                            // If the cursor's row is less than the editor's height

                            // Move down to the next row
                            editor.move_cursor_vis_down();
                        } else {
                            // If the cursor's row is at the editor's height

                            // Push the visible rows of the document down
                            editor.current_buffer().borrow_mut().push_vis_down();

                            // Reset the editor
                            editor.reset_editor_view();
                        }

                        // Move to the cursor visually the left edge of the editor
                        editor.move_cursor_vis_editor_left();
                        // Make the cursor's doc_column value 0 and then move it to the right (increment it) because the cursor needs to hover over the second character of the row
                        // in this particular case
                        editor.move_cursor_doc_editor_left();

                        // Because the end of the previous line is included within the conditions of the previous if clause, move the cursor to the right of the immediate next
                        // chracter in the line
                        editor.move_cursor_right();

                        // Set the place of the cursor within the document properly
                        editor.move_cursor_doc_down();
                    }
                }
                // Move up
                K_LOWER if editor.curr_mode == Modes::Normal => {
                    let cursor_pos = editor.get_cursor_pos_in_line();

                    if editor.current_buffer().borrow().visible_rows.0 != 0 {
                        // If the document's visible rows does not include the first row

                        if editor.get_cursor_vis_row() - 1 > editor.doc_disp_home_row() {
                            // If moving the cursor visually updwards will not be the home row of the editor

                            editor.move_cursor_up();

                            if cursor_pos
                                > editor
                                    .current_buffer()
                                    .borrow()
                                    .get_line_at_cursor(editor.get_cursor_doc_row())
                                    .1
                                    .len()
                                    + 1
                            {
                                // If moving up would be outside of the bounds of the previos line

                                editor.move_cursor_to_end_line();
                            }
                        } else {
                            // If the cursor is visually below the editor's home row

                            editor.move_cursor_doc_up();
                            editor
                                .current_buffer()
                                .borrow_mut()
                                .push_vis_up(editor.doc_disp_height());

                            editor.reset_editor_view();

                            if cursor_pos
                                > editor
                                    .current_buffer()
                                    .borrow()
                                    .get_line_at_cursor(editor.get_cursor_doc_row())
                                    .1
                                    .len()
                                    + 1
                            {
                                // If moving up would be outside of the bounds of the previos line

                                editor.move_cursor_to_end_line();
                            }
                        }
                    } else if editor.get_cursor_vis_row() != editor.doc_disp_home_row() {
                        // If the cursor is not visually on the editor's home row

                        // Get the current position of the cursor
                        let cursor_pos = editor.get_cursor_pos_in_line();

                        editor.move_cursor_up();

                        if editor
                            .current_buffer()
                            .borrow()
                            .get_line_at_cursor(editor.get_cursor_doc_row())
                            .0
                            .len()
                            == 1
                            && cursor_pos
                                > editor
                                    .current_buffer()
                                    .borrow()
                                    .get_line_at_cursor(editor.get_cursor_doc_row())
                                    .1
                                    .len()
                        {
                            // If the new row is only one row long and the cursor's position is outside the bounds of the row
                            editor.move_cursor_to_end_line();
                        }
                    }

                    editor.multi_row_bump();
                }
                // Move left
                H_LOWER if editor.curr_mode == Modes::Normal => {
                    let cursor_pos = editor.get_cursor_pos_in_line();

                    if editor.get_cursor_column_in_doc_disp() > 1 || cursor_pos == 1 {
                        // If moving the cursor left does not reach the first column of the editor's field (i.e. the cursor will not be moved to the first possible column where characters can be printed to)
                        // or the cursor is at the second position of the line

                        editor.move_cursor_left();
                    } else if cursor_pos / editor.doc_disp_width() != 0 && cursor_pos != 0 {
                        // If the row in the line where the cursor is is not the first row of the line and the cursor is not at the first position of the line

                        if editor.current_buffer().borrow().visible_rows.0 == 0
                            || editor.get_cursor_vis_row() > editor.doc_disp_home_row()
                        {
                            // If the document's visible rows does include the first row

                            editor.move_cursor_vis_up();
                        } else {
                            // If the document's visible rows does not include the first row

                            editor
                                .current_buffer()
                                .borrow_mut()
                                .push_vis_up(editor.doc_disp_height());

                            editor.reset_editor_view();
                        }

                        editor.move_cursor_doc_to_editor_right();

                        editor.move_cursor_vis_to_editor_right();

                        editor.move_cursor_doc_up();
                    }
                }
                G_LOWER if editor.curr_mode == Modes::Normal => {
                    editor.change_mode(Modes::MoveTo);

                    // This flush is necessary because otherwise the new mode is not printed
                    editor.flush_pen();

                    let new_c = get_char();

                    if new_c == 'l' {
                        editor.move_cursor_to_end_line();

                        editor.change_mode(Modes::Normal);
                    } else if new_c == 'h' {
                        editor.move_cursor_to_start_line();

                        editor.change_mode(Modes::Normal);
                    } else if new_c == 'g' {
                        editor.move_cursor_vis_to(
                            editor.doc_disp_home_row(),
                            editor.doc_disp_left_edge(),
                        );
                        editor.move_cursor_doc_to(0, 0);

                        editor.current_buffer().borrow_mut().visible_rows.0 = 0;
                        editor.current_buffer().borrow_mut().visible_rows.1 =
                            editor.doc_disp_height();

                        editor.reset_editor_view();

                        editor.change_mode(Modes::Normal);
                    } else if new_c == 'e' {
                        editor.move_cursor_vis_to(
                            editor.doc_disp_height(),
                            editor.doc_disp_left_edge(),
                        );

                        editor.move_cursor_doc_to(editor.current_buffer_last_row_index(), 0);

                        let new_top_vis = (editor.current_buffer().borrow().num_rows() + 1)
                            - editor.doc_disp_height();
                        editor.current_buffer().borrow_mut().visible_rows.0 = new_top_vis;

                        let new_bottom_vis = editor.current_buffer().borrow().num_rows();
                        editor.current_buffer().borrow_mut().visible_rows.1 = new_bottom_vis;

                        editor.reset_editor_view();

                        editor.change_mode(Modes::Normal);
                    } else if new_c == 'n' {
                        editor.next_buffer();

                        editor.change_mode(Modes::Normal);
                    } else if new_c == 'p' {
                        editor.prev_buffer();

                        editor.change_mode(Modes::Normal);
                    } else {
                        editor.change_mode(Modes::Normal);
                    }
                }
                X_LOWER if editor.curr_mode == Modes::Normal => {
                    // todo!("Reimplement for scrolling");
                    if get_char() == 'd' {
                        editor.move_cursor_to_start_line();

                        // The key combination xd will delete a line
                        // Remove the line from the document
                        editor.current_buffer().borrow_mut().remove_line_from_doc(
                            editor.get_cursor_doc_row(),
                            editor.doc_disp_width(),
                        );

                        if editor.current_buffer().borrow().num_rows() > 0 {
                            if editor.get_cursor_doc_row() > 0 {
                                editor.move_cursor_doc_up();

                                if editor.get_cursor_vis_row() == editor.doc_disp_home_row() {
                                    // Move the cursor to the previous row
                                    editor.move_cursor_to_start_line();
                                } else {
                                    editor.move_cursor_vis_up();
                                    editor.move_cursor_to_start_line();
                                }
                            }

                            if editor.current_buffer().borrow().visible_rows.0 != 0
                                && editor.get_cursor_vis_row() == editor.doc_disp_home_row()
                            {
                                let curr_line_inds = editor
                                    .current_buffer()
                                    .borrow()
                                    .get_line_at_cursor(editor.get_cursor_doc_row())
                                    .0
                                    .clone();

                                while curr_line_inds[0]
                                    != editor.current_buffer().borrow().visible_rows.0
                                {
                                    editor
                                        .current_buffer()
                                        .borrow_mut()
                                        .push_vis_up(editor.doc_disp_height());
                                }
                            }
                        }

                        editor.reset_editor_view();
                    }
                }
                // Enter insert mode
                I_LOWER if editor.curr_mode == Modes::Normal => {
                    // Change mode to insert
                    editor.change_mode(Modes::Insert);

                    if editor.current_buffer().borrow().lines.len() > 0 {
                        // Create a new gap buffer from the string at the current cursor position
                        gap_buf = GapBuf::from_str(
                            editor
                                .current_buffer()
                                .borrow()
                                .get_str_at_cursor(editor.get_cursor_doc_row())
                                .to_owned(),
                            editor.get_cursor_pos_in_line(),
                        );
                    } else {
                        gap_buf = GapBuf::new();

                        editor.current_buffer().borrow_mut().add_scratch_line();
                    }
                }
                // Create a new empty line below current position of the cursor
                O_LOWER if editor.curr_mode == Modes::Normal => {
                    let mut new_line = Line::new();

                    // Change mode to insert
                    editor.change_mode(Modes::Insert);

                    // Add the last index of the current line incremented to the new line's index list
                    new_line.0.push(editor.get_cursor_doc_row() + 1);

                    // Move to the beginning of the next possible line
                    editor.move_cursor_to_end_line();

                    if editor.get_cursor_vis_row() < editor.doc_disp_height() {
                        // If the cursor's row is less than the editor's height

                        // Move down to the next row
                        editor.move_cursor_vis_down();
                    } else if editor.get_cursor_doc_row()
                        != *editor
                            .current_buffer()
                            .borrow()
                            .lines
                            .last()
                            .unwrap()
                            .0
                            .last()
                            .unwrap()
                    {
                        // If the cursor's row is at the editor's height

                        // Push the visible rows of the document down
                        editor.current_buffer().borrow_mut().push_vis_down();
                    }

                    editor.move_cursor_vis_editor_left();
                    editor.move_cursor_doc_down();

                    // Add the new line to the document
                    editor
                        .current_buffer()
                        .borrow_mut()
                        .add_line_at_row(new_line, editor.get_cursor_doc_row());

                    // Crate an empty gap buffer since the line will be empty guaranteed
                    gap_buf = GapBuf::new();

                    // Reset view
                    editor.reset_editor_view();
                }
                // Create new empty line at the current cursor position, push all other contents down
                O_UPPER if editor.curr_mode == Modes::Normal => {
                    let mut new_line = Line::new();

                    // Change mode to insert
                    editor.change_mode(Modes::Insert);

                    // The new line will be inserted at the current position and will not change
                    // the position of the cursor visually or within the document
                    new_line.0.push(editor.get_cursor_doc_row());

                    // Move to the beginning of the current line
                    editor.move_cursor_to_start_line();

                    // Move the cursor visually and within the document to the leftmost position
                    editor.move_cursor_vis_editor_left();
                    editor.move_cursor_doc_editor_left();

                    // Add the new line to the document at the cursor's current row
                    editor
                        .current_buffer()
                        .borrow_mut()
                        .add_line_at_row(new_line, editor.get_cursor_doc_row());

                    // Crate an empty gap buffer since the line will be empty guaranteed
                    gap_buf = GapBuf::new();

                    // Reset view
                    editor.reset_editor_view();
                }
                // Exit insert mode
                ESC if editor.curr_mode == Modes::Insert => {
                    // Change mode to normal
                    editor.change_mode(Modes::Normal);

                    // Set the the to the string representation of the current gap buffer, reculculating the row indices for the line
                    editor.current_buffer().borrow_mut().set_line_at_cursor(
                        editor.get_cursor_doc_row(),
                        gap_buf.to_string(),
                        editor.doc_disp_width(),
                    );
                }
                // Cancel entering a command
                ESC if editor.curr_mode == Modes::Command => {
                    editor.exit_command_mode::<String>(None);

                    // Change mode to normal
                    editor.change_mode(Modes::Normal);
                }
                // Delete a character while in insert mode
                BCKSP | CTRL_H if editor.curr_mode == Modes::Insert => {
                    let cursor_pos = editor.get_cursor_pos_in_line();

                    let curr_num_rows = editor.current_buffer().borrow().num_rows();

                    if editor.get_cursor_doc_col() > 1 || cursor_pos == 1 {
                        // If the cursor is one space away from being on top of the first column of characters (i.e. the cursor is within the line)

                        let num_leading_spaces = (editor
                            .current_buffer()
                            .borrow()
                            .get_line_at_cursor(editor.get_cursor_doc_row())
                            .1
                            .chars()
                            .take_while(|c| *c == ' ')
                            .count()
                            / 4)
                            * 4;

                        if num_leading_spaces == editor.get_cursor_pos_in_line()
                            && num_leading_spaces % 4 == 0
                        {
                            // If the number of leading spaces is equivalent to the cursor's current position and
                            // the number of leading spaces is divisible by 4

                            gap_buf.pop_tab();

                            for _ in 0..4 {
                                editor.move_cursor_left();
                            }
                        } else {
                            // Remove the next character in the gap buffer
                            gap_buf.pop();

                            editor.move_cursor_left();
                        }

                        editor.current_buffer().borrow_mut().set_line_at_cursor(
                            editor.get_cursor_doc_row(),
                            gap_buf.to_string(),
                            editor.doc_disp_width(),
                        );
                    } else if cursor_pos / editor.doc_disp_width() != 0 {
                        // If the cursor is not in the first row of the line

                        // Remove the next character in the gap buffer
                        gap_buf.pop();

                        if editor.current_buffer().borrow().visible_rows.0 == 0
                            || editor.get_cursor_vis_row() > editor.doc_disp_home_row()
                        {
                            // If the document's visible rows does include the first row

                            // Move the cursor to the previous row
                            editor.move_cursor_vis_up();
                        } else {
                            // If the document's visible rows does not include the first row

                            editor
                                .current_buffer()
                                .borrow_mut()
                                .push_vis_up(editor.doc_disp_height());
                        }

                        // Move the cursor to the end of the previous row
                        editor.move_cursor_vis_to_editor_right();

                        editor.move_cursor_doc_up();
                        editor.move_cursor_doc_to_editor_right();

                        editor.current_buffer().borrow_mut().set_line_at_cursor(
                            editor.get_cursor_doc_row(),
                            gap_buf.to_string(),
                            editor.doc_disp_width(),
                        );

                        // Reset the view
                    } else if cursor_pos == 0
                        && editor.get_cursor_vis_row() != editor.doc_disp_home_row()
                    {
                        // If the cursor is at the first positon of the line and it is not in the first line of the document
                        // (note: cursor's doc row field is not used during checking because editor_top starts at the same
                        // index that cursor's row starts at)

                        // Get the current line's string
                        let curr_str = editor
                            .current_buffer()
                            .borrow()
                            .get_str_at_cursor(editor.get_cursor_doc_row())
                            .to_owned();

                        // Remove the current line from the document
                        editor.current_buffer().borrow_mut().remove_line_from_doc(
                            editor.get_cursor_doc_row(),
                            editor.doc_disp_width(),
                        );

                        // Move to the previous line
                        editor.move_cursor_up();

                        // Move to the end of the previous line
                        editor.move_cursor_to_end_line();

                        editor.current_buffer().borrow_mut().append_to_line(
                            editor.get_cursor_doc_row(),
                            &curr_str,
                            editor.doc_disp_width(),
                        );

                        // Create a new gap buffer based on the new string at the cursor position
                        gap_buf = GapBuf::from_str(
                            editor
                                .current_buffer()
                                .borrow()
                                .get_str_at_cursor(editor.get_cursor_doc_row())
                                .to_owned(),
                            editor.get_cursor_pos_in_line(),
                        );

                        // Reset the view
                    } else if cursor_pos == 0
                        && editor.current_buffer().borrow().visible_rows.0 != 0
                    {
                        // If the cursor is at the first positon of the line and the first visible row is not the first row of the document

                        // Get the current line's string
                        let curr_str = editor
                            .current_buffer()
                            .borrow()
                            .get_str_at_cursor(editor.get_cursor_doc_row())
                            .to_owned();

                        // Remove the current line from the document
                        editor.current_buffer().borrow_mut().remove_line_from_doc(
                            editor.get_cursor_doc_row(),
                            editor.doc_disp_width(),
                        );

                        editor
                            .current_buffer()
                            .borrow_mut()
                            .push_vis_up(editor.doc_disp_height());

                        // Move to the previous line
                        editor.move_cursor_doc_up();

                        // Move to the end of the previous line
                        editor.move_cursor_to_end_line();

                        editor.current_buffer().borrow_mut().append_to_line(
                            editor.get_cursor_doc_row(),
                            &curr_str,
                            editor.doc_disp_width(),
                        );

                        // Create a new gap buffer based on the new string at the cursor position
                        gap_buf = GapBuf::from_str(
                            editor
                                .current_buffer()
                                .borrow()
                                .get_str_at_cursor(editor.get_cursor_doc_row())
                                .to_owned(),
                            editor.get_cursor_pos_in_line(),
                        );

                        // Reset the view
                    }

                    let new_num_rows = editor.current_buffer().borrow().num_rows();

                    if curr_num_rows == new_num_rows {
                        editor.print_line();
                    } else {
                        editor.reset_editor_view();
                    }
                }
                // Insert a new line character to break line while in insert mode
                c if editor.curr_mode == Modes::Insert
                    && (c as char == ' ' || !(c as char).is_whitespace()) =>
                {
                    // Here, c can only be a non whitespace character except for space
                    if editor.get_cursor_doc_col() < editor.doc_disp_width() {
                        // If adding a new character on the current row will not move past the editor's right edge

                        // Add the character
                        gap_buf.insert(c as char);

                        // Move the cursor to the right
                        editor.move_cursor_right();

                        let curr_line_ind = editor
                            .current_buffer()
                            .borrow()
                            .get_index_at_cursor(editor.get_cursor_doc_row());

                        let num_line_rows = editor.current_buffer().borrow().lines[curr_line_ind]
                            .rows(editor.doc_disp_width())
                            .count();

                        // Set the current line's string content to the gap buffer
                        editor.current_buffer().borrow_mut().set_line_at_cursor(
                            editor.get_cursor_doc_row(),
                            gap_buf.to_string(),
                            editor.doc_disp_width(),
                        );

                        // Reset the view
                        if num_line_rows
                            == editor.current_buffer().borrow().lines[curr_line_ind]
                                .rows(editor.doc_disp_width())
                                .count()
                        {
                            editor.print_line();
                        } else {
                            editor.reset_editor_view();
                        }
                    } else {
                        // If inserting a character will go beyond the editor's right edge (i.e. if the character should begin a new row)

                        // Insert the character into the gap buffer
                        gap_buf.insert(c as char);

                        let curr_line_ind = editor
                            .current_buffer()
                            .borrow()
                            .get_index_at_cursor(editor.get_cursor_doc_row());

                        let num_line_rows = editor.current_buffer().borrow().lines[curr_line_ind]
                            .rows(editor.doc_disp_width())
                            .count();

                        // Set the current line's string content to the gap buffer
                        editor.current_buffer().borrow_mut().set_line_at_cursor(
                            editor.get_cursor_doc_row(),
                            gap_buf.to_string(),
                            editor.doc_disp_width(),
                        );

                        if editor.get_cursor_vis_row() < editor.doc_disp_height() {
                            // If the cursor's row is less than the editor's height

                            // Move down to the next row
                            editor.move_cursor_vis_down();
                        } else {
                            editor.current_buffer().borrow_mut().push_vis_down();
                        }

                        // Move the cursor to the left edge of the editor
                        editor.move_cursor_vis_editor_left();

                        // Move the cursor to the right to provide space for the character that was inserted
                        editor.move_cursor_vis_right();

                        editor.move_cursor_doc_down();
                        editor.move_cursor_doc_editor_left();

                        editor.move_cursor_doc_right();

                        // Reset the view
                        if num_line_rows
                            == editor.current_buffer().borrow().lines[curr_line_ind]
                                .rows(editor.doc_disp_width())
                                .count()
                        {
                            editor.print_line();
                        } else {
                            editor.reset_editor_view();
                        }
                    }
                }
                // Insert a character while in insert mode
                c if editor.curr_mode == Modes::Insert && c == RETURN => {
                    // Collect the two sides of the gap buffer
                    let (lhs, mut rhs) = gap_buf.collect_to_pieces();

                    let num_spaces = (editor
                        .current_buffer()
                        .borrow()
                        .get_line_at_cursor(editor.get_cursor_doc_row())
                        .1
                        .chars()
                        .take_while(|c| *c == ' ')
                        .count()
                        / 4)
                        * 4;

                    rhs = (0..num_spaces).into_iter().map(|_| ' ').collect::<String>() + &rhs;

                    // Set the current line to the left hand side of the gap buffer
                    editor.current_buffer().borrow_mut().set_line_at_cursor(
                        editor.get_cursor_doc_row(),
                        lhs,
                        editor.doc_disp_right_edge(),
                    );

                    // Move to the start of the new line to be created from the right hand side of the gap buffer
                    editor.move_cursor_to_end_line();

                    if editor.get_cursor_vis_row() < editor.doc_disp_height() {
                        // If the cursor's row is less than the editor's height

                        // Move down to the next row
                        editor.move_cursor_vis_down();
                    } else {
                        // If the cursor's row is at the editor's height

                        // Push the visible rows of the document down
                        editor.current_buffer().borrow_mut().push_vis_down();
                    }

                    editor.move_cursor_doc_down();
                    editor.move_cursor_vis_editor_left();
                    editor.move_cursor_doc_editor_left();

                    // This ind_counter variable is created in such a way as to conform with the Line struct's from_str method requiring a mutable reference to a usize variable
                    // this will be addressed later
                    #[allow(unused_mut)]
                    let mut ind_counter = editor.get_cursor_doc_row();

                    let new_line = Line::from_str(rhs, &mut ind_counter, editor.doc_disp_width());

                    editor
                        .current_buffer()
                        .borrow_mut()
                        .add_line_at_row(new_line, editor.get_cursor_doc_row());

                    gap_buf = GapBuf::from_line(
                        editor
                            .current_buffer()
                            .borrow()
                            .get_line_at_cursor(editor.get_cursor_doc_row()),
                        num_spaces,
                    );

                    editor.move_cursor_to_pos(num_spaces);

                    editor.reset_editor_view();
                }
                c if editor.curr_mode == Modes::Insert && c as char == '\t' => {
                    // For now, a tab is represented as four spaces

                    for _ in 0..4 {
                        gap_buf.insert(' ');
                    }

                    let curr_pos = editor.get_cursor_pos_in_line();

                    editor.current_buffer().borrow_mut().set_line_at_cursor(
                        editor.get_cursor_doc_row(),
                        gap_buf.to_string(),
                        editor.doc_disp_width(),
                    );

                    editor.move_cursor_to_pos(curr_pos + 4);

                    editor.reset_editor_view();
                }
                // Enter command mode
                COLON if editor.curr_mode == Modes::Normal => {
                    // Change to command mode
                    editor.change_mode(Modes::Command);

                    editor.initialize_command_row();
                }
                // Execute command while in command mdoe
                RETURN if editor.curr_mode == Modes::Command => {
                    let input = editor.command_buf.borrow().clone();
                    let mut input_iter = input
                        .as_str()
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .into_iter();

                    if let Some(command) = input_iter.next() {
                        match command {
                            "w" => {
                                editor.write_current_buffer_to_file(input_iter.next());

                                editor.exit_command_mode::<String>(None);

                                editor.change_mode(Modes::Normal);
                            }
                            "q" => {
                                break;
                            }
                            "wq" => {
                                editor.write_current_buffer_to_file(input_iter.next());

                                break;
                            }
                            "o" => {
                                while let Some(new_buf) = input_iter.next() {
                                    editor.add_file_buffer(new_buf);
                                }

                                editor.change_mode(Modes::Normal);

                                editor.exit_command_mode::<String>(None);

                                editor.reset_editor_view();
                            }
                            "bc" => {
                                editor.remove_file_buffer();

                                editor.change_mode(Modes::Normal);

                                editor.exit_command_mode::<String>(None);

                                editor.reset_editor_view();
                            }
                            _ => {
                                editor.revert_cursor_vis_pos();
                                editor.print_command_message("Invalid Command");

                                editor.command_buf.borrow_mut().clear();

                                editor.change_mode(Modes::Normal);
                            }
                        }
                    }
                }
                // Delete character while in command mode
                BCKSP | CTRL_H if editor.curr_mode == Modes::Command => {
                    if editor.command_buf.borrow().len() > 0 {
                        // If the buffer is not empty

                        editor.move_cursor_vis_to(
                            editor.command_row(),
                            editor.doc_disp_left_edge() + editor.command_buf.borrow().len() - 1,
                        );

                        editor.pop_command_buf();

                        // Move cursor to just after the original buffer minus the last character
                        editor.move_cursor_vis_to(
                            editor.command_row(),
                            editor.doc_disp_left_edge() + editor.command_buf.borrow().len(),
                        );
                    }
                }
                // Insert character while in command mode
                c if editor.curr_mode == Modes::Command => {
                    // Push the pressed character to the buffer
                    // Display the character to the screen
                    editor.push_command_buf(c as char);
                }

                _ => (),
            }
        }

        editor.flush_pen();

        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}
