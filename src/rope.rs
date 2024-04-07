// This rope implementation is based on my work in my data_structures_in_rust repo

use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RopeNode {
    /// If a node is a leaf, the weight is 0, otherwise it is the length of the str_piece
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

    fn from_nodes(lhs: Self, rhs: Self) -> Self {
        Self {
            weight: lhs.lhs_weight(),
            lhs: Some(Rc::new(RefCell::new(lhs))),
            rhs: Some(Rc::new(RefCell::new(rhs))),
            str_piece: None,
        }
    }

    fn from_node(lhs: Self) -> Self {
        Self {
            weight: lhs.lhs_weight(),
            lhs: Some(Rc::new(RefCell::new(lhs))),
            rhs: None,
            str_piece: None,
        }
    }

    fn lhs_weight(&self) -> usize {
        match self.str_piece {
            Some(_) => self.weight,
            None => {
                let curr_l = self.lhs.as_ref().unwrap().borrow().lhs_weight();

                match self.rhs.as_ref() {
                    Some(n) => n.borrow().lhs_weight() + curr_l,
                    None => curr_l,
                }
            }
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

#[derive(Debug, PartialEq, Eq)]
pub struct Rope {
    root: Rc<RefCell<RopeNode>>,
}

impl Rope {
    pub fn new() -> Self {
        Self {
            root: Rc::new(RefCell::new(RopeNode::new())),
        }
    }

    pub fn from_str(value: String, doc_disp_width: usize) -> Self {
        //! I use a custom from_str instead of implementing From or FromStr,
        //! because I need the document's display width

        fn check_is_within_width(pieces: &Vec<String>, doc_disp_width: usize) -> bool {
            for piece in pieces {
                if piece.len() > doc_disp_width {
                    return false;
                }
            }

            true
        }

        fn process_nodes_to_root(nodes: Vec<RopeNode>) -> RopeNode {
            let mut new_nodes: Vec<RopeNode>;

            if nodes.len() % 2 == 0 {
                new_nodes = nodes
                    .windows(2)
                    .step_by(2)
                    .map(|n| RopeNode::from_nodes(n[0].clone(), n[1].clone()))
                    .collect();
            } else {
                new_nodes = nodes
                    .windows(2)
                    .step_by(2)
                    .map(|n| RopeNode::from_nodes(n[0].clone(), n[1].clone()))
                    .collect();

                new_nodes.push(RopeNode::from_node(nodes.last().unwrap().clone()));
            }

            if new_nodes.len() > 1 {
                process_nodes_to_root(new_nodes)
            } else {
                new_nodes[0].clone()
            }
        }

        let mut pieces = vec![value];

        while !check_is_within_width(&pieces, doc_disp_width) {
            pieces = pieces
                .iter()
                .map(|p| {
                    let (lhs, rhs) = p.split_at(p.len() / 2);

                    vec![lhs.to_string(), rhs.to_string()]
                })
                .flatten()
                .collect();
        }

        Self {
            root: Rc::new(RefCell::new(process_nodes_to_root(
                pieces.into_iter().map(|p| RopeNode::from(p)).collect(),
            ))),
        }
    }

    pub fn collect_leaves(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_to_rope() {
        let test_str = String::from("Hello,_World!");

        let test_rope = Rope::from_str(test_str, 4);

        println!("{test_rope:#?}");

        let control = Rope {
            root: Rc::new(RefCell::new(RopeNode {
                weight: 6,
                lhs: Some(Rc::new(RefCell::new(RopeNode {
                    weight: 3,
                    lhs: Some(Rc::new(RefCell::new(RopeNode {
                        weight: 3,
                        lhs: None,
                        rhs: None,
                        str_piece: Some("Hel".to_string()),
                    }))),
                    rhs: Some(Rc::new(RefCell::new(RopeNode {
                        weight: 3,
                        lhs: None,
                        rhs: None,
                        str_piece: Some("lo,".to_string()),
                    }))),
                    str_piece: None,
                }))),
                rhs: Some(Rc::new(RefCell::new(RopeNode {
                    weight: 3,
                    lhs: Some(Rc::new(RefCell::new(RopeNode {
                        weight: 3,
                        lhs: None,
                        rhs: None,
                        str_piece: Some("_Wo".to_string()),
                    }))),
                    rhs: Some(Rc::new(RefCell::new(RopeNode {
                        weight: 4,
                        lhs: None,
                        rhs: None,
                        str_piece: Some("rld!".to_string()),
                    }))),
                    str_piece: None,
                }))),
                str_piece: None,
            })),
        };

        assert_eq!(test_rope, control);
    }
}
