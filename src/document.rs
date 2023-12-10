use std::fmt::Display;
use std::fs::File;
use std::io::Write;

#[derive(Clone)]
pub struct Line(pub Vec<usize>, pub String);

impl Line {
    pub fn new() -> Self {
        Self(Vec::new(), "".to_string())
    }

    pub fn from_str(src: String) -> Self {
        // src is assumed to be a single line string, not containing any new line characters, creation of multiple Lines is to be done outside of this function
        // so will to the insertion of indices
        Self(Vec::new(), src)
    }
}

pub struct Document {
    pub file_name: String,
    pub lines: Vec<Line>,
}

impl Document {
    pub fn new(file_name: String, content: String, width: u32) -> Self {
        let mut curr_ind: usize = 0;
        let mut lines: Vec<Line> = Vec::new();

        for line in content.lines() {
            let mut new_line = Line::from_str(line.to_string());

            if line.len() <= width as usize {
                new_line.0 = vec![curr_ind];

                curr_ind += 1;
            } else {
                let overflow = line.len() / width as usize;

                for i in 0..=overflow {
                    new_line.0.push(curr_ind + i);
                }

                curr_ind += overflow;

                curr_ind += 1;
            }

            lines.push(new_line);
        }

        Self { file_name, lines }
    }

    pub fn get_str_at_cursor(&self, cursor_row: u32) -> String {
        for line in self.lines.iter() {
            if line.0.contains(&((cursor_row - 2) as usize)) {
                return line.1.clone();
            }
        }

        panic!("Not found");
    }

    pub fn get_line_at_cursor(&self, cursor_row: u32) -> Line {
        for line in self.lines.iter() {
            if line.0.contains(&((cursor_row - 2) as usize)) {
                return line.clone();
            }
        }

        panic!("Not found");
    }

    pub fn set_line_at_cursor(&mut self, cursor_row: u32, new_line: String) {
        let mut changed_line = Line::from_str(new_line);
        changed_line.0 = self.lines[(cursor_row - 2) as usize].0.clone();
        self.lines[(cursor_row - 2) as usize] = changed_line;
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
