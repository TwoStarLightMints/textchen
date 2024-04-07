// This rope implementation is based on my work in my data_structures_in_rust repo

use std::{cell::RefCell, rc::Rc};

pub struct RopeNode {
    weight: usize,

    // These are behind Rcs to enable easier backtracking later
    lhs: Option<Rc<RefCell<RopeNode>>>,
    rhs: Option<Rc<RefCell<RopeNode>>>,

    str_piece: Option<String>,
}

impl RopeNode {
    fn new() -> Self {
        Self {
            weight: 0,
            lhs: None,
            rhs: None,
            str_piece: None,
        }
    }
}

impl From<String> for RopeNode {
    fn from(value: String) -> Self {
        Self {
            weight: value.len(),
            lhs: None,
            rhs: None,
            str_piece: Some(value),
        }
    }
}

pub struct Rope {
    root: RopeNode,
}
