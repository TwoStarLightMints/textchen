use std::fmt::Display;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line(pub Vec<usize>, pub String);

impl Line {
    pub fn new() -> Self {
        Self(Vec::new(), "".to_string())
    }

    pub fn from_str(src: String, ind_counter: &mut usize, width: usize) -> Self {
        // src is assumed to be a single line string, not containing any new line characters, creation of multiple Lines is to be done outside of this function
        // so will to the insertion of indices
        let mut new = Self(Vec::new(), src.clone());

        if src.len() <= width as usize {
            new.0 = vec![*ind_counter];

            *ind_counter += 1;
        } else {
            let overflow = src.len() / width;

            for i in 0..=overflow {
                new.0.push(*ind_counter + i);
            }

            *ind_counter += overflow;

            *ind_counter += 1;
        }

        new
    }

    pub fn from_existing(original: Line, width: usize) -> Line {
        let mut new = Self(Vec::new(), original.1.clone());

        let ind_counter = *original.0.first().unwrap();

        if original.1.len() <= width {
            new.0 = vec![ind_counter];
        } else {
            let overflow = original.1.len() / width;

            for i in 0..=overflow {
                new.0.push(ind_counter + i);
            }
        }

        new
    }
}

pub struct Document {
    pub file_name: String,
    pub lines: Vec<Line>,
}

impl Document {
    pub fn new(file_name: String, content: String, width: usize) -> Self {
        let mut curr_ind: usize = 0;
        let mut lines: Vec<Line> = Vec::new();

        for line in content.lines() {
            let new_line = Line::from_str(line.to_string(), &mut curr_ind, width);

            lines.push(new_line);
        }

        Self { file_name, lines }
    }

    fn find_line_from_index(&self, ind: usize) -> Line {
        for line in self.lines.iter() {
            if line.0.contains(&ind) {
                return line.clone();
            }
        }

        panic!("Not found")
    }

    pub fn get_str_at_cursor(&self, cursor_row: usize) -> String {
        self.find_line_from_index(cursor_row - 2).1.clone()
    }

    pub fn get_line_at_cursor(&self, cursor_row: usize) -> Line {
        self.find_line_from_index(cursor_row - 2)
    }

    pub fn set_line_at_cursor(&mut self, cursor_row: usize, new_line: String, width: usize) {
        let mut dest = self.get_line_at_cursor(cursor_row); // The line to be re-set
        dest.1 = new_line;

        let changed_line = Line::from_existing(dest, width);

        let mut ind_to_change = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_row - 2)) {
                break;
            }

            ind_to_change += 1;
        }

        self.lines[ind_to_change] = changed_line.clone();

        if ind_to_change != self.lines.len() - 1 {
            // If the changed index is not the end of the lines vector
            if !(changed_line.0[changed_line.0.len() - 1] + 1 == self.lines[ind_to_change + 1].0[0])
            {
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

    pub fn remove_line_from_doc(&mut self, cursor_row: usize) {
        let mut ind_to_remove = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_row - 2)) {
                break;
            }

            ind_to_remove += 1;
        }

        let line_removed = self.lines.remove(ind_to_remove);

        if ind_to_remove != self.lines.len() - 1 {
            // If the changed index is not the end of the lines vector
            if !(line_removed.0[line_removed.0.len() - 1] + 1 == self.lines[ind_to_remove + 1].0[0])
            {
                // If the last index of the changed line incremented by 1 is not equal to the first index of the line after it, recalculate the indices
                // the first index should equal the previous line's last index decremented by 1 as it was directly copied from the original Line
                if line_removed.0[line_removed.0.len() - 1] > self.lines[ind_to_remove + 1].0[0] {
                    // If the last index of the changed line is greater than the next Line's first index
                    let difference = line_removed.0[line_removed.0.len() - 1]
                        - (self.lines[ind_to_remove + 1].0[0] + 1);

                    for i in (ind_to_remove + 1)..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l + difference).collect();
                    }
                } else if line_removed.0[line_removed.0.len() - 1]
                    < self.lines[ind_to_remove + 1].0[0]
                {
                    let difference = self.lines[ind_to_remove + 1].0[0]
                        - (line_removed.0[line_removed.0.len() - 1] + 1);

                    for i in (ind_to_remove + 1)..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l - difference).collect();
                    }
                } else {
                    for i in (ind_to_remove + 1)..self.lines.len() {
                        self.lines[i].0 = self.lines[i].0.iter().map(|l| *l + 1).collect();
                    }
                }
            }
        }
    }

    pub fn add_line_at_row(&mut self, new_line: Line, cursor_row: usize) {
        let mut insert_ind = 0;

        for line in self.lines.iter() {
            if line.0.contains(&(cursor_row - 2)) {
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
