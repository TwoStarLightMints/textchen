use crate::cursor::*;
use crate::document::*;
use crate::term::clear_screen;
use crate::term::get_char;
use crate::term::{kbhit, Wh};
use std::fs::File;
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

// ==================== MODE FUNCTIONS AND DEFINITIONS ====================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Modes {
    Normal,
    Insert,
    Command,
    MoveTo,
}

pub fn change_mode(curr: &mut Modes, new_mode: Modes, mode_row: usize, cursor: &mut Cursor) {
    //! curr - Current mode stored in the state of the application
    //! new_mode - The new mode which will be stored in the state of the application
    //! mode_row - The row at which the mode will be printed
    //! cursor - Get control of cursor
    //!
    //! Changes the current mode of the editor to a new target mode, handles changing state and drawing to screen

    *curr = new_mode;

    cursor.save_current_pos();

    cursor.move_to(mode_row, 0);

    match curr {
        Modes::Normal => print!("NOR"),
        Modes::Insert => print!("INS"),
        Modes::Command => print!("COM"),
        Modes::MoveTo => print!("MOV"),
    };

    io::stdout().flush().unwrap();

    cursor.revert_pos();
}

// ==================== EDITOR DIMENSIONS STRUCT ====================

pub struct Editor {
    pub editor_top: usize,
    pub editor_bottom: usize,
    pub editor_left_edge: usize,
    pub editor_right_edge: usize,
    pub editor_width: usize,
    pub editor_height: usize,
    pub mode_row: usize,
    pub command_row: usize,
}

impl Editor {
    pub fn new(dimensions: Wh, editor_left_edge: usize, editor_right_edge: usize) -> Self {
        Self {
            editor_top: 2,
            editor_bottom: dimensions.height - 2,
            editor_left_edge,
            editor_right_edge,
            editor_width: editor_right_edge - editor_left_edge,
            editor_height: dimensions.height - 3,
            mode_row: dimensions.height - 1,
            command_row: dimensions.height,
        }
    }
}

// ==================== DISPLAY METHODS FOR EDITOR ====================

pub fn display_document(document: &Document, editor_dim: &Editor, cursor: &mut Cursor) {
    //! document - Document being edited
    //! editor_left_edge - This is the offset from the left side of the terminal
    //! editor_width - Size of the editor screen, calculated from the left side offset and the right side offset, pass this calculated result to the function
    //! cursor - Get control of cursor
    //!
    //! Displays the document that is currently being edited to the screen, handles drawing within given bounds

    cursor.save_current_pos();

    cursor.move_to(2, editor_dim.editor_left_edge);

    if document.visible_rows.0 == 0 {
        for row in document
            .rows(editor_dim.editor_width)
            .take(document.visible_rows.1)
        {
            print!("{}", row.1);
            cursor.move_down();
            cursor.move_to_editor_left(editor_dim.editor_left_edge);
        }
    } else {
        for row in document
            .rows(editor_dim.editor_width)
            .skip(document.visible_rows.0)
            .take(document.visible_rows.1 - document.visible_rows.0)
        {
            print!("{}", row.1);
            cursor.move_down();
            cursor.move_to_editor_left(editor_dim.editor_left_edge);
        }
    }

    cursor.revert_pos();
}

pub fn clear_editor_window(editor_dim: &Editor, cursor: &mut Cursor) {
    //! editor_right_edge - This is the offset from the right side of the terminal
    //! document - Document being edited
    //! cursor - Get control of cursor
    //!
    //! Visually clears the contents of the editor window, the rest of the screen is untouched

    cursor.save_current_pos();

    cursor.move_to(2, 1);

    for _ in 0..editor_dim.editor_height {
        print!("{: >1$}", "", editor_dim.editor_right_edge);

        cursor.move_down();
    }

    cursor.revert_pos();
}

pub fn reset_editor_view(document: &Document, editor_dim: &Editor, cursor: &mut Cursor) {
    //! editor_right_edge - This is the offset from the right side of the terminal
    //! editor_left_edge - This is the offset from the left side of the terminal
    //! document - Document being edited
    //! cursor - Get control of cursor
    //!
    //! Clears the editor screen and redraws the document provided, tends to be used as to refresh the screen after an edit has occurred

    clear_editor_window(editor_dim, cursor);

    display_document(document, editor_dim, cursor);
}

