use crate::document::{Document, Line};
use crate::term::move_cursor_to;

pub struct Cursor {
    pub row: u32,
    pub column: u32,
    pub prev_row: u32,
    pub prev_col: u32,
}

impl Cursor {
    pub fn new(row: u32, column: u32) -> Self {
        Self {
            row,
            column,
            prev_row: 0, // Both initialized to 0 so there's no need for options
            prev_col: 0,
        }
    }

    pub fn get_row_usize(&self) -> usize {
        self.row as usize
    }

    pub fn get_column_usize(&self) -> usize {
        self.column as usize
    }

    pub fn get_position_in_line(&self, document: &Document, width: usize) -> usize {
        (document
            .get_line_at_cursor(self.row)
            .0
            .iter()
            .position(|i| *i == (self.get_row_usize() - 2))
            .unwrap()
            * width)
            + self.get_column_usize()
    }

    pub fn move_to_end_line(&mut self, document: &Document, width: usize) {
        let curr_line = document.get_line_at_cursor(self.row);

        if let Some(last_line_ind) = curr_line.0.last() {
            let len_last_row = curr_line.1.len() % width;

            self.move_to(*last_line_ind as u32, len_last_row as u32);
        }
    }

    pub fn move_to(&mut self, new_row: u32, new_col: u32) {
        self.row = new_row;
        self.column = new_col;
        self.update_pos()
    }

    pub fn move_up(&mut self) {
        self.row -= 1;
        self.update_pos();
    }
    pub fn move_left(&mut self) {
        self.column -= 1;
        self.update_pos();
    }
    pub fn move_down(&mut self) {
        self.row += 1;
        self.update_pos();
    }
    pub fn move_right(&mut self) {
        self.column += 1;
        self.update_pos();
    }

    pub fn move_to_left_border(&mut self) {
        self.column = 1;
        self.update_pos();
    }

    pub fn update_pos(&self) {
        move_cursor_to(self.column, self.row)
    }

    pub fn save_current_pos(&mut self) {
        self.prev_row = self.row;
        self.prev_col = self.column;
    }

    pub fn revert_pos(&mut self) {
        self.row = self.prev_row;
        self.column = self.prev_col;
        move_cursor_to(self.column, self.row);
    }
}
