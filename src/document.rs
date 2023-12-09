use std::fmt::Display;

struct Line(Vec<usize>, String);

// pub struct Document {
//     pub file_name: String,
//     pub lines: Vec<String>,
// }

// impl Document {
//     pub fn new(file_name: String, content: String) -> Self {
//         Self {
//             file_name,
//             lines: content.lines().map(|e| e.to_string()).collect(),
//         }
//     }

//     pub fn get_line_from_cursor_pos(&self, cursor_row: u32) -> String {
//         self.lines[(cursor_row - 2) as usize].clone()
//     }

//     pub fn set_line_from_cursor_pos(&mut self, cursor_row: u32, new_line: String) {
//         self.lines[(cursor_row - 2) as usize] = new_line;
//     }

//     pub fn to_string(&self) -> String {
//         self.lines.join("\n")
//     }
// }

// impl Display for Document {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.lines.join("\n"))
//     }
// }

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
                let inds: Vec<_> = Vec::new();

                for i in 0..overflow {
                    inds.push(curr_ind + i);
                }

                curr_ind += overflow;
            }
        }

        Self { file_name, lines }
    }

    pub fn get_line_from_cursor_pos(&self, cursor_row: u32) -> String {
        self.lines[(cursor_row - 2) as usize].clone()
    }

    pub fn set_line_from_cursor_pos(&mut self, cursor_row: u32, new_line: String) {
        self.lines[(cursor_row - 2) as usize] = new_line;
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}
