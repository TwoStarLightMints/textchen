use crate::document::Line;
use std::fmt;

pub struct GapBuf {
    pub lhs: Vec<char>,
    pub rhs: Vec<char>,
}

impl GapBuf {
    pub fn new() -> Self {
        Self {
            lhs: Vec::new(),
            rhs: Vec::new(),
        }
    }

    // Here, ind refers to the index at which to split the source string
    pub fn from_str(src: String, ind: usize) -> Self {
        let (lhs, rhs) = src.split_at(ind);

        Self {
            lhs: lhs.chars().collect(),
            rhs: rhs.chars().collect(),
        }
    }

    pub fn from_line(src: Line, ind: usize) -> Self {
        let (lhs, rhs) = src.1.split_at(ind);

        Self {
            lhs: lhs.chars().collect(),
            rhs: rhs.chars().collect(),
        }
    }

    pub fn insert(&mut self, c: char) {
        self.lhs.push(c);
    }

    pub fn pop(&mut self) {
        self.lhs.pop();
        self.lhs.shrink_to_fit();
    }

    pub fn len(&self) -> usize {
        self.lhs.iter().chain(self.rhs.iter()).count()
    }

    pub fn collect_to_string(&self) -> String {
        self.lhs.iter().chain(self.rhs.iter()).collect()
    }

    pub fn collect_to_pieces(&self) -> (String, String) {
        let lhs = self.lhs.iter().collect::<String>();
        let rhs = self.rhs.iter().collect::<String>();
        (lhs, rhs)
    }
}

impl fmt::Display for GapBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.lhs.iter().chain(self.rhs.iter()).collect::<String>()
        )
    }
}
