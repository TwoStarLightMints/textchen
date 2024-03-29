use crate::editor::Editor;
use std::fmt::Display;
use std::iter::Iterator;
use std::{fs::File, io::Write};

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
            let overflow = if src.len() % editor_width == 0 {
                (src.len() / editor_width) - 1
            } else {
                src.len() / editor_width
            };

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

    pub fn rows(&self, editor_width: usize) -> Rows {
        if self.1.len() > 0 {
            let mut chars = self.1.chars().peekable();

            let mut sub_rows: Vec<_> = Vec::new();

            while let Some(_) = chars.by_ref().peek() {
                let next_row_content = chars.by_ref().take(editor_width).collect::<String>();
                sub_rows.push(next_row_content);
            }

            let mut rows = Vec::new();
            self.0
                .iter()
                .zip(sub_rows.iter())
                .map(|row| (*row.0, row.1.clone()))
                .for_each(|e| rows.push(e));

            Rows::new(Some(rows))
        } else {
            Rows::new(Some(vec![(self.0[0], String::new())]))
        }
    }
}

pub struct Rows {
    rows: Vec<(usize, String)>,
    curr_ind: usize,
    curr: Option<(usize, String)>,
}

impl Rows {
    pub fn new(opt_rows: Option<Vec<(usize, String)>>) -> Self {
        if let Some(rows) = opt_rows {
            let curr = Some(rows[0].clone());
            Self {
                rows,
                curr_ind: 0,
                curr,
            }
        } else {
            Self {
                rows: Vec::new(),
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

        if self.curr_ind + 1 < self.rows.len() {
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
                visible_rows: (0, editor_dim.doc_disp_height),
            };
        }

        let mut curr_ind: usize = 0;
        let mut lines: Vec<Line> = Vec::new();

        for line in content.lines() {
            let new_line =
                Line::from_str(line.to_string(), &mut curr_ind, editor_dim.doc_disp_width);

            lines.push(new_line);
        }

        Self {
            file_name,
            lines,
            visible_rows: (0, editor_dim.doc_disp_height),
        }
    }

    pub fn get_str_at_cursor(&self, cursor_doc_row: usize) -> &str {
        //! Returns the string content of the line which is located at the cursor's row relative to the document

        match self.get_index_at_cursor(cursor_doc_row) {
            Ok(ind) => &self.lines[ind].1,
            Err(message) => panic!("{message}"),
        }
    }

    pub fn get_line_at_cursor(&self, cursor_doc_row: usize) -> &Line {
        //! Returns the entire line which is located at the cursor's row relative to the document

        match self.get_index_at_cursor(cursor_doc_row) {
            Ok(ind) => &self.lines[ind],
            Err(message) => panic!("{message}"),
        }
    }

    pub fn get_index_at_cursor(&self, cursor_doc_row: usize) -> Result<usize, String> {
        //! Returns the index of the line within the Document's line vector which is located at the cursor's row
        //! relative to the document

        // Beginning of the search area
        let mut beg = 0;
        // End of the search area
        let mut end = self.lines.len() - 1;

        // While the beginning is not equal to the end (the search area is 0)
        while beg != end {
            // mid is the index current element being compared
            let mid = beg + ((end - beg) / 2);

            if *self.lines[mid].0.first().unwrap() > cursor_doc_row {
                // If the cursor's row in the document is less than the first row index of this element

                // Make the current element the last element in the area
                end = mid;
            } else if *self.lines[mid].0.last().unwrap() < cursor_doc_row {
                // If the cursor's row in the document is greater than the last row index of this element

                // Make the current elemenent thte last element in the area
                beg = mid;
            } else {
                // The cursor's row in the document is within the rows spanned by the current element

                // Return this element's index
                return Ok(mid);
            }
        }

        Err("Line not found with given cursor document row".to_string())
    }

    pub fn set_line_at_cursor(
        &mut self,
        cursor_doc_row: usize,
        new_str: String,
        editor_width: usize,
    ) {
        //! Sets the line's string value at cursor_row to new_line and recalculates the line's indices
        //! as well as the following lines according to the editor_width

        let line_ind = self.get_index_at_cursor(cursor_doc_row).unwrap();

        self.lines[line_ind].1 = new_str;

        self.recalculate_indices(editor_width);
    }

    pub fn append_to_line(&mut self, cursor_doc_row: usize, suffix: &str, editor_width: usize) {
        let line_ind = self.get_index_at_cursor(cursor_doc_row).unwrap();

        self.lines[line_ind].1 += suffix;

        self.recalculate_indices(editor_width);
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
        //! The number of rows is similar to getting the length of a line

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

    pub fn num_above_rows(&self, editor_width: usize, cursor_doc_row: usize) -> usize {
        self.rows(editor_width)
            .take_while(|row| row.0 != cursor_doc_row)
            .count()
    }

    pub fn remove_line_from_doc(&mut self, cursor_doc_row: usize, editor_width: usize) {
        let mut ind_to_remove = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_doc_row)) {
                break;
            }

            ind_to_remove += 1;
        }

        self.lines.remove(ind_to_remove);

        self.recalculate_indices(editor_width);
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

    pub fn add_scratch_line(&mut self) {
        //! This function is to be used to add a new line when there are no lines at all present in the document
        //! Possibly generalized in coming while
        let mut blank = Line::new();

        blank.0.push(0);

        self.lines.push(blank);
    }

    pub fn recalculate_indices(&mut self, editor_width: usize) {
        let mut ind_counter = 0;

        for i in 0..self.lines.len() {
            if self.lines[i].1.len() <= editor_width {
                self.lines[i].0 = vec![ind_counter];

                ind_counter += 1;
            } else {
                let overflow = if self.lines[i].1.len() % editor_width == 0 {
                    (self.lines[i].1.len() / editor_width) - 1
                } else {
                    self.lines[i].1.len() / editor_width
                };

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

        if self.lines.len() > 0 {
            for line in self.lines.iter() {
                if line.1.len() > 0 {
                    // If line is not empty, this guard needs to be here due to a graphical bug I encountered

                    let mut chars = line.1.chars().peekable();

                    let mut sub_rows: Vec<_> = Vec::new();

                    while let Some(_) = chars.by_ref().peek() {
                        let next_row_content =
                            chars.by_ref().take(editor_width).collect::<String>();
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
            Rows::new(Some(rows))
        } else {
            Rows::new(None)
        }
    }

    pub fn push_vis_down(&mut self) {
        //! Manipulate the visible rows of the document in such a way as to give the appearance of
        //! pushing the view down

        if self.visible_rows.1 < self.num_rows() + 1 {
            self.visible_rows.0 += 1;
            self.visible_rows.1 += 1;
        }
    }

    pub fn push_vis_up(&mut self) {
        //! Manipulate the visible rows of the document in such a way as to give the appearance of
        //! pushing the view up

        if self.visible_rows.0 > 0 {
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
