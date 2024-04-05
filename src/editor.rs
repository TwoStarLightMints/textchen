use crate::cursor::*;
use crate::document::*;
use crate::term::get_char;
use crate::term::term_size;
use crate::term::{clear_screen, switch_to_alt_buf};
use crate::term::{kbhit, Wh};
use crate::term_color::{Theme, ThemeBuilder};
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufWriter, Read, Stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

// ==================== MODE FUNCTIONS AND DEFINITIONS ================

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
    left_edge_offset: usize,
    right_edge_offset: usize,
    /// Stores the current mode that the editor is in
    pub curr_mode: Modes,
    /// Stores the theme to be used for colors
    /// TODO: Make user configurable
    theme: Theme,
    /// The buffer for user entered commands
    pub command_buf: RefCell<String>,
    writer: RefCell<Cursor>,
    buffer: RefCell<BufWriter<Stdout>>,
}

impl Editor {
    pub fn new(left_edge_offset: usize, right_edge_offset: usize) -> Self {
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

        let dimensions = term_size();

        Self {
            left_edge_offset,
            right_edge_offset,
            curr_mode: Modes::Normal,
            // Note, I am working with only defaults right now
            theme,
            command_buf: RefCell::new(String::new()),
            term_dimensions: dimensions,
            buffer: RefCell::new(BufWriter::new(io::stdout())),
            writer: RefCell::new(Cursor::new()),
        }
    }

    // ==================== DISPLAY METHODS FOR EDITOR ====================

    // -------------------- COLOR APPLYING METHODS ------------------------

    fn reset_color(&self) {
        self.add_to_draw_buf("\u{001b}[0m");
    }

    fn print_line_color(&self, color: impl AsRef<str>) {
        self.add_to_draw_buf(format!("{}\u{001b}[2K", color.as_ref()));
    }

    fn print_text_colored(&self, color: impl AsRef<str>, message: impl AsRef<str>) {
        self.add_to_draw_buf(format!("{}{}", color.as_ref(), message.as_ref()));
    }

    // --------------------- PRINTING METHODS ------------------------------

    fn print_title(&self, document: &Document) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(0, 0);

        self.print_line_color(self.theme.title_line_color());
        self.print_text_colored(
            self.theme.title_text_color(),
            format!(" {}", &document.file_name),
        );
        self.reset_color();

