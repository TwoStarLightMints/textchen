use crate::cursor::*;
use crate::document::*;
use crate::term::clear_screen;
use crate::term::get_char;
use crate::term::{kbhit, Wh};
use crate::term_color::{Theme, ThemeBuilder};
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufWriter, Read, Stdout, Write};
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

pub struct Editor {
    /// Responsible for holding all information about terminal size, document
    /// display window size, and printing to the screen
    pub term_dimensions: Wh,
    pub left_edge_offset: usize,
    pub right_edge_offset: usize,
    /// Stores the current mode that the editor is in
    pub curr_mode: Modes,
    /// Stores the theme to be used for colors
    /// TODO: Make user configurable
    pub theme: Theme,
    /// The buffer for user entered commands
    pub command_buf: String,
    pen: RefCell<BufWriter<Stdout>>,
}

impl Editor {
    pub fn new(dimensions: Wh, left_edge_offset: usize, right_edge_offset: usize) -> Self {
        //! left_edge_offset - The index of the column at which the document will start
        //! to be displayed in the document display window
        //! right_edge_offset - The amount of spaces from the right side of the terminal
        //! that the document will be displayed

        let theme = ThemeBuilder::new()
            .title_line("31;35;53")
            .mode_line("31;35;53")
            .font_accents("169;177;214")
            .font_body("122;162;247")
            .editor_background("36;40;59")
            .build();

        Self {
            left_edge_offset,
            right_edge_offset,
            curr_mode: Modes::Normal,
            // Note, I am working with only defaults right now
            theme,
            command_buf: String::new(),
            term_dimensions: dimensions,
            pen: RefCell::new(BufWriter::new(io::stdout())),
        }
    }

    // ==================== DISPLAY METHODS FOR EDITOR ====================

    fn reset_color(&self) {
        print!("\u{001b}[0m");
        io::stdout().flush().unwrap();
    }

    fn print_line_color(&self, color: impl AsRef<str>) {
        print!("{}\u{001b}[2K", color.as_ref());
    }

    fn print_text_colored(&self, color: impl AsRef<str>, message: impl AsRef<str>) {
        print!("{}{}", color.as_ref(), message.as_ref());
        self.reset_color();
    }

    fn print_title(&self, document: &Document, cursor: &mut Cursor) {
        cursor.save_current_pos();

        cursor.move_to(0, 0);

        self.print_line_color(self.theme.title_line_color());

        self.print_text_colored(
            self.theme.title_text_color(),
            format!(" {}", &document.file_name),
        );

        self.reset_color();

        cursor.revert_pos();
    }

    fn print_mode_row(&self, cursor: &mut Cursor) {
        cursor.save_current_pos();

        cursor.move_to(self.mode_row(), 0);

        self.print_line_color(self.theme.mode_line_color());

        self.print_text_colored(
            self.theme.title_text_color(),
            format!(
                " {}",
                match self.curr_mode {
                    Modes::Normal => "NOR",
                    Modes::Insert => "INS",
                    Modes::Command => "COM",
                    Modes::MoveTo => "MOV",
                }
            ),
        );

        self.reset_color();

        cursor.revert_pos();
    }

    fn print_command_row(&self, cursor: &mut Cursor) {
        cursor.save_current_pos();

        cursor.move_to(self.command_row(), 0);

        self.print_line_color(self.theme.background_color());

        self.reset_color();

        cursor.revert_pos();
    }

    pub fn change_mode(&mut self, new_mode: Modes, cursor: &mut Cursor) {
        //! curr - Current mode stored in the state of the application
        //! new_mode - The new mode which will be stored in the state of the application
        //! mode_row - The row at which the mode will be printed
        //! cursor - Get control of cursor
        //!
        //! Changes the current mode of the editor to a new target mode, handles changing state and drawing to screen

        self.curr_mode = new_mode;

        cursor.save_current_pos();

        cursor.move_to(self.mode_row(), 0);

        self.print_mode_row(cursor);

        cursor.revert_pos();
    }

