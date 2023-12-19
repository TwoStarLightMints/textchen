use crate::cursor::*;
use crate::document::*;
use crate::term::clear_screen;
use crate::term::get_char;
use crate::term::print_flush;
use crate::term::{kbhit, Wh};
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Modes {
    Normal,
    Insert,
    Command,
    MoveTo,
}

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

pub fn display_line(
    editor_left_edge: usize,
    editor_width: usize,
    document: &Document,
    cursor: &mut Cursor,
) {
    cursor.save_current_pos();

    let line = document.get_line_at_cursor(cursor.row);

    cursor.move_to_start_line(document, editor_left_edge);

    for (ind, char) in line.1.chars().enumerate() {
        print_flush(format!("{char}").as_str());

        if ind != 0 && (ind + 1) % editor_width == 0 && ind != line.1.len() - 1 {
            cursor.move_down();
            cursor.move_to_editor_left(editor_left_edge);
        }
    }

    cursor.revert_pos();
}

pub fn display_document(document: &Document, editor_dim: &Editor, cursor: &mut Cursor) {
    //! document - Document being edited
    //! editor_left_edge - This is the offset from the left side of the terminal
    //! editor_width - Size of the editor screen, calculated from the left side offset and the right side offset, pass this calculated result to the function
    //! cursor - Get control of cursor
    //!
    //! Displays the document that is currently being edited to the screen, handles drawing within given bounds

    cursor.save_current_pos();

    cursor.move_to(2, editor_dim.editor_left_edge);

    for row in document
        .rows(editor_dim.editor_width)
        .skip(document.visible_lines.0)
    {
        print!("{}", row.1);
        cursor.move_down();
        cursor.move_to_editor_left(editor_dim.editor_left_edge);
    }

    cursor.revert_pos();
}

pub fn clear_editor_window(editor_right_edge: usize, document: &Document, cursor: &mut Cursor) {
    //! editor_right_edge - This is the offset from the right side of the terminal
    //! document - Document being edited
    //! cursor - Get control of cursor
    //!
    //! Visually clears the contents of the editor window, the rest of the screen is untouched

    cursor.save_current_pos();

    cursor.move_to(2, 1);

    for _ in 0..=document.num_rows() {
        // print!("\u{001b}[2K");
        print!("{: >1$}", "", editor_right_edge);
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

    clear_editor_window(editor_dim.editor_right_edge, document, cursor);

    display_document(document, editor_dim, cursor);
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

pub fn redraw_screen(
    dimensions: &Wh,
    curr_mode: &mut Modes,
    document: &mut Document,
    editor_dim: &mut Editor,
    editor_home: &mut (usize, usize),
    cursor: &mut Cursor,
) {
    // Used to return back
    let cursor_pos = cursor.get_position_in_line(&document, editor_dim);

    // Save to see if it will be at least within the right line or an adjacent one instead of only going to the start of the editor
    cursor.save_current_pos();

    editor_dim.editor_right_edge = dimensions.width - 2;
    editor_dim.editor_width = editor_dim.editor_right_edge - editor_dim.editor_left_edge;
    editor_dim.mode_row = dimensions.height - 1;
    editor_dim.command_row = dimensions.height;
    *editor_home = (editor_dim.editor_top, editor_dim.editor_left_edge);

    // Clear the screen, blank canvas
    clear_screen();

    // Redraw document title
    cursor.move_to(0, 0);
    print!("{}", document.file_name);

    document.recalculate_indices(editor_dim.editor_width);

    // Redraw document
    reset_editor_view(document, editor_dim, cursor);

    // Redraw mode
    change_mode(curr_mode, *curr_mode, editor_dim.mode_row, cursor);

    // Return to the previous cursor position
    cursor.revert_pos();

    // Because it is possible that the line at which the cursor was has moved, using the cursor's current row minus two, you can
    // get the index of the line at which the cursor should be, then get the first index of that line's index list and move to
    // that row along with keeping in mind the editor's left edge
    cursor.move_to(
        document.lines[cursor.row - 2].0[0] + 2,
        editor_dim.editor_left_edge,
    );

    // Move to the original position of the cursor within the line
    cursor.move_to_pos_in_line(document, editor_dim, cursor_pos);
}

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
