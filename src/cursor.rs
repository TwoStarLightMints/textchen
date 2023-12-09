use crate::term::move_cursor_to;

pub struct Cursor {
    pub row: u32,
    pub column: u32,
}

impl Cursor {
    pub fn new(row: u32, column: u32) -> Self {
        Self { row, column }
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
}