pub fn redraw_screen(
    dimensions: &Wh,
    curr_mode: &mut Modes,
    document: &mut Document,
    editor_dim: &mut Editor,
    editor_home_row: usize,
    cursor: &mut Cursor,
) {
    //! dimensions - The new dimensions of the terminal screen after resize
    //! editor_dim - The old dimensions of the editor screen

    let curr_line_ind = document.get_index_at_cursor(cursor.doc_row).unwrap();
    let curr_line_indices = document.lines[curr_line_ind].0.clone();
    let curr_index_in_line = cursor.doc_row;

    let curr_num_above =
        document.num_above_rows(editor_dim.editor_width, document.lines[curr_line_ind].0[0]);

    let curr_height = editor_dim.editor_height;
    let curr_width = editor_dim.editor_width;

    // Save to see if it will be at least within the right line or an adjacent one instead of only going to the start of the editor
    cursor.save_current_pos();

    // Recalculate the values of the editor_dim variable
    editor_dim.editor_right_edge = dimensions.width - 2;
    editor_dim.editor_width = editor_dim.editor_right_edge - editor_dim.editor_left_edge;
    editor_dim.editor_height = dimensions.height - 3;
    editor_dim.mode_row = dimensions.height - 1;
    editor_dim.command_row = dimensions.height;

    // Clear the screen, blank canvas
    clear_screen();

    // Redraw document title
    cursor.move_to(0, 0);
    print!("{}", document.file_name);

    if curr_width != editor_dim.editor_width {
        // If the original width is not equal to the new width

        // Recalculate the indices of the lines
        document.recalculate_indices(editor_dim.editor_width);
    }

    // Return to the previous cursor position
    cursor.revert_pos();

    if curr_height != editor_dim.editor_height {
        // If the original height is not equal to the new width

        if (cursor.row - 2) <= (curr_height / 2) {
            // If the cursor's visual row within the editor is located at the middle of or in the upper half of the editor screen

            if curr_height > editor_dim.editor_height {
                document.visible_rows.1 -= curr_height - editor_dim.editor_height;
            } else {
                document.visible_rows.1 += editor_dim.editor_height - curr_height;
            }
        } else {
            // If the cursor's visual row within the editor is located in the lower half of the editor screen

            if document.visible_rows.0 == 0 {
                if curr_height < editor_dim.editor_height {
                    // If the document's first visible row is 0 and the original height is less than
                    // the new height

                    document.visible_rows.1 += editor_dim.editor_height - curr_height;
                } else if curr_height > editor_dim.editor_height {
                    // If the document's first visible row is 0 and the original height is greater than
                    // the new height

                    if cursor.doc_row == document.visible_rows.1 - 2 {
                        document.visible_rows.0 += curr_height - editor_dim.editor_height;
                    } else {
                        document.visible_rows.1 -= curr_height - editor_dim.editor_height;
                    }
                }
            } else {
                // Since document.visible_rows's parts are usize, below 0 is not possible and
                // thus this branch handles above 0 values

                if curr_height < editor_dim.editor_height {
                    // If the document's first visible row is greater than 0 and the original height
                    // is less than the new height

                    document.visible_rows.0 -= editor_dim.editor_height - curr_height;
                } else if curr_height > editor_dim.editor_height {
                    // If original height is greater than the new height and the cursor is not at the
                    // second to last visible row

                    if cursor.doc_row == document.visible_rows.1 - 2 {
                        document.visible_rows.1 -= curr_height - editor_dim.editor_height;
                    } else {
                        document.visible_rows.0 += curr_height - editor_dim.editor_height;
                    }
                }
            }
        }
    }

    if curr_width != editor_dim.editor_width {
        if cursor.doc_row != curr_line_indices[0] {
            // If the cursor's row in relation to the document is not at the first possible row of the original line

            if cursor.doc_column == editor_dim.editor_width + 1 {
                // If the cursor is at the last possible position within the row

                cursor.move_down();
                cursor.move_to_editor_left(editor_dim.editor_left_edge);
                cursor.move_right();

                cursor.move_doc_down();
                cursor.move_doc_to_editor_left();
                cursor.move_doc_right();
            } else if cursor.doc_row <= editor_dim.editor_width {
                // If the cursor is within the row

                cursor.move_right();
                cursor.move_doc_right();
            }
        }

        // If row delta is greater than 0, that means there is a difference in the number of rows
        // and the new number of rows is less than the original, if the delta is negative, the new
        // number of rows is greater than the original, and if they are equal it will be 0

        if document.num_above_rows(editor_dim.editor_width, document.lines[curr_line_ind].0[0])
            != curr_num_above
        {
            // If the number of rows above the current line is greater than the number of rows prior

            let row_delta = (document
                .num_above_rows(editor_dim.editor_width, document.lines[curr_line_ind].0[0])
                as i32)
                - (curr_num_above as i32);

            if row_delta > 0 {
                // If there are more rows than previous

                for _ in 0..row_delta {
                    cursor.move_down();
                    cursor.move_doc_down();
                }
            } else if row_delta < 0 {
                // If there are less rows than previous

                for _ in 0..(row_delta * -1) {
                    cursor.move_up();
                    cursor.move_doc_up();
                }
            }
        }

        // if document.num_above_rows(editor_dim.editor_width, cursor.doc_row)
        //     > original_num_rows_above
        // {
        //     // If the amount of rows increased after resizing

        //     for _ in 0..(document.num_above_rows(editor_dim.editor_width, cursor.doc_row)
        //         - original_num_rows_above)
        //     {
        //         cursor.move_down();
        //     }
        // } else if document.num_above_rows(editor_dim.editor_width, cursor.doc_row)
        //     < original_num_rows_above
        // {
        //     // If the amount of rows decreased after resizing

        //     for _ in 0..(original_num_rows_above
        //         - document.num_above_rows(editor_dim.editor_width, cursor.doc_row))
        //     {
        //         cursor.move_up();
        //     }
        // }
    }

    // Redraw document
    reset_editor_view(document, editor_dim, cursor);

    // Redraw mode
    change_mode(curr_mode, *curr_mode, editor_dim.mode_row, cursor);

    // Move to the original position of the cursor within the line
    // cursor.reposition_after_resize(document, editor_dim, editor_dim_change);
}

