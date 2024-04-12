use crate::term::{get_char, kbhit, switch_to_alt_buf, term_size, Wh};
use crate::term_color::{Theme, ThemeBuilder};
use crate::{cursor::*, document::*};
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Stdout, Write};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
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
    term_dimensions: Wh,
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
    draw_buffer: RefCell<BufWriter<Stdout>>,
    file_buffers: Vec<Rc<RefCell<Document>>>,
    active_buffer: usize,
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
            draw_buffer: RefCell::new(BufWriter::new(io::stdout())),
            writer: RefCell::new(Cursor::new()),
            file_buffers: Vec::new(),
            active_buffer: 0,
        }
    }

    // ==================== DISPLAY METHODS FOR EDITOR ====================

    // -------------------- COLOR APPLYING METHODS ------------------------

    fn apply_reset_color(&self) {
        self.add_to_draw_buf("\u{001b}[0m");
    }

    fn apply_line_color(&self, color: impl AsRef<str>) {
        self.add_to_draw_buf(format!("{}\u{001b}[2K", color.as_ref()));
    }

    // --------------------- PRINTING METHODS ------------------------------

    fn print_text_w_color(&self, color: impl AsRef<str>, message: impl AsRef<str>) {
        self.add_to_draw_buf(format!("{}{}", color.as_ref(), message.as_ref()));
    }

    fn print_title(&self) {
        let document = Rc::clone(&self.current_buffer());

        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(0, 0);

        self.apply_line_color(self.theme.title_line_color());

        self.print_text_w_color(
            self.theme.title_text_color(),
            format!(" {}", &document.borrow().file_name),
        );

        self.apply_reset_color();

        self.revert_cursor_vis_pos();
    }

    fn print_mode_row(&self) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(self.mode_row(), 0);

        self.apply_line_color(self.theme.mode_line_color());

        self.print_text_w_color(
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

        self.apply_reset_color();

        self.revert_cursor_vis_pos();
    }

    fn print_command_row(&self) {
        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(self.command_row(), 0);

        self.apply_line_color(self.theme.background_color());

        self.apply_reset_color();

        self.revert_cursor_vis_pos();
    }

    fn print_document(&self) {
        let document = Rc::clone(&self.current_buffer());

        self.save_cursor_vis_pos();

        self.move_cursor_vis_to(2, self.doc_disp_left_edge());

        self.apply_line_color(self.theme.background_color());

        if document.borrow().visible_rows.0 == 0
            && document.borrow().visible_rows.1 < self.doc_disp_bottom()
        {
            // Number of lines in document does not exceed editor height
            for row in document
                .borrow()
                .rows(self.doc_disp_width())
                .take(document.borrow().visible_rows.1)
            {
                self.print_text_w_color(self.theme.body_text_color(), row.1);

                self.move_cursor_vis_down();
                self.move_cursor_vis_editor_left();
            }

            // Since the document is not as big as the editor window, print the last lines
            while self.get_cursor_vis_row() <= self.doc_disp_bottom() {
                self.move_cursor_vis_down();
            }
        } else {
            // Number of lines in document does exceed editor height
            for row in document
                .borrow()
                .rows(self.doc_disp_width())
                .skip(document.borrow().visible_rows.0)
                .take(document.borrow().visible_rows.1 - document.borrow().visible_rows.0)
            {
                self.print_text_w_color(self.theme.body_text_color(), row.1);

                self.move_cursor_vis_down();
                self.move_cursor_vis_editor_left();
            }
        }

        self.apply_reset_color();

        self.revert_cursor_vis_pos();
    }

    pub fn print_line(&self) {
        self.save_cursor_vis_pos();

        let binding = Rc::clone(&self.current_buffer());

        let document = binding.borrow();

        let curr_line_rows: Vec<(usize, &str)> = document
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

        self.apply_line_color(self.theme.background_color());

        for _ in 0..curr_line_rows.len() {
            self.move_cursor_vis_down();
        }

        self.revert_cursor_vis_pos();

        self.save_cursor_vis_pos();

        for (_, s) in curr_line_rows {
            self.print_text_w_color(self.theme.command_text_color(), s);
            self.move_cursor_vis_down();
        }

        self.revert_cursor_vis_pos();

        self.revert_cursor_vis_pos();

        self.apply_reset_color();
    }

    fn initialize_display(&self) {
        self.add_to_draw_buf(switch_to_alt_buf());
        self.clear_doc_disp_window();
        self.print_title();
        self.print_document();
        self.print_mode_row();
        self.print_command_row();
        self.move_cursor_vis_to(self.doc_disp_home_row(), self.doc_disp_left_edge());
        self.flush_pen();
    }

    pub fn initialize(&mut self) {
        let mut file_names = env::args().skip(1);

        while let Some(file_name) = file_names.next() {
            self.add_file_buffer(&file_name);
        }

        self.set_active_buffer_start();

        self.initialize_display();
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

    pub fn reset_editor_view(&self) {
        //! document - Document being edited
        //! cursor - Get control of cursor
        //!
        //! Clears the editor screen and redraws the document provided, tends to be used as to refresh the screen after an edit has occurred

        self.clear_doc_disp_window();

        // self.print_title();

        self.print_document();
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
                self.print_text_w_color(self.theme.command_text_color(), c.to_string());
            }
            Modes::Normal | Modes::MoveTo => unreachable!("Not scientifically possible!"),
        }
    }

    pub fn redraw_screen(&self) {
        //! dimensions - The new dimensions of the terminal screen after resize
        //! self - The old dimensions of the editor screen

        let document = Rc::clone(&self.current_buffer());

        document
            .borrow_mut()
            .recalculate_indices(self.doc_disp_width());

        document.borrow_mut().visible_rows.1 =
            document.borrow().visible_rows.0 + self.doc_disp_height();

        self.initialize_display();
    }

    // -------------------- PRINT BUFFER MANIPULATION ---------------------

    pub fn add_to_draw_buf<S: AsRef<str>>(&self, content: S) {
        self.draw_buffer
            .borrow_mut()
            .write(content.as_ref().as_bytes())
            .unwrap();
    }

    pub fn flush_pen(&self) {
        self.draw_buffer.borrow_mut().flush().unwrap();
    }

    // ==================== CURSOR WRAPPER FUNCTIONS ======================

    pub fn get_cursor_pos_in_line(&self) -> usize {
        self.writer.borrow().get_position_in_line(self)
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

    pub fn move_cursor_to_pos(&self, new_pos: usize) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to_pos(new_pos, self));
    }

    pub fn move_cursor_to_start_line(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to_start_line(self));
    }

    pub fn move_cursor_to_end_line(&self) {
        self.add_to_draw_buf(self.writer.borrow_mut().move_to_end_line(self));
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

    pub fn multi_row_bump(&self) {
        //! cursor_pos : This position is the position before having moved the cursor
        //! curr_line : This is the line before moving
        //! next_line : This is the line *AFTER* moving
        //!
        //! This function is used to move a cursor to the appropriate position within a line when moving vertically
        //! "Appropriate" here means that if the cursor is in a row of a line other than the beginning line, the very first position the
        //! cursor should be able to take is on top of the second character of the row

        if self.get_cursor_pos_in_line() / self.doc_disp_width() == 0 {
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

    pub fn check_resize(&mut self) {
        let checker = term_size();

        if checker.width != self.term_dimensions.width
            || checker.height != self.term_dimensions.height
        {
            self.term_dimensions = checker;

            self.redraw_screen();
        }
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

        self.apply_line_color(self.theme.background_color());

        self.print_char(':');

        self.move_cursor_vis_right();
    }

    pub fn print_command_message(&self, message: impl AsRef<str>) {
        self.save_cursor_vis_pos();

        self.print_command_row();

        self.move_cursor_vis_to(self.command_row(), 1);
        self.print_text_w_color(self.theme.command_text_color(), message);

        self.revert_cursor_vis_pos();
    }

    pub fn exit_command_mode<S: AsRef<str>>(&self, message: Option<S>) {
        match message.as_ref() {
            Some(m) => self.print_command_message(m),
            None => self.print_command_message(""),
        };

        self.command_buf.borrow_mut().clear();

        self.revert_cursor_vis_pos();
    }

    pub fn pop_command_buf(&self) {
        self.command_buf.borrow_mut().pop();
        self.print_text_w_color(self.theme.command_text_color(), " ");
    }

    pub fn add_file_buffer(&mut self, file_name: &str) {
        if self.file_buffers.len() == 0 {
            self.file_buffers
                .push(Rc::new(RefCell::new(Document::new(file_name, &self))));
        } else {
            self.file_buffers
                .push(Rc::new(RefCell::new(Document::new(file_name, &self))));

            self.active_buffer = self.file_buffers.len() - 1;
        }
    }

    pub fn remove_file_buffer(&mut self) {
        self.file_buffers.remove(self.active_buffer);

        if self.active_buffer != 0 {
            self.active_buffer -= 1;
        } else {
            self.file_buffers
                .push(Rc::new(RefCell::new(Document::new_scratch(
                    self.doc_disp_height(),
                ))));
        }
    }

    pub fn current_buffer(&self) -> Rc<RefCell<Document>> {
        Rc::clone(&self.file_buffers[self.active_buffer])
    }

    pub fn set_active_buffer_start(&mut self) {
        if self.file_buffers.len() == 0 {
            self.file_buffers
                .push(Rc::new(RefCell::new(Document::new_scratch(
                    self.doc_disp_height(),
                ))));
        } else {
            self.active_buffer = 0;
        }
    }

    pub fn next_buffer(&mut self) {
        if self.active_buffer + 1 == self.file_buffers.len() {
            self.active_buffer = 0;
        } else {
            self.active_buffer += 1;
        }

        self.initialize_display();
    }

    pub fn prev_buffer(&mut self) {
        if self.active_buffer == 0 {
            self.active_buffer = self.file_buffers.len() - 1;
        } else {
            self.active_buffer -= 1;
        }

        self.initialize_display();
    }

    pub fn write_current_buffer_to_file(&self, new_name: Option<&str>) {
        if let Some(file_name) = new_name {
            let mut out_file = File::create(file_name).unwrap();

            out_file
                .write(self.current_buffer().borrow().to_string().as_bytes())
                .unwrap();
        } else {
            let doc_bind = Rc::clone(&self.current_buffer());

            let document = doc_bind.borrow();

            let mut out_file = File::create(&document.file_name).unwrap();

            out_file.write(document.to_string().as_bytes()).unwrap();
        }
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
