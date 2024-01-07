use crate::document::Line;
use std::fmt;

pub struct GapBuf {
    pub lhs: Vec<char>,
    pub rhs: String,
}

impl GapBuf {
    pub fn new() -> Self {
        Self {
            lhs: Vec::new(),
            rhs: "".to_string(),
        }
    }

    pub fn from_str(src: String, ind: usize) -> Self {
        //! ind refers to the index at which to split the source string

        let (lhs, rhs) = src.split_at(ind);

        Self {
            lhs: lhs.chars().collect(),
            rhs: rhs.to_owned(),
        }
    }

    pub fn from_line(src: &Line, ind: usize) -> Self {
        let (lhs, rhs) = src.1.split_at(ind);

        Self {
            lhs: lhs.chars().collect(),
            rhs: rhs.to_owned(),
        }
    }

    pub fn insert(&mut self, c: char) {
        self.lhs.push(c);
    }

    pub fn pop(&mut self) {
        self.lhs.pop();
        self.lhs.shrink_to_fit();
    }

    pub fn pop_tab(&mut self) {
        for _ in 0..4 {
            self.lhs.pop();
        }
        self.lhs.shrink_to_fit();
    }

    pub fn len(&self) -> usize {
        self.lhs.iter().count() + self.rhs.len()
    }

    pub fn collect_to_string(&self) -> String {
        // self.lhs.iter().chain(self.rhs.iter()).collect()
        self.lhs.iter().collect::<String>() + self.rhs.as_str()
    }

    pub fn collect_to_pieces(&self) -> (String, String) {
        let lhs = self.lhs.iter().collect::<String>();
        let rhs = self.rhs.to_owned();
        (lhs, rhs)
    }
}

impl fmt::Display for GapBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.lhs.iter().collect::<String>(), self.rhs)
    }
}
