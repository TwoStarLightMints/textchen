use crate::document::Document;
use crate::term::move_cursor_to;

pub struct Cursor {
    pub row: usize,
    pub column: usize,
    pub prev_row: usize,
    pub prev_col: usize,
}

impl Cursor {
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            prev_row: 0, // Both initialized to 0 so there's no need for options
            prev_col: 0,
        }
    }

    pub fn get_position_in_line(&self, document: &Document, width: usize) -> usize {
        // document.get_str_at_cursor(cursor.row).len() as u32 / editor_right : takes into account whole string
        // cursor.row - 2 : doesn't take actual cursor position into full account
        // cursor.column : only gives where the cursor is inside of the line

        // document.get_line_at_cursor(cursor.row).0.iter().find(|i| *i == cursor.row - 2) * editor_right : skip x amount of lines, refer to this line as skip_amount
        // skip_amount + cursor.column

        (document
            .get_line_at_cursor(self.row)
            .0
            .iter()
            .position(|i| *i == (self.row - 2))
            .unwrap()
            * width)
            + self.column
    }

    pub fn move_to_end_line(&mut self, document: &Document, width: usize) {
        let curr_line = document.get_line_at_cursor(self.row);

        if let Some(last_line_ind) = curr_line.0.last() {
            if curr_line.1.len() % width == 0 && curr_line.1.len() != 0 {
                self.move_to(*last_line_ind + 2, width);
            } else {
                let len_last_row = curr_line.1.len() % width;

                self.move_to(*last_line_ind + 2, len_last_row + 1);
            }
        }
    }

    pub fn move_to(&mut self, new_row: usize, new_col: usize) {
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
