use crate::editor::Editor;
use std::rc::Rc;

pub struct Cursor {
    pub doc_row: usize,
    pub doc_column: usize,
    pub row: usize,
    pub column: usize,
    pub prev_row: Vec<usize>,
    pub prev_col: Vec<usize>,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            doc_row: 0,
            doc_column: 0,
            row: 0,
            column: 0,
            prev_row: Vec::new(),
            prev_col: Vec::new(),
        }
    }

    pub fn new_at_pos(row: usize, column: usize) -> Self {
        Self {
            doc_row: 0,
            doc_column: 0,
            row,
            column,
            prev_row: Vec::new(),
            prev_col: Vec::new(),
        }
    }

    // =================== Cursor Movement Functions ===================
    pub fn move_to(&mut self, new_row: usize, new_col: usize) -> String {
        self.row = new_row;
        self.column = new_col;

        self.update_pos()
    }

    pub fn move_doc_to(&mut self, new_doc_row: usize, new_doc_col: usize) {
        self.doc_row = new_doc_row;
        self.doc_column = new_doc_col;
    }

    // ------------------- Cursor Visual Movement -------------------
    pub fn move_vis_up(&mut self) -> String {
        //! Used to move within the editor visually
        self.row -= 1;
        self.update_pos()
    }
    pub fn move_vis_left(&mut self) -> String {
        //! Used to move within the editor visually
        self.column -= 1;
        self.update_pos()
    }
    pub fn move_vis_down(&mut self) -> String {
        //! Used to move within the editor visually
        self.row += 1;
        self.update_pos()
    }
    pub fn move_vis_right(&mut self) -> String {
        //! Used to move within the editor visually
        self.column += 1;
        self.update_pos()
    }

    // ------------------- Cursor Movement Within Document -------------------
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

    // ------------------- Cursor Movement Visual And Within Document -------------------
    pub fn move_up(&mut self) -> String {
        self.row -= 1;
        self.doc_row -= 1;
        self.update_pos()
    }
    pub fn move_left(&mut self) -> String {
        self.column -= 1;
        self.doc_column -= 1;
        self.update_pos()
    }
    pub fn move_down(&mut self) -> String {
        self.row += 1;
        self.doc_row += 1;
        self.update_pos()
    }
    pub fn move_right(&mut self) -> String {
        self.column += 1;
        self.doc_column += 1;
        self.update_pos()
    }

    // ------------------- Cursor Movement To Previous -------------------
    pub fn revert_pos(&mut self) -> String {
        //! Pops the last saved row and column from the prev_row and prev_col stacks
        //! and moves the cursor to that saved position

        let old_row = self.prev_row.pop().unwrap();
        let old_col = self.prev_col.pop().unwrap();

        self.move_to(old_row, old_col)
    }

    // ------------------- Cursor Movement Related To Lines -------------------
    pub fn move_to_end_line(&mut self, editor: &Editor) -> String {
        //! This method will only be called when the cursor is within a given line
        //! This will move both the cursor's visual position *AND* the doc position

        let document = Rc::clone(&editor.current_buffer());
        let binding = document.borrow();

        let curr_line = binding.get_line_at_cursor(self.doc_row);

        let mut curr_line_final_row = *curr_line.0.last().unwrap();

        let mut move_str = String::new();

        // The cursor's position mod the editor width is the distance from the left edge, adding the left
        // edge to the result gets the distance from the terminal's left edge
        // (curr_line.1.len() % editor_width) + editor_left_edge;

        // Last row index of the line, index from the top of the EDITOR not the terminal, so add editor_top as offset
        // curr_line.0[curr_line.0.len() - 1] + editor_top;

        if self.get_position_in_line(editor) != curr_line.1.len() {
            // If the cursor is not already at the end of the line

            if (document.borrow().visible_rows.0..(document.borrow().visible_rows.1 - 1))
                .contains(&curr_line_final_row)
            {
                // If the last row of the current line is within the visible rows exclusive of the document and the last visible row is strictly
                // less than the last row of the current line

                move_str += self
                    .move_to(
                        (curr_line_final_row - document.borrow().visible_rows.0)
                            + editor.doc_disp_home_row(),
                        (curr_line.1.len() % editor.doc_disp_width()) + editor.doc_disp_left_edge(),
                    )
                    .as_str();

                self.move_doc_to(
                    curr_line_final_row,
                    curr_line.1.len() % editor.doc_disp_width(),
                );
            } else {
                // If the last row of the current line is within the visible rows inclusive of the document and the last row of the current line
                // is greater than or equal to the last visible row

                move_str += self
                    .move_to(
                        editor.doc_disp_height(),
                        curr_line.1.len() % editor.doc_disp_width() + editor.doc_disp_left_edge(),
                    )
                    .as_str();

                self.move_doc_to(
                    curr_line_final_row,
                    curr_line.1.len() % editor.doc_disp_width(),
                );

                let current_last_vis_row = document.borrow().visible_rows.1;

                while current_last_vis_row > curr_line_final_row {
                    document.borrow_mut().push_vis_down();

                    curr_line_final_row += 1;
                }
            }
        }

        move_str
    }

    pub fn move_to_start_line(&mut self, editor: &Editor) -> String {
        let document = Rc::clone(&editor.current_buffer());
        let binding = document.borrow();
        let curr_line = binding.get_line_at_cursor(self.doc_row);
        let cursor_pos = self.get_position_in_line(editor);

        let mut move_str = String::new();

        if cursor_pos / editor.doc_disp_width() != 0 {
            // If the cursor is not in the first row of the line

            if document.borrow().visible_rows.0 < curr_line.0[0]
                || document.borrow().visible_rows.0 == 0
            {
                // If the current line's first index is strictly less than the first visible row, or the first visible row is the
                // first row of the document, i.e. the first row of the current line is visible or the first row of the document is visible

                move_str += self
                    .move_to_editor_left(editor.doc_disp_left_edge())
                    .as_str();

                move_str += self
                    .move_to(
                        self.row - (cursor_pos / editor.doc_disp_width()),
                        self.column,
                    )
                    .as_str();

                self.move_doc_to(curr_line.0[0], 0);
            } else {
                move_str += self
                    .move_to_editor_left(editor.doc_disp_left_edge())
                    .as_str();

                self.move_doc_to_editor_left();

                let current_first_vis_row = document.borrow().visible_rows.0;

                let mut curr_line_first_row = curr_line.0[0];

                while current_first_vis_row > curr_line_first_row {
                    document.borrow_mut().push_vis_up(editor.doc_disp_height());

                    curr_line_first_row += 1;
                }

                move_str += self
                    .move_to(editor.doc_disp_home_row(), self.column)
                    .as_str();

                editor.reset_editor_view();
            }
        } else {
            self.move_doc_to_editor_left();

            move_str += self
                .move_to_editor_left(editor.doc_disp_left_edge())
                .as_str();
        }

        move_str
    }

    pub fn move_to_pos(&mut self, new_pos: usize, editor: &Editor) -> String {
        //! visible_range : Expected to be the last visible row minus the first

        // If the new position is 0, just set it to 0, otherwise, the new column will be equal
        // to the new position mod the editor's width plus 1 (to allow for the cursor to hang
        // on the right side) plus the editor's left edge to start the counting from within
        // the editor's window, and then add the "row" to give the necessary bump in movement
        let document = editor.current_buffer();

        let new_column = if new_pos == 0 {
            editor.doc_disp_left_edge()
        } else {
            new_pos % (editor.doc_disp_width() + 1)
                + editor.doc_disp_left_edge()
                + (new_pos / (editor.doc_disp_width() + 1))
        };

        // For calculating the cursor's position within the document, use the current line to
        // grab the first row index in the current line, using this as the starting point
        // add the calculated row to that index to get the new document row, then use the
        // above calculated new column value minus the editor's left edge
        self.move_doc_to(
            document.borrow().get_line_at_cursor(self.doc_row).0[0]
                + (new_pos / (editor.doc_disp_width() + 1)),
            new_column - editor.doc_disp_left_edge(),
        );

        let new_row =
            (self.doc_row - document.borrow().visible_rows.0) + editor.doc_disp_home_row();

        let safe_row = if new_row >= editor.doc_disp_bottom() - 1 {
            editor.doc_disp_bottom() - 1
        } else {
            new_row
        };

        self.move_to(safe_row, new_column)
    }

    // ------------------- Cursor Movement Related Within Editor -------------------
    pub fn move_to_editor_left(&mut self, editor_left_edge: usize) -> String {
        self.move_to(self.row, editor_left_edge)
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

    // =================== Cursor Position Retrieval Functions ===================
    pub fn get_position_in_line(&self, editor: &Editor) -> usize {
        // document.get_line_at_cursor(cursor.row).0.iter().position(|i| *i == cursor.row - 2) * editor_right : skip x amount of lines, refer to this line as skip_amount
        // skip_amount + cursor.column

        // self.doc_row : The row of the cursor in relation to the document, will be equal to an index of one of the Lines within the document
        // self.doc_column : The column of the cursor in relation to the document, will be within the string in some way

        // document.get_line_at_cursor(self.doc_row).0.iter.position(|i| *i == self.doc_row).unwrap() : Returns the row number within the line that the cursor lies
        // /\ * editor_width : The above times the editor's width will give the amount of spaces to skip given the row of the cursor in relation to the document
        // /\ + doc_column : This will be the position of the cursor within the line

        ((self.doc_row
            - editor
                .current_buffer()
                .borrow()
                .get_line_at_cursor(self.doc_row)
                .0[0])
            * editor.doc_disp_width())
            + self.doc_column
    }

    pub fn get_column_in_editor(&self, editor_left_edge: usize) -> usize {
        //! Used to get column with respect to the editor's left edge (take away the amount that the left edge adds)
        self.column - editor_left_edge
    }

    // =================== Utility Functions ===================
    pub fn save_current_pos(&mut self) {
        self.prev_row.push(self.row);
        self.prev_col.push(self.column);
    }

    fn update_pos(&self) -> String {
        //! Used to set the cursor's visual position to the stored row and column

        format!("\u{001b}[{};{}H", self.row, self.column)
    }
}
