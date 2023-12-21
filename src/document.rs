use crate::editor::Editor;
use std::fmt::Display;
use std::iter::Iterator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line(pub Vec<usize>, pub String);

impl Line {
    pub fn new() -> Self {
        Self(Vec::new(), "".to_string())
    }

    pub fn from_str(src: String, ind_counter: &mut usize, editor_width: usize) -> Self {
        //! src is assumed to be a single line string, not containing any new line characters, creation of multiple Lines is to be done outside of this function
        //! so will to the insertion of indices
        let mut new = Self(Vec::new(), src.clone());

        if src.len() <= editor_width {
            new.0 = vec![*ind_counter];

            *ind_counter += 1;
        } else {
            let overflow = src.len() / editor_width;

            for i in 0..=overflow {
                new.0.push(*ind_counter + i);
            }

            *ind_counter += overflow;
            *ind_counter += 1;
        }

        new
    }

    pub fn from_existing(original: Line, editor_width: usize, cursor_row: usize) -> Line {
        //! cursor_row is provided if the "original" line has not yet received any indices
        let mut new = Self(Vec::new(), original.1.clone());

        let mut ind_counter = cursor_row;

        if let Some(index) = original.0.first() {
            ind_counter = *index;
        }

        if original.1.len() <= editor_width {
            new.0 = vec![ind_counter];
        } else {
            let overflow = original.1.len() / editor_width;

            for i in 0..=overflow {
                new.0.push(ind_counter + i);
            }
        }

        new
    }
}

pub struct Rows {
    rows: Vec<(usize, String)>,
    curr_ind: usize,
    curr: Option<(usize, String)>,
}

impl Rows {
    pub fn new(rows: Vec<(usize, String)>) -> Self {
        let curr = Some(rows[0].clone());
        if rows.len() > 0 {
            Self {
                rows,
                curr_ind: 0,
                curr,
            }
        } else {
            Self {
                rows,
                curr_ind: 0,
                curr: None,
            }
        }
    }
}

impl Iterator for Rows {
    type Item = (usize, String);

    fn next(&mut self) -> Option<Self::Item> {
        let res = match &self.curr {
            Some(item) => Some(item.clone()),
            None => None,
        };

        if self.curr_ind + 1 != self.rows.len() {
            self.curr_ind += 1;
            self.curr = Some(self.rows[self.curr_ind].clone());
        } else {
            self.curr = None;
        }

        res
    }
}

#[derive(Debug)]
pub struct Document {
    pub file_name: String,
    pub lines: Vec<Line>,
    pub visible_rows: (usize, usize),
}

impl Document {
    pub fn new(file_name: String, content: String, editor_dim: &Editor) -> Self {
        if content.len() == 0 {
            let mut line = Line::new();

            line.0.push(0);

            return Self {
                file_name,
                lines: vec![line],
                visible_rows: (0, editor_dim.editor_height),
            };
        }

        let mut curr_ind: usize = 0;
        let mut lines: Vec<Line> = Vec::new();

        for line in content.lines() {
            let new_line = Line::from_str(line.to_string(), &mut curr_ind, editor_dim.editor_width);

            lines.push(new_line);
        }

        Self {
            file_name,
            lines,
            visible_rows: (0, editor_dim.editor_height),
        }
    }

    fn find_line_from_index(&self, ind: usize) -> Line {
        for line in self.lines.iter() {
            if line.0.contains(&ind) {
                return line.clone();
            }
        }

        panic!("Not found")
    }

    pub fn get_str_at_cursor(&self, cursor_doc_row: usize) -> String {
        //! Auto offsets cursor_row value by the distance from the top of the terminal to the actual start of the editor
        self.find_line_from_index(cursor_doc_row).1.clone()
    }

    pub fn get_line_at_cursor(&self, cursor_doc_row: usize) -> Line {
        //! Auto offsets cursor_row value by the distance from the top of the terminal to the actual start of the editor
        self.find_line_from_index(cursor_doc_row)
    }