    fn print_document(&self, document: &Document, cursor: &mut Cursor) {
        cursor.save_current_pos();

        cursor.move_to(2, self.doc_disp_left_edge());

        if document.visible_rows.0 == 0 {
            // Number of lines in document does not exceed editor height
            for row in document
                .rows(self.doc_disp_width())
                .take(document.visible_rows.1)
            {
                self.print_line_color(self.theme.background_color());
                self.print_text_colored(self.theme.body_text_color(), row.1);

                cursor.move_vis_down();
                cursor.move_to_editor_left(self.doc_disp_left_edge());
            }

            // Since the document is not as big as the editor window, print the last lines
            while cursor.row <= self.doc_disp_bottom() {
                self.print_line_color(self.theme.background_color());
                cursor.move_vis_down();
            }
        } else {
            // Number of lines in document does exceed editor height
            let vis_rows: Vec<_> = document
                .rows(self.doc_disp_width())
                .skip(document.visible_rows.0)
                .take(document.visible_rows.1 - document.visible_rows.0)
                .collect();

            for row in vis_rows.iter() {
                self.print_line_color(self.theme.background_color());
                self.print_text_colored(self.theme.body_text_color(), row.1.as_str());

                cursor.move_vis_down();
                cursor.move_to_editor_left(self.doc_disp_left_edge());
            }

            if vis_rows.len() < (document.visible_rows.1 - document.visible_rows.0) {
                self.print_line_color(self.theme.background_color());
            }
        }

        cursor.revert_pos();
    }

    pub fn print_line(&self, document: &Document, cursor: &mut Cursor) {
        cursor.save_current_pos();

        let curr_line_rows: Vec<(usize, String)> = document
            .get_line_at_cursor(cursor.doc_row)
            .rows(self.doc_disp_width())
            .collect();

        // Get the row number of the first row in the Line, subtract the cursor's position
        // in the rows of the document to get the amount up that the cursor needs to be moved
        // The row number of the first row must be subtracted from the cursor's doc row
        // because cursor.doc_row >= curr_line_rows[0].0
        let diff = cursor.doc_row - curr_line_rows[0].0;

        cursor.move_to_editor_left(self.doc_disp_left_edge());

        for _ in 0..diff {
            cursor.move_vis_up();
        }

        cursor.save_current_pos();

        for _ in 0..curr_line_rows.len() {
            self.print_line_color(self.theme.background_color());
            cursor.move_vis_down();
        }

        cursor.revert_pos();

        cursor.save_current_pos();

        for (_, s) in curr_line_rows {
            self.print_text_colored(self.theme.command_text_color(), s);
            cursor.move_vis_down();
        }

        cursor.revert_pos();

        cursor.revert_pos();
    }

    pub fn initialize_display(&self, document: &Document, cursor: &mut Cursor) {
        self.clear_document_window(cursor);
        self.print_title(document, cursor);
        self.print_document(document, cursor);
        self.print_mode_row(cursor);
        self.print_command_row(cursor);
    }

    pub fn clear_document_window(&self, cursor: &mut Cursor) {
        //! document - Document being edited
        //! cursor - Get control of cursor
        //!
        //! Visually clears the contents of the editor window, the rest of the screen is untouched

        cursor.save_current_pos();

        cursor.move_to(2, 1);

        for _ in 0..self.doc_disp_height() {
            print!("\u{001b}[2K");

            cursor.move_vis_down();
        }

        cursor.revert_pos();
    }

    pub fn display_document(&self, document: &Document, cursor: &mut Cursor) {
        //! document - Document being edited
        //! cursor - Get control of cursor
        //!
        //! Displays the document that is currently being edited to the screen, handles drawing within given bounds

        cursor.save_current_pos();

        cursor.move_to(2, self.doc_disp_left_edge());

        if document.visible_rows.0 == 0 {
            for row in document
                .rows(self.doc_disp_width())
                .take(document.visible_rows.1)
            {
                print!("\u{001b}[2K{}", row.1);
                cursor.move_vis_down();
                cursor.move_to_editor_left(self.doc_disp_left_edge());
            }
        } else {
            for row in document
                .rows(self.doc_disp_width())
                .skip(document.visible_rows.0)
                .take(document.visible_rows.1 - document.visible_rows.0)
            {
                print!("\u{001b}[2K{}", row.1);
                cursor.move_vis_down();
                cursor.move_to_editor_left(self.doc_disp_left_edge());
            }
        }

        if document.visible_rows.1 == *document.lines[document.lines.len()].0.last().unwrap() {
            cursor.move_vis_down();
            self.print_line_color(self.theme.background_color());
        }

        cursor.revert_pos();
    }