// ==================== CURSOR HELPER FUNCTIONS ====================

pub fn same_line_different_row_bump(
    cursor_pos: usize,
    editor_dim: &Editor,
    curr_line: Line,
    next_line: Line,
    document: &Document,
    cursor: &mut Cursor,
) {
    //! cursor_pos : This position is the position before having moved the cursor
    //! curr_line : This is the line before moving
    //! next_line : This is the line *AFTER* moving
    //!
    //! This function is used to move a cursor to the appropriate position within a line when moving vertically
    //! "Appropriate" here means that if the cursor is in a row of a line other than the beginning line, the very first position the
    //! cursor should be able to take is on top of the second character of the row

    // TODO: Fix moving back and forth at the home position of the editor

    if cursor_pos == 0
        && ((curr_line == next_line
            && curr_line.0.len() > 1
            && cursor_pos != cursor.get_position_in_line(document, editor_dim))
            || (next_line.0.len() > 1 && cursor.doc_row != next_line.0[0]))
    {
        // If the cursor's position is 0 (first position in line) and either:
        //     The current line is the same as the next line, the current line is a multiline, and the cursor's current position is not equal
        //     to the new position of the cursor
        //     The next line is a multiline and the cursor's row in relation to the document is not equal to the next line's first row index

        cursor.move_right();
        cursor.move_doc_right();
    } else if (curr_line != next_line && next_line.0[0] > curr_line.0[0] && cursor.doc_column == 1)
        || (curr_line == next_line
            && cursor_pos % editor_dim.editor_width == 1
            && cursor.doc_row == next_line.0[0])
    {
        // If either:
        //     The current line is not the next line and the next line's first row index is less than the current line's first index and the
        //     cursor's column in relation to the document is 1
        //     The current line is the next line and the cursor's positon mod the editor's width is 1 and the cursor's row in relation to
        //     the document is equal to the next line's first row index

        cursor.move_left();
        cursor.move_doc_left();
    }
}

// ==================== INPUT RETRIEVAL FUNCTION ====================

pub fn spawn_char_channel() -> Receiver<char> {
    //! (kill sender, the receiver for the character)

    let (from_thread, to_use) = mpsc::channel::<char>();

    thread::spawn(move || loop {
        // Here, you need to check for a keyboard hit before trying to send a character
        // because otherwise when quiting the editor, the program will wait till the
        // user enters a key to exit
        if kbhit() {
            match from_thread.send(get_char()) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
    });

    to_use
}
