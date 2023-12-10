use std::fmt::Display;

struct Line(Vec<usize>, String);

pub struct Document {
    pub file_name: String,
    pub lines: Vec<Line>,
}

impl Document {
    pub fn new(file_name: String, content: String, width: u32) -> Self {
        let mut curr_ind: usize = 0;
        let mut lines: Vec<Line> = Vec::new();

        for line in content.lines() {
            let mut new_line = Line(Vec::new(), line.to_string());

            if line.len() <= width as usize {
                new_line.0 = vec![curr_ind];

                curr_ind += 1;
            } else {
                let overflow = line.len() / width as usize;
                let mut inds: Vec<_> = Vec::new();

                for i in 0..overflow {
                    inds.push(curr_ind + i);
                }

                curr_ind += overflow;
            }
        }

        Self { file_name, lines }
    }

    pub fn get_line_from_cursor_pos(&self, cursor_row: u32) -> String {
        for line in self.lines {
            if line.0.contains(cursor_row) {
                line.1.clone()
            }
        }
        self.lines[(cursor_row - 2) as usize].clone()
    }

    pub fn set_line_from_cursor_pos(&mut self, cursor_row: u32, new_line: String) {
        self.lines[(cursor_row - 2) as usize] = new_line;
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}