    pub fn set_line_at_cursor(
        &mut self,
        cursor_doc_row: usize,
        new_line: String,
        editor_width: usize,
    ) {
        //! Sets the line's string value at cursor_row to new_line and recalculates the line's indices
        //! as well as the following lines according to the editor_width
        let mut dest = self.get_line_at_cursor(cursor_doc_row); // The line to be re-set

        dest.1 = new_line;

        let changed_line = Line::from_existing(dest, editor_width, cursor_doc_row);

        let mut ind_to_change = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_doc_row)) {
                break;
            }

            ind_to_change += 1;
        }

        self.lines[ind_to_change] = changed_line.clone();

        if ind_to_change != self.lines.len() - 1 {
            // If the changed index is not the end of the lines vector
            if changed_line.0[changed_line.0.len() - 1] + 1 != self.lines[ind_to_change + 1].0[0] {
                // If the last index of the changed line incremented by 1 is not equal to the first index of the line after it, recalculate the indices
                // the first index should equal the previous line's last index decremented by 1 as it was directly copied from the original Line
                if changed_line.0[changed_line.0.len() - 1] > self.lines[ind_to_change + 1].0[0] {
                    // If the last index of the changed line is greater than the next Line's first index
                    let difference = changed_line.0[changed_line.0.len() - 1]
                        - (self.lines[ind_to_change + 1].0[0] + 1);

                    for i in (ind_to_change + 1)..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l + difference).collect();
                    }
                } else if changed_line.0[changed_line.0.len() - 1]
                    < self.lines[ind_to_change + 1].0[0]
                {
                    let difference = self.lines[ind_to_change + 1].0[0]
                        - (changed_line.0[changed_line.0.len() - 1] + 1);

                    for i in (ind_to_change + 1)..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l - difference).collect();
                    }
                } else {
                    for i in (ind_to_change + 1)..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l + 1).collect();
                    }
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|e| e.1.clone())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_number_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn num_rows(&self) -> usize {
        //! Get the number of rows in the current document
        //!
        //! Every document is a collection of Lines, and a Line is a collection of rows and a string.
        //! So, the number of rows will be the total number of rows that the document spans in the
        //! editor.
        // Bare in mind, getting the indices allow is 0 indexed, so add 1 to get real number

        match self.lines.last() {
            Some(line) => {
                if line.0.len() > 1 {
                    line.0[line.0.len() - 1] + 1
                } else {
                    line.0[0] + 1
                }
            }
            None => 0,
        }
    }

    pub fn remove_index_from_line(&mut self, cursor_row: usize) {
        for line in self.lines.iter_mut() {
            if line.0.contains(&(cursor_row - 2)) {
                line.0.remove(cursor_row - 2);
                break;
            }
        }
    }

    pub fn remove_line_from_doc(&mut self, cursor_doc_row: usize) {
        let mut ind_to_remove = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_doc_row)) {
                break;
            }

            ind_to_remove += 1;
        }

        let line_removed = self.lines.remove(ind_to_remove);

        if ind_to_remove != self.lines.len() {
            // If the changed index is not the end of the lines vector
            if line_removed.0[line_removed.0.len() - 1] + 1 != self.lines[ind_to_remove].0[0] {
                // If the last index of the changed line incremented by 1 is not equal to the first index of the line after it, recalculate the indices
                // the first index should equal the previous line's last index decremented by 1 as it was directly copied from the original Line
                if line_removed.0[line_removed.0.len() - 1] > self.lines[ind_to_remove + 1].0[0] {
                    // If the last index of the changed line is greater than the next Line's first index
                    let difference = line_removed.0[line_removed.0.len() - 1]
                        - (self.lines[ind_to_remove + 1].0[0] + 1);

                    for i in ind_to_remove..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l + difference).collect();
                    }
                } else if line_removed.0[line_removed.0.len() - 1]
                    < self.lines[ind_to_remove + 1].0[0]
                {
                    let difference = self.lines[ind_to_remove + 1].0[0]
                        - (line_removed.0[line_removed.0.len() - 1] + 1);

                    for i in ind_to_remove..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l - difference).collect();
                    }
                } else {
                    for i in ind_to_remove..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l + 1).collect();
                    }
                }
            }
        }
    }

    pub fn add_line_at_row(&mut self, new_line: Line, cursor_doc_row: usize) {
        let mut insert_ind = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_doc_row)) {
                break;
            }

            insert_ind += 1;
        }

        self.lines.insert(insert_ind, new_line);

        // insert_ind will now be the position that the new line was inserted, so to iterate over the elements after it, add 1
        for i in (insert_ind + 1)..self.lines.len() {
            self.lines[i].0 = self.lines[i].0.iter().map(|inds| inds + 1).collect();
        }
    }

    pub fn recalculate_indices(&mut self, editor_width: usize) {
        let mut ind_counter = 0;

        for i in 0..self.lines.len() {
            if self.lines[i].1.len() <= editor_width {
                self.lines[i].0 = vec![ind_counter];

                ind_counter += 1;
            } else {
                let overflow = self.lines[i].1.len() / editor_width;

                let mut new_inds = Vec::new();

                for j in 0..=overflow {
                    new_inds.push(ind_counter + j);
                }

                self.lines[i].0 = new_inds;

                ind_counter += overflow;
                ind_counter += 1;
            }
        }
    }

    pub fn rows(&self, editor_width: usize) -> Rows {
        let mut rows: Vec<_> = Vec::new();

        for line in self.lines.iter() {
            if line.1.len() > 0 {
                // If line is not empty, this guard needs to be here due to a graphical bug I encountered

                let mut chars = line.1.chars().peekable();

                let mut sub_rows: Vec<_> = Vec::new();

                while let Some(_) = chars.by_ref().peek() {
                    let next_row_content = chars.by_ref().take(editor_width).collect::<String>();
                    sub_rows.push(next_row_content);
                }

                line.0
                    .iter()
                    .zip(sub_rows.iter())
                    .map(|e| (*e.0, e.1.clone()))
                    .for_each(|e| rows.push(e));
            } else {
                // If line is empty

                rows.push((line.0[0], "".to_string()));
            }
        }

        Rows::new(rows)
    }

    pub fn push_vis_down(&mut self) {
        //! Do not worry about "showing rows that aren't there" the reset_editor_view will do the rest for you

        self.visible_rows.0 += 1;
        if self.visible_rows.1 < self.num_rows() {
            self.visible_rows.1 += 1;
        }
    }

    pub fn push_vis_up(&mut self, editor_dim: &Editor) {
        //! Do not worry about "showing rows that aren't there" the reset_editor_view will do the rest for you

        if self.visible_rows.1 - self.visible_rows.0 != editor_dim.editor_height {
            self.visible_rows.0 -= 1;
        } else {
            self.visible_rows.0 -= 1;
            self.visible_rows.1 -= 1;
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.lines
                .iter()
                .map(|e| e.1.clone())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