    pub fn reset_editor_view(&self, document: &Document, cursor: &mut Cursor) {
        //! document - Document being edited
        //! cursor - Get control of cursor
        //!
        //! Clears the editor screen and redraws the document provided, tends to be used as to refresh the screen after an edit has occurred

        // self.clear_document_window(cursor);

        self.print_document(document, cursor);
    }

    pub fn print_char(&mut self, c: char) {
        match self.curr_mode {
            Modes::Insert => {
                todo!();
            }
            Modes::Command => {
                if c != ':' {
                    self.command_buf.push(c);
                }
                self.print_text_colored(self.theme.command_text_color(), c.to_string());
            }
            Modes::Normal | Modes::MoveTo => unreachable!("Not scientifically possible!"),
        }
    }

    pub fn pop_command_buf(&mut self) {
        self.command_buf.pop();
        self.print_text_colored(self.theme.command_text_color(), " ");
    }

    pub fn print_command_message(&self, message: impl AsRef<str>, cursor: &mut Cursor) {
        cursor.save_current_pos();

        self.print_command_row(cursor);

        cursor.move_to(self.command_row(), 1);
        self.print_text_colored(self.theme.command_text_color(), message);

        cursor.revert_pos();
    }

    pub fn redraw_screen(&mut self, document: &mut Document, cursor: &mut Cursor) {
        //! dimensions - The new dimensions of the terminal screen after resize
        //! self - The old dimensions of the editor screen

        let curr_line_index = document.get_index_at_cursor(cursor.doc_row).unwrap();
        let curr_pos = cursor.get_position_in_line(&document, self);
        let curr_num_above =
            document.num_above_rows(self.doc_disp_width(), document.lines[curr_line_index].0[0]);

        let original_width = self.doc_disp_width();
        let original_height = self.doc_disp_height();

        // Save to see if it will be at least within the right line or an adjacent one instead of only going to the start of the editor
        cursor.save_current_pos();

        // Clear the screen, blank canvas
        clear_screen();

        // Redraw document title
        cursor.move_to(0, 0);
        print!("{}", document.file_name);

        // Return to the previous cursor position
        cursor.revert_pos();

        // Redraw mode
        self.print_mode_row(cursor);

        document.recalculate_indices(self.doc_disp_width());

        // If cursor_half is true, the cursor is located in the top half of the editor, else
        // it is in the bottom half
        let cursor_half = (cursor.row - 2) < self.doc_disp_height() / 2;

        if original_width > self.doc_disp_width() {
            // If the original width is greater than the new width (the screen is shrinking horizontally)

            let new_num_above = document
                .num_above_rows(self.doc_disp_width(), document.lines[curr_line_index].0[0]);

            if new_num_above > curr_num_above {
                // If the number of lines above the current line is greater than the original number of lines
                // that were above the current line

                for _ in 0..(new_num_above - curr_num_above) {
                    document.push_vis_down();
                }
            }
        } else if self.doc_disp_width() > original_width {
            let new_num_above = document
                .num_above_rows(self.doc_disp_width(), document.lines[curr_line_index].0[0]);

            if curr_num_above > new_num_above {
                for _ in 0..(curr_num_above - new_num_above) {
                    document.push_vis_up();
                }
            }
        }

        if original_height > self.doc_disp_height() {
            // If the original height is greater than the new height (the screen is shrinking vertically)

            if cursor_half {
                // If the cursor is in the first half of the editor

                document.visible_rows.1 -= original_height - self.doc_disp_height();
            } else {
                // If the cursor is in the second half of the editor

                document.visible_rows.0 += original_height - self.doc_disp_height();
            }
        } else if self.doc_disp_height() > original_height {
            // If the new height is greater than the original height (the screen is growing vertically)

            if cursor_half {
                // If the cursor is in the first half of the editor

                if document.visible_rows.1 <= *document.lines.last().unwrap().0.last().unwrap() {
                    document.visible_rows.1 += self.doc_disp_height() - original_height;
                } else {
                    if document.visible_rows.0 != 0 {
                        document.visible_rows.0 -= self.doc_disp_height() - original_height;
                    }
                }
            } else {
                // If the cursor is in the second half of the editor

                if document.visible_rows.0 != 0 {
                    document.visible_rows.0 -= self.doc_disp_height() - original_height;
                } else {
                    document.visible_rows.1 += self.doc_disp_height() - original_height;
                }
            }
        }

        cursor.move_to_pos(curr_pos, &document.lines[curr_line_index], &document, self);

        // Redraw document
        self.reset_editor_view(document, cursor);
    }