        self.revert_cursor_vis_pos();
    }

    fn print_mode_row(&self) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(self.mode_row(), 0);

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

        self.revert_cursor_vis_pos();
    }

    fn print_command_row(&self) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(self.command_row(), 0);

        self.print_line_color(self.theme.background_color());

        self.reset_color();

        self.revert_cursor_vis_pos();
    }

    fn print_document(&self, document: &Document) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(2, self.doc_disp_left_edge());

        if document.visible_rows.0 == 0 && document.visible_rows.1 < self.doc_disp_bottom() {
            // Number of lines in document does not exceed editor height
            for row in document
                .rows(self.doc_disp_width())
                .take(document.visible_rows.1)
            {
                self.print_line_color(self.theme.background_color());
                self.print_text_colored(self.theme.body_text_color(), row.1);

                self.move_cursor_vis_down();
                self.move_cursor_vis_editor_left();
            }

            // Since the document is not as big as the editor window, print the last lines
            while self.get_cursor_vis_row() <= self.doc_disp_bottom() {
                self.print_line_color(self.theme.background_color());
                self.move_cursor_vis_down();
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

                self.move_cursor_vis_down();
                self.move_cursor_vis_editor_left();
            }

            if vis_rows.len() < self.doc_disp_height() {
                self.print_line_color(self.theme.background_color());
            }
        }

        self.revert_cursor_vis_pos();
    }

    pub fn print_line(&self, document: &Document) {
        self.save_cursor_vis_pos();

        let curr_line_rows: Vec<(usize, String)> = document
            .get_line_at_cursor(self.get_cursor_doc_row())
            .rows(self.doc_disp_width())
            .collect();

        // Get the row number of the first row in the Line, subtract the cursor's position
        // in the rows of the document to get the amount up that the cursor needs to be moved
        // The row number of the first row must be subtracted from the cursor's doc row
        // because cursor.doc_row >= curr_line_rows[0].0
        let diff = self.get_cursor_doc_row() - curr_line_rows[0].0;

        self.move_cursor_vis_editor_left();

        for _ in 0..diff {
            self.move_cursor_vis_up();
        }

        self.save_cursor_vis_pos();

        for _ in 0..curr_line_rows.len() {
            self.print_line_color(self.theme.background_color());
            self.move_cursor_vis_down();
        }

        self.revert_cursor_vis_pos();

        self.save_cursor_vis_pos();

        for (_, s) in curr_line_rows {
            self.print_text_colored(self.theme.command_text_color(), s);
            self.move_cursor_vis_down();
        }

        self.revert_cursor_vis_pos();

        self.revert_cursor_vis_pos();
    }

    pub fn initialize_display(&self, document: &Document) {
        self.add_to_draw_buf(switch_to_alt_buf());
        self.clear_doc_disp_window();
        self.print_title(document);
        self.print_document(document);
        self.print_mode_row();
        self.print_command_row();
        self.move_cursor_vis_to(self.doc_disp_home_row(), self.doc_disp_left_edge());
        self.flush_pen();
    }

    fn clear_line(&self, color: impl AsRef<str>) {
        self.add_to_draw_buf(format!("\u{001b}[2K{}", color.as_ref()));
    }

    pub fn clear_doc_disp_window(&self) {
        //! document - Document being edited
        //! cursor - Get control of cursor
        //!
        //! Visually clears the contents of the editor window, the rest of the screen is untouched

        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(2, 1);

        for _ in 0..self.doc_disp_height() {
            self.clear_line(self.theme.background_color());

            self.move_cursor_vis_down();
        }

        self.revert_cursor_vis_pos();
    }

    pub fn reset_editor_view(&self, document: &Document) {
        //! document - Document being edited
        //! cursor - Get control of cursor
        //!
        //! Clears the editor screen and redraws the document provided, tends to be used as to refresh the screen after an edit has occurred

        self.clear_doc_disp_window();

        self.print_document(document);
    }

    pub fn print_char(&self, c: char) {
        match self.curr_mode {
            Modes::Insert => {
                todo!();
            }
            Modes::Command => {
                if c != ':' {
                    self.command_buf.borrow_mut().push(c);
                }
                self.print_text_colored(self.theme.command_text_color(), c.to_string());
            }
            Modes::Normal | Modes::MoveTo => unreachable!("Not scientifically possible!"),
        }
    }

    pub fn redraw_screen(&mut self, document: &mut Document) {
        //! dimensions - The new dimensions of the terminal screen after resize
        //! self - The old dimensions of the editor screen

        let curr_line_index = document.get_index_at_cursor(self.get_cursor_doc_row());
        let curr_pos = self
            .writer
            .borrow_mut()
            .get_position_in_line(&document, self);
        let curr_num_above =
            document.num_above_rows(self.doc_disp_width(), document.lines[curr_line_index].0[0]);

        let original_width = self.doc_disp_width();
        let original_height = self.doc_disp_height();

        // Save to see if it will be at least within the right line or an adjacent one instead of only going to the start of the editor
        self.save_cursor_vis_pos();

        // Clear the screen, blank canvas
        clear_screen();

        // Redraw document title
        self.move_cursor_vis_to(0, 0);
        self.add_to_draw_buf(format!("{}", document.file_name));

        // Return to the previous cursor position
        self.revert_cursor_vis_pos();

        // Redraw mode
        self.print_mode_row();

        document.recalculate_indices(self.doc_disp_width());

        // If cursor_half is true, the cursor is located in the top half of the editor, else
        // it is in the bottom half
        let cursor_half = (self.get_cursor_vis_row() - 2) < self.doc_disp_height() / 2;

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
                    document.push_vis_up(self.doc_disp_height());
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

        self.move_cursor_to_pos(curr_pos, &document.lines[curr_line_index], &document);

        // Redraw document
        self.reset_editor_view(document);
    }

    // -------------------- PRINT BUFFER MANIPULATION ---------------------

    pub fn add_to_draw_buf<S: AsRef<str>>(&self, content: S) {
        self.buffer
            .borrow_mut()
            .write(content.as_ref().as_bytes())
            .unwrap();
    }

    pub fn flush_pen(&self) {
        self.buffer.borrow_mut().flush().unwrap();
    }

    // ==================== CURSOR WRAPPER FUNCTIONS ======================

    pub fn get_cursor_pos_in_line(&self, document: &Document) -> usize {
        self.writer.borrow().get_position_in_line(document, self)
    }

    // -------------------- CURSOR INFORMATION RETRIEVAL ------------------

    pub fn get_cursor_vis_row(&self) -> usize {
        self.writer.borrow().row
    }

    pub fn get_cursor_vis_col(&self) -> usize {
        self.writer.borrow().column
    }

    pub fn get_cursor_doc_row(&self) -> usize {
        self.writer.borrow().doc_row
    }

    pub fn get_cursor_doc_col(&self) -> usize {
        self.writer.borrow().doc_column
    }

    // -------------------- CURSOR HISTORY MANIPULATION -------------------

    pub fn save_cursor_vis_pos(&self) {
        self.writer.borrow_mut().save_current_pos();
    }

    pub fn revert_cursor_vis_pos(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().revert_pos());
    }

    // -------------------- CURSOR RELATIVE TO DOCUMENT -------------------

    pub fn get_cursor_column_in_doc_disp(&self) -> usize {
        self.writer.borrow().column - self.doc_disp_left_edge()
    }

    pub fn move_cursor_to_pos(&self, new_pos: usize, current_line: &Line, document: &Document) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to_pos(
            new_pos,
            current_line,
            document,
            self,
        ));
    }

    pub fn move_cursor_to_start_line(&self, document: &mut Document) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to_start_line(document, self));
    }

    pub fn move_cursor_to_end_line(&self, document: &mut Document) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to_end_line(document, self));
    }

    // -------------------- CURSOR MOVEMENT -------------------------------

    pub fn move_cursor_vis_to(&self, new_row: usize, new_column: usize) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to(new_row, new_column));
    }
    pub fn move_cursor_doc_to(&self, new_doc_row: usize, new_doc_col: usize) {
        self.writer
            .borrow_mut()
            .move_doc_to(new_doc_row, new_doc_col);
    }

    pub fn move_cursor_down(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_down());
    }

    pub fn move_cursor_up(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_up());
    }

    pub fn move_cursor_right(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_right());
    }

    pub fn move_cursor_left(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_left());
    }

    pub fn move_cursor_vis_down(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_vis_down());
    }

    pub fn move_cursor_vis_up(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_vis_up());
    }

    pub fn move_cursor_vis_right(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_vis_right());
    }

    pub fn move_cursor_vis_left(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_vis_left());
    }

    pub fn move_cursor_doc_down(&self) {
        self.writer.borrow_mut().move_doc_down()
    }

    pub fn move_cursor_doc_up(&self) {
        self.writer.borrow_mut().move_doc_up()
    }

    pub fn move_cursor_doc_right(&self) {
        self.writer.borrow_mut().move_doc_right()
    }

    pub fn move_cursor_doc_left(&self) {
        self.writer.borrow_mut().move_doc_left()
    }

    pub fn move_cursor_doc_to_editor_right(&self) {
        self.writer.borrow_mut().doc_column = self.doc_disp_width();
    }

    pub fn move_cursor_vis_to_editor_right(&self) {
        let curr_row = self.get_cursor_vis_row();

        self.move_cursor_vis_to(curr_row, self.doc_disp_right_edge());
    }

    pub fn move_cursor_vis_editor_left(&self) {
        self.add_to_draw_buf(
            self.writer
                .borrow_mut()
                .move_to_editor_left(self.doc_disp_left_edge()),
        );
    }

    pub fn move_cursor_doc_editor_left(&self) {
        self.writer.borrow_mut().move_doc_to_editor_left();
    }

    pub fn same_line_different_row_bump(&self, document: &Document) {
        //! cursor_pos : This position is the position before having moved the cursor
        //! curr_line : This is the line before moving
        //! next_line : This is the line *AFTER* moving
        //!
        //! This function is used to move a cursor to the appropriate position within a line when moving vertically
        //! "Appropriate" here means that if the cursor is in a row of a line other than the beginning line, the very first position the
        //! cursor should be able to take is on top of the second character of the row

        let curr_line = document.get_line_at_cursor(self.get_cursor_pos_in_line(document));

        if self.get_cursor_pos_in_line(document) / self.doc_disp_width()
            == *curr_line.0.first().unwrap()
        {
            // If after the cursor moved it is at the first row in the line

            if self.get_cursor_doc_col() == 1 {
                // The cursor is now at index 1 of the line, so it needs to be moved to the left to give the
                // proper bump effect

                self.move_cursor_left();
            }
        } else {
            // If after the cursor moved it is on any other row

            if self.get_cursor_doc_col() == 0 {
                // The cursor is now touching the left edge of the doc display area and in any of the rows
                // that are not the first row, so we need to bump it to the right

                self.move_cursor_right();
            }
        }
    }

    // ==================== DIMENSIONS ====================================

    // -------------------- DIMENSION INFORMATION -------------------------

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

    // -------------------- DIMENSION MANIPULATION ------------------------

    pub fn check_resize(&mut self) -> bool {
        let checker = term_size();

        if checker.width != self.term_dimensions.width
            || checker.height != self.term_dimensions.height
        {
            self.term_dimensions.width = checker.width;
            self.term_dimensions.height = checker.height;

            return true;
        }

        false
    }

    // ============================== MODE ================================

    pub fn change_mode(&mut self, new_mode: Modes) {
        //! curr - Current mode stored in the state of the application
        //! new_mode - The new mode which will be stored in the state of the application
        //! mode_row - The row at which the mode will be printed
        //! cursor - Get control of cursor
        //!
        //! Changes the current mode of the editor to a new target mode, handles changing state and drawing to screen

        self.curr_mode = new_mode;

        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(self.mode_row(), 0);

        self.print_mode_row();

        self.revert_cursor_vis_pos();
    }

    // ============================== COMMAND =============================

    pub fn initialize_command_row(&self) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(self.command_row(), 1);

        self.print_line_color(self.theme.background_color());

        self.print_char(':');

        self.move_cursor_vis_right();
    }

    pub fn print_command_message(&self, message: impl AsRef<str>) {
        self.save_cursor_vis_pos();

        self.print_command_row();

        self.move_cursor_vis_to(self.command_row(), 1);
        self.print_text_colored(self.theme.command_text_color(), message);

        self.revert_cursor_vis_pos();
    }

    pub fn exit_command_mode<S: AsRef<str>>(&self, message: Option<S>) {
        self.save_cursor_vis_pos();

        match message.as_ref() {
            Some(m) => self.print_command_message(m),
            None => self.print_command_message(""),
        };

        self.command_buf.borrow_mut().clear();

        self.revert_cursor_vis_pos();
    }

    pub fn pop_command_buf(&self) {
        self.command_buf.borrow_mut().pop();
        self.print_text_colored(self.theme.command_text_color(), " ");
    }
}

// ==================== COMMAND ============================================

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

// ==================== INPUT RETRIEVAL FUNCTION ===========================

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
