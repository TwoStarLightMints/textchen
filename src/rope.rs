// This rope implementation is based on my work in my data_structures_in_rust repo

use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RopeNode {
    Node {
        weight: usize,
        lhs: Option<Rc<RefCell<RopeNode>>>,
        rhs: Option<Rc<RefCell<RopeNode>>>,
    },
    Leaf {
        weight: usize,
        val: String,
    },
}

impl RopeNode {
    fn new_leaf(val: String) -> Self {
        RopeNode::Leaf {
            weight: val.len(),
            val,
        }
    }

    fn new_node(lhs: Option<Rc<RefCell<Self>>>, rhs: Option<Rc<RefCell<Self>>>) -> Self {
        RopeNode::Node {
            weight: Rc::clone(lhs.as_ref().unwrap()).borrow().lhs_weight(),
            lhs,
            rhs,
        }
    }

    fn lhs_weight(&self) -> usize {
        match self {
            RopeNode::Leaf { weight, val } => *weight,
            RopeNode::Node { weight, lhs, rhs } => match lhs {
                Some(l) => {
                    Rc::clone(l).borrow().lhs_weight()
                        + match rhs {
                            Some(r) => Rc::clone(r).borrow().lhs_weight(),
                            None => 0,
                        }
                }
                None => 0,
            },
        }
    }
}

impl From<String> for RopeNode {
    fn from(value: String) -> RopeNode {
        Self::new_leaf(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rope {
    root: Rc<RefCell<RopeNode>>,
}

impl Rope {
    pub fn new() -> Self {
        Self {
            root: Rc::new(RefCell::new(RopeNode::new_node(None, None))),
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
                    .map(|n| {
                        RopeNode::new_node(
                            Some(Rc::new(RefCell::new(n[0].clone()))),
                            Some(Rc::new(RefCell::new(n[1].clone()))),
                        )
                    })
                    .collect();
            } else {
                new_nodes = nodes
                    .windows(2)
                    .step_by(2)
                    .map(|n| {
                        RopeNode::new_node(
                            Some(Rc::new(RefCell::new(n[0].clone()))),
                            Some(Rc::new(RefCell::new(n[1].clone()))),
                        )
                    })
                    .collect();

                new_nodes.push(RopeNode::new_node(
                    Some(Rc::new(RefCell::new(nodes.last().unwrap().clone()))),
                    None,
                ));
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

    pub fn collect_leaves(&self) {
        let mut node_stack = vec![Rc::clone(&self.root)];

        while !node_stack.is_empty() {}
    }
}

struct RopeLeaves<'a> {
    leaves: Vec<&'a RopeNode>,
    index: usize,
}

impl<'a> RopeLeaves<'a> {
    pub fn new(leaves: Vec<&'a RopeNode>) -> Self {
        Self { leaves, index: 0 }
    }
}

impl<'a> Iterator for RopeLeaves<'a> {
    type Item = &'a RopeNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.leaves.len() {
            let res = Some(self.leaves[self.index]);

            self.index += 1;

            res
        } else {
            None
        }
    }
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
            root: Rc::new(RefCell::new(RopeNode::Node {
                weight: 6,
                lhs: Some(Rc::new(RefCell::new(RopeNode::Node {
                    weight: 3,
                    lhs: Some(Rc::new(RefCell::new(RopeNode::Leaf {
                        weight: 3,
                        val: "Hel".to_string(),
                    }))),
                    rhs: Some(Rc::new(RefCell::new(RopeNode::Leaf {
                        weight: 3,
                        val: "lo,".to_string(),
                    }))),
                }))),
                rhs: Some(Rc::new(RefCell::new(RopeNode::Node {
                    weight: 3,
                    lhs: Some(Rc::new(RefCell::new(RopeNode::Leaf {
                        weight: 3,
                        val: "_Wo".to_string(),
                    }))),
                    rhs: Some(Rc::new(RefCell::new(RopeNode::Leaf {
                        weight: 4,
                        val: "rld!".to_string(),
                    }))),
                }))),
            })),
        };

        assert_eq!(test_rope, control);
    }
}