    pub fn add_to_draw_buf<S: AsRef<str>>(&self, content: S) {
        self.pen
            .borrow_mut()
            .write(content.as_ref().as_bytes())
            .unwrap();
    }

    pub fn flush_pen(&self) {
        self.pen.borrow_mut().flush().unwrap();
    }

    pub fn doc_disp_home_row(&self) -> usize {
        //! The first row on which the document will be displayed
        2
    }

    pub fn doc_disp_height(&self) -> usize {
        //! The height spanned from the first possible row to the last where
        //! the document is displayed
        self.term_dimensions.height - 3
    }

    pub fn doc_disp_bottom(&self) -> usize {
        //! The last row on which the document will be displayed
        self.term_dimensions.height - 2
    }

    pub fn doc_disp_right_edge(&self) -> usize {
        //! The offset from the right side of the terminal, last column
        //! the document will be displayed
        self.term_dimensions.width - self.right_edge_offset
    }

    pub fn doc_disp_left_edge(&self) -> usize {
        //! The offset from the left side of the terminal, first column
        //! the document will be displayed
        self.left_edge_offset
    }

    pub fn doc_disp_width(&self) -> usize {
        //! The width spanned from the first possible column to the last
        //! where the document is displayed
        (self.term_dimensions.width - self.right_edge_offset) - self.left_edge_offset
    }

    pub fn mode_row(&self) -> usize {
        //! The row on which the mode will be displayed
        self.term_dimensions.height - 1
    }

    pub fn command_row(&self) -> usize {
        //! The row on which the user will type commands
        self.term_dimensions.height
    }
}

// ==================== CURSOR HELPER FUNCTIONS ====================

pub fn same_line_different_row_bump(
    cursor_pos: usize,
    editor_dim: &Editor,
    curr_line: &Line,
    next_line: &Line,
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
    } else if (curr_line != next_line && next_line.0[0] > curr_line.0[0] && cursor.doc_column == 1)
        || (curr_line == next_line
            && cursor_pos % editor_dim.doc_disp_width() == 1
            && cursor.doc_row == next_line.0[0])
    {
        // If either:
        //     The current line is not the next line and the next line's first row index is less than the current line's first index and the
        //     cursor's column in relation to the document is 1
        //     The current line is the next line and the cursor's positon mod the editor's width is 1 and the cursor's row in relation to
        //     the document is equal to the next line's first row index

        cursor.move_left();
    }
}

// ==================== DOCUMENT RETRIEVAL AND CREATION ====================

pub fn create_document(file_name: Option<String>, editor_dim: &Editor) -> Document {
    if let Some(ifile) = file_name {
        // If a file has been provided through command line

        // Attempt to open the file provided
        match File::open(&ifile) {
            Ok(mut in_file) => {
                let mut buf = String::new();

                // Read the file contents into the buffer
                in_file.read_to_string(&mut buf).unwrap();

                // Create document struct instance from file contents and editor width
                Document::new(ifile, buf.clone(), editor_dim)
            }
            Err(_) => Document::new(ifile, "".to_string(), editor_dim),
        }
    } else {
        // No file name provided

        // Create new empty document with default name scratch
        Document::new("scratch".to_string(), "".to_string(), &editor_dim)
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
