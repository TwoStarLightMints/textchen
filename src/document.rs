use std::fmt::Display;

pub struct Document {
    pub file_name: String,
    pub lines: Vec<String>,
}

impl Document {
    pub fn new(file_name: String, content: String) -> Self {
        Self {
            file_name,
            lines: content.lines().map(|e| e.to_string()).collect(),
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}
