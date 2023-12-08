use std::fmt::Display;

pub struct Document {
    pub lines: Vec<String>,
}

impl Document {
    pub fn new(content: String) -> Self {
        Self {
            lines: content.lines().map(|e| e.to_string()).collect(),
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}
