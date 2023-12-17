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

    pub fn get_position_in_line(
        &self,
        document: &Document,
        editor_left_edge: usize,
        editor_width: usize,
    ) -> usize {
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
            * editor_width)
            + self.get_column_in_editor(editor_left_edge)
    }

    pub fn move_to_end_line(
        &mut self,
        document: &Document,
        editor_left_edge: usize,
        editor_width: usize,
        editor_top: usize,
    ) {
        let curr_line = document.get_line_at_cursor(self.row);
        let cursor_pos = self.get_position_in_line(document, editor_left_edge, editor_width);

        // The cursor's position mod the editor width is the distance from the left edge, adding the left
        // edge to the result gets the distance from the terminal's left edge
        // (curr_line.1.len() % editor_width) + editor_left_edge;

        // Last row index of the line, index from the top of the EDITOR not the terminal, so add editor_top as offset
        // curr_line.0[curr_line.0.len() - 1] + editor_top;

        self.move_to(
            curr_line.0[curr_line.0.len() - 1] + editor_top,
            (curr_line.1.len() % editor_width) + editor_left_edge,
        );
    }

    pub fn move_to_pos_in_line(
        &mut self,
        document: &Document,
        editor_left_edge: usize,
        editor_width: usize,
        editor_top: usize,
        new_pos: usize,
    ) {
        //! new_pos - An index value of where in the line the cursor should visually appear
        //! Assumed to be the current line the cursor is inside of
        let curr_line = document.get_line_at_cursor(self.row);

        // Based on the index given, these will be the coordinates to move to within the line
        let row_index = new_pos / editor_width;
        // Add editor_left_edge to account for the blank space between the edge and the terminal left edge
        let column = (new_pos % editor_width) + editor_left_edge;

        // row_index simply gives the row within the line where the actual overall index is given, and because the rows are only in realtion to the beginning of
        // the document, add 2 to get the actual position in the terminal to place the cursor visually
        let row = curr_line.0[row_index] + 2;

        if new_pos <= curr_line.1.len() {
            // If the new position is within the line's string content

            // Move to that position
            self.move_to(row, column);
        } else {
            // If the new position is outside the bounds of the line
            self.move_to_end_line(&document, editor_left_edge, editor_width, editor_top);
        }
    }

    pub fn move_to_start_line(&mut self, document: &Document, editor_left_edge: usize) {
        self.move_to_editor_left(editor_left_edge);

        self.move_to(document.get_line_at_cursor(self.row).0[0], self.column);
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

    pub fn move_to_editor_left(&mut self, editor_left_edge: usize) {
        self.move_to(self.row, editor_left_edge);
    }

    pub fn move_to_editor_right(&mut self, editor_right_edge: usize) {
        self.move_to(self.row, editor_right_edge);
    }

    fn update_pos(&self) {
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

    pub fn get_column_in_editor(&self, editor_left_edge: usize) -> usize {
        //! Used to get column with respect to the editor's left edge (take away the amount that the left edge adds)
        self.column - editor_left_edge
    }
}
