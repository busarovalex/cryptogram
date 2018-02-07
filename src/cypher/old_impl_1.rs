use std::collections::{HashSet, HashMap};
use std::rc::Rc;
// use std::cell::RefCell;

#[derive(Debug)]
struct Cypher {
    phrase: Vec<String>,
    // root_solution: RootSolution
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
struct CharEncoding {
    from: char,
    to: char
}

// #[derive(Debug)]
// struct RootSolution {
//     start_encodings: Vec<Solution>
// }

// #[derive(Debug)]
// struct Solution {
//     encoding: CharEncoding,
//     next_encoding: Vec<Solution>
// }

#[derive(Debug)]
struct SolutionNodeMap {
    solutions: HashMap<CharEncoding, Vec<Rc<SolutionNode>>>
}

#[derive(Debug, Clone)]
struct SolutionNode {
    encoding: CharEncoding,
    next_node: Option<Rc<SolutionNode>>
}

#[derive(Debug, Clone)]
struct SolutionNodeInner {
    encoding: CharEncoding
}

#[derive(Debug)]
struct Solver {
    node_map: SolutionNodeMap,
    possible_solutions: Vec<Rc<SolutionNode>>
}

impl Solver {
    fn new(known_char_encodings: Vec<CharEncoding>) -> Solver {
        if known_char_encodings.is_empty() {
            return Solver {
                node_map: SolutionNodeMap::new(),
                possible_solutions: Vec::with_capacity(1024)
            };
        } 
        let initial_solution_node = Rc::new(known_char_encodings.into_iter()
            .fold(None, |previous_node, next_encoding| {
                match previous_node {
                    Some(node) => Some(SolutionNode::new(next_encoding, node)),
                    None => Some(SolutionNode::without_next(next_encoding))
                }
            }).unwrap());
        let mut node_map = SolutionNodeMap::new();
        node_map.insert(initial_solution_node.clone());
        Solver {
            node_map,
            possible_solutions: vec![initial_solution_node]
        }
    }

    fn insert(&mut self, new_possible_solution: Rc<SolutionNode>) {
        self.node_map.insert(new_possible_solution.clone());
        self.possible_solutions.push(new_possible_solution);
    }

    fn get_possible_non_contradictive_solutions(&self, for_solution_node: &SolutionNode) -> Vec<Rc<SolutionNode>> {
        Vec::new()
    }
}

impl SolutionNodeMap {
    fn new() -> SolutionNodeMap {
        SolutionNodeMap {
            solutions: HashMap::new()
        }
    }

    fn insert(&mut self, reference: Rc<SolutionNode>) {
        self.solutions.entry(reference.encoding).or_insert_with(Vec::new).push(reference);
    }
}

// #[derive(Debug, Clone)]
// struct PartialSolution {
//     encoding: Vec<CharEncoding>
// }

// #[derive(Debug)]
// struct PartialSolutionReference {
//     field: Type
// }

// impl PartialSolution {
//     fn clone_from_inclusive(&self, char_encoding: CharEncoding) -> PartialSolution {
//         PartialSolution {
//             encoding: self.encoding.iter().skip_while(|encoding| **encoding != char_encoding).cloned().collect()
//         }
//     }
// }

impl SolutionNode {
    fn clone_chain(&self) -> SolutionNode {
        SolutionNode {
            encoding: self.encoding,
            next_node: self.next_node.as_ref().map(|next_node| Rc::new(next_node.clone_chain()))
        }
    }

    fn new(encoding: CharEncoding, next_node: SolutionNode) -> SolutionNode {
        SolutionNode {
            encoding,
            next_node: Some(Rc::new(next_node))
        }
    }

    fn without_next(encoding: CharEncoding) -> SolutionNode {
        SolutionNode {
            encoding,
            next_node: None
        }
    }
}

impl Cypher {
    fn new(phrase: Vec<String>, known_char_encodings: HashSet<CharEncoding>) -> Cypher {
        unimplemented!()
        // let mut known_char_encodings = known_char_encodings.into_iter().collect();
        // known_char_encodings.sort_unstable_by(|a, b| b.cmp(a));

        // let root_solutionr = RootSolution {
        //     start_encodings: 
        // };

        // Cypher {
        //     phrase,
        //     root_solution
        // }
    }
}
