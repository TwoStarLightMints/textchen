use crate::document::Document;
use crate::editor::{reset_editor_view, Editor};
use crate::term::move_cursor_to;
use std::fs::File;
use std::io::Write;

pub struct Cursor {
    pub doc_row: usize,
    pub doc_column: usize,
    pub row: usize,
    pub column: usize,
    pub prev_row: Vec<usize>,
    pub prev_col: Vec<usize>,
}

impl Cursor {
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            doc_row: 0,
            doc_column: 0,
            row,
            column,
            prev_row: Vec::new(), // Both initialized to 0 so there's no need for options
            prev_col: Vec::new(),
        }
    }

    pub fn get_position_in_line(&self, document: &Document, editor_dim: &Editor) -> usize {
        // document.get_line_at_cursor(cursor.row).0.iter().position(|i| *i == cursor.row - 2) * editor_right : skip x amount of lines, refer to this line as skip_amount
        // skip_amount + cursor.column

        // self.doc_row : The row of the cursor in relation to the document, will be equal to an index of one of the Lines within the document
        // self.doc_column : The column of the cursor in relation to the document, will be within the string in some way

        // document.get_line_at_cursor(self.doc_row).0.iter.position(|i| *i == self.doc_row).unwrap() : Returns the row number within the line that the cursor lies
        // /\ * editor_width : The above times the editor's width will give the amount of spaces to skip given the row of the cursor in relation to the document
        // /\ + doc_column : This will be the position of the cursor within the line

        (document
            .get_line_at_cursor(self.doc_row)
            .0
            .iter()
            .position(|i| *i == self.doc_row)
            .unwrap()
            * editor_dim.editor_width)
            + self.doc_column
    }

    pub fn move_to_end_line(&mut self, document: &mut Document, editor_dim: &Editor) {
        //! This method will only be called when the cursor is within a given line
        //! This will move both the cursor's visual position *AND* the doc position

        let curr_line = document.get_line_at_cursor(self.doc_row);

        let mut curr_line_final_row = *curr_line.0.last().unwrap();

        // The cursor's position mod the editor width is the distance from the left edge, adding the left
        // edge to the result gets the distance from the terminal's left edge
        // (curr_line.1.len() % editor_width) + editor_left_edge;

        // Last row index of the line, index from the top of the EDITOR not the terminal, so add editor_top as offset
        // curr_line.0[curr_line.0.len() - 1] + editor_top;

        if self.get_position_in_line(document, editor_dim) != curr_line.1.len() {
            // If the cursor is not already at the end of the line

            if (document.visible_rows.0..(document.visible_rows.1 - 1))
                .contains(&curr_line_final_row)
            {
                // If the last row of the current line is within the visible rows exclusive of the document and the last visible row is strictly
                // less than the last row of the current line

                self.move_to(
                    (curr_line_final_row - document.visible_rows.0) + editor_dim.editor_top,
                    (curr_line.1.len() % editor_dim.editor_width) + editor_dim.editor_left_edge,
                );

                self.move_doc_to(
                    curr_line_final_row,
                    curr_line.1.len() % editor_dim.editor_width,
                );
            } else {
                // If the last row of the current line is within the visible rows inclusive of the document and the last row of the current line
                // is greater than or equal to the last visible row

                self.move_to(
                    editor_dim.editor_height,
                    curr_line.1.len() % editor_dim.editor_width + editor_dim.editor_left_edge,
                );

                self.move_doc_to(
                    curr_line_final_row,
                    curr_line.1.len() % editor_dim.editor_width,
                );

                let current_last_vis_row = document.visible_rows.1;

                while current_last_vis_row > curr_line_final_row {
                    document.push_vis_down();

                    curr_line_final_row += 1;
                }

                reset_editor_view(&document, editor_dim, self);
            }
        }
    }

    pub fn move_to_pos_in_line(
        &mut self,
        document: &mut Document,
        editor_dim: &Editor,
        new_pos: usize,
    ) {
        //! new_pos - An index value of where in the line the cursor should visually appear
        //! Assumed to be the current line the cursor is inside of
        let curr_line = document.get_line_at_cursor(self.row);

        // Based on the index given, these will be the coordinates to move to within the line
        let row_index = new_pos / editor_dim.editor_width;
        // Add editor_left_edge to account for the blank space between the edge and the terminal left edge
        let column = (new_pos % editor_dim.editor_width) + editor_dim.editor_left_edge;

        // row_index simply gives the row within the line where the actual overall index is given, and because the rows are only in realtion to the beginning of
        // the document, add 2 to get the actual position in the terminal to place the cursor visually
        let row = curr_line.0[row_index] + 2;

        if new_pos <= curr_line.1.len() {
            // If the new position is within the line's string content

            // Move to that position
            self.move_to(row, column);
        } else {
            // If the new position is outside the bounds of the line
            self.move_to_end_line(document, editor_dim);
        }
    }

    pub fn move_to_start_line(
        &mut self,
        document: &mut Document,
        editor_dim: &Editor,
        editor_home_row: usize,
    ) {
        let curr_line = document.get_line_at_cursor(self.doc_row);
        let cursor_pos = self.get_position_in_line(&document, editor_dim);

        if cursor_pos != 0 {
            if ((document.visible_rows.0 + 2)..document.visible_rows.1).contains(&curr_line.0[0]) {
                self.move_to_editor_left(editor_dim.editor_left_edge);
                self.move_doc_to_editor_left();

                self.move_to(
                    self.row - (cursor_pos / editor_dim.editor_width),
                    self.column,
                );

                self.move_doc_to(curr_line.0[0], self.doc_column);
            } else {
                self.move_to_editor_left(editor_dim.editor_left_edge);
                self.move_doc_to_editor_left();

                let current_first_vis_row = document.visible_rows.0;

                let mut curr_line_first_row = curr_line.0[0];

                while current_first_vis_row > curr_line_first_row {
                    document.push_vis_up(editor_dim);

                    curr_line_first_row += 1;
                }

                self.move_to(editor_home_row, self.column);

                reset_editor_view(document, editor_dim, self);
            }
        }
    }

    pub fn move_to(&mut self, new_row: usize, new_col: usize) {
        self.row = new_row;
        self.column = new_col;

        self.update_pos()
    }

    pub fn move_doc_to(&mut self, new_doc_row: usize, new_doc_col: usize) {
        self.doc_row = new_doc_row;
        self.doc_column = new_doc_col;
    }

    pub fn move_up(&mut self) {
        //! Used to move within the editor visually
        self.row -= 1;
        self.update_pos();
    }
    pub fn move_left(&mut self) {
        //! Used to move within the editor visually
        self.column -= 1;
        self.update_pos();
    }
    pub fn move_down(&mut self) {
        //! Used to move within the editor visually
        self.row += 1;
        self.update_pos();
    }
    pub fn move_right(&mut self) {
        //! Used to move within the editor visually
        self.column += 1;
        self.update_pos();
    }

    pub fn move_doc_up(&mut self) {
        //! Used to move within the document for editing
        self.doc_row -= 1;
    }

    pub fn move_doc_left(&mut self) {
        //! Used to move within the document for editing
        self.doc_column -= 1;
    }

    pub fn move_doc_down(&mut self) {
        //! Used to move within the document for editing
        self.doc_row += 1;
    }

    pub fn move_doc_right(&mut self) {
        //! Used to move within the document for editing
        self.doc_column += 1;
    }

    pub fn move_to_editor_left(&mut self, editor_left_edge: usize) {
        self.move_to(self.row, editor_left_edge);
    }

    pub fn move_doc_to_editor_left(&mut self) {
        //! This function is *ONLY* meant to be used to reset the cursor's doc_column value to zero and provide documentation through the name of the
        // function

        self.doc_column = 0;
    }

    pub fn move_to_editor_right(&mut self, editor_right_edge: usize) {
        self.move_to(self.row, editor_right_edge);
    }

    pub fn move_doc_to_editor_width(&mut self, editor_width: usize) {
        //! For the doc_column field, the "editor right" would be the *WIDTH* of the editor, not the right edge

        self.doc_column = editor_width;
    }

    fn update_pos(&self) {
        move_cursor_to(self.row, self.column)
    }

    pub fn save_current_pos(&mut self) {
        self.prev_row.push(self.row);
        self.prev_col.push(self.column);
    }

    pub fn revert_pos(&mut self) {
        let old_row = self.prev_row.pop().unwrap();
        let old_col = self.prev_col.pop().unwrap();

        self.move_to(old_row, old_col);
    }

    pub fn get_column_in_editor(&self, editor_left_edge: usize) -> usize {
        //! Used to get column with respect to the editor's left edge (take away the amount that the left edge adds)
        self.column - editor_left_edge
    }
}
