//! Element type for Decision Diagrams
use {
    crate::types::{DecisionDiagram, DecisionDiagramNode, Indexer},
    std::{
        collections::{HashMap, HashSet},
        hash::Hash,
        io, ptr,
        rc::Rc,
    },
};

pub type Node = Rc<Vertex>;

#[derive(Clone, Debug)]
pub enum Vertex {
    Bool(bool),
    Var {
        var_index: usize,
        low: Node,
        high: Node,
    },
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self, state)
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex::Bool(false)
    }
}

impl DecisionDiagram<Node> for Node {
    /// returns the number of nodes under self and self itself.
    ///```
    /// use ddir::node::Node;
    /// use ddir::types::{DecisionDiagram, DecisionDiagramNode};
    ///
    /// let f = Node::new_constant(false);
    /// assert_eq!(f.len(), 1);
    /// let n = Node::new_var(2, f.clone(), f.clone());
    /// assert_eq!(n.len(), 2);
    /// let k = Node::new_var(1, n.clone(), f.clone());
    /// assert_eq!(k.len(), 3);
    ///```
    fn len(&self) -> usize {
        self.all_nodes().len()
    }
    /// returns all nodes under self and self itself.
    ///```
    /// use ddir::node::Node;
    /// use ddir::types::{DecisionDiagram, DecisionDiagramNode};
    ///
    /// let f = Node::new_constant(false);
    /// let n = Node::new_var(2, f.clone(), f.clone());
    /// let k = Node::new_var(1, n.clone(), f.clone());
    /// assert_eq!(k.all_nodes().len(), 3);
    ///```
    fn all_nodes<'a>(&'a self) -> HashSet<&'a Node> {
        let mut map: HashSet<&'a Node> = HashSet::new();
        fn traverse<'a>(node: &'a Node, map: &mut HashSet<&'a Node>) {
            map.insert(node);
            if let Vertex::Var {
                ref low, ref high, ..
            } = **node
            {
                traverse(low, map);
                traverse(high, map);
            }
        }
        traverse(self, &mut map);
        map
    }
    fn write_as_gv(&self, mut sink: impl io::Write) -> io::Result<()> {
        sink.write_all(
            b"digraph regexp {{
  fontname=\"Helvetica,Arial,sans-serif\"
  node [fontname=\"Helvetica,Arial,sans-serif\"]
  edge [fontname=\"Helvetica,Arial,sans-serif\",color=blue]\n",
        )?;
        let mut index: HashMap<&Node, usize> = HashMap::new();
        let mut bools = (false, false);
        for (i, n) in self.all_nodes().iter().enumerate() {
            index.insert(*n, i + 2);
            match n.is_constant() {
                Some(false) => bools.0 |= true,
                Some(true) => bools.1 |= true,
                None => (),
            }
        }
        // nodes
        if bools.0 {
            sink.write_all(
                b"  0[style=filled,fillcolor=\"gray80\",label=\"false\",shape=\"box\"];\n",
            )?;
        }
        if bools.1 {
            sink.write_all(
                b"  1[style=filled,fillcolor=\"gray95\",label=\"true\",shape=\"box\"];\n",
            )?;
        }
        for node in self.all_nodes().iter() {
            if let Vertex::Var { ref var_index, .. } = ***node {
                let i = if let Some(b) = node.is_constant() {
                    b as usize
                } else {
                    *index.get(node).unwrap()
                };
                sink.write_all(format!("  {i}[label=\"{var_index}\"];\n").as_bytes())?;
            }
        }
        // edges
        for node in self.all_nodes().iter() {
            if let Vertex::Var {
                ref low, ref high, ..
            } = ***node
            {
                let i = if let Some(b) = node.is_constant() {
                    b as usize
                } else {
                    *index.get(node).unwrap()
                };
                let j = if let Some(b) = low.is_constant() {
                    b as usize
                } else {
                    *index.get(&low).unwrap()
                };
                let k = if let Some(b) = high.is_constant() {
                    b as usize
                } else {
                    *index.get(&high).unwrap()
                };
                if j == k {
                    sink.write_all(format!("  {i} -> {j}[color=black,penwidth=2];\n").as_bytes())?;
                } else {
                    sink.write_all(
                        format!("  {i} -> {j}[color=red,style=\"dotted\"];\n").as_bytes(),
                    )?;
                    sink.write_all(format!("  {i} -> {k}[color=blue];\n").as_bytes())?;
                }
            }
        }
        sink.write_all(b"}}\n")?;
        Ok(())
    }
    fn satisfy_one(&self) -> bool {
        if let Some(b) = self.is_constant() {
            return b;
        }
        if self.low().unwrap().satisfy_one() {
            return true;
        }
        self.high().unwrap().satisfy_one()
    }
    fn satisfy_all(&self) -> usize {
        let mut count: HashMap<&Node, usize> = HashMap::new();
        fn linear_count<'a>(count: &mut HashMap<&'a Node, usize>, node: &'a Node) -> usize {
            if let Some(n) = count.get(node) {
                return *n;
            }
            if let Some(b) = node.is_constant() {
                return b as usize;
            }
            let a = linear_count(count, node.low().unwrap());
            let b = linear_count(count, node.high().unwrap());
            let c = a + b;
            count.insert(node, c);
            c
        }
        linear_count(&mut count, self)
    }
}

impl DecisionDiagramNode for Node {
    /// returns a new terminal node.
    fn new_constant(b: bool) -> Node {
        Rc::new(Vertex::Bool(b))
    }
    /// returns a new non-terminal node.
    fn new_var(var_index: usize, low: Node, high: Node) -> Node {
        Rc::new(Vertex::Var {
            var_index,
            low,
            high,
        })
    }
    /// returns `None` if self is a non-terminal node.
    ///```
    /// use ddir::node::Node;
    /// use ddir::types::{DecisionDiagram, DecisionDiagramNode};
    ///
    /// let f = Node::new_constant(false);
    /// assert!(f.is_constant().is_some());
    ///```
    fn is_constant(&self) -> Option<bool> {
        match **self {
            Vertex::Bool(b) => Some(b),
            Vertex::Var { .. } => None,
        }
    }
    fn unified_key(&self) -> usize {
        match **self {
            Vertex::Bool(b) => b as usize,
            Vertex::Var { var_index, .. } => var_index + 2,
        }
    }
    /// returns the number of nodes under self and self itself.
    ///```
    /// use ddir::node::Node;
    /// use ddir::types::{DecisionDiagram, DecisionDiagramNode};
    ///
    /// let f = Node::new_constant(false);
    /// let n = Node::new_var(2, f.clone(), f.clone());
    /// assert_eq!(n.var_index(), Some(2));
    ///```
    fn var_index(&self) -> Option<usize> {
        match **self {
            Vertex::Bool(_) => None,
            Vertex::Var { var_index, .. } => Some(var_index),
        }
    }
    fn low(&self) -> Option<&Node> {
        match **self {
            Vertex::Bool(_) => None,
            Vertex::Var { ref low, .. } => Some(low),
        }
    }
    fn high(&self) -> Option<&Node> {
        match **self {
            Vertex::Bool(_) => None,
            Vertex::Var { ref high, .. } => Some(high),
        }
    }
    fn build_indexer(nodes: &[Self]) -> Indexer<Self> {
        let mut node: HashMap<usize, Node> = HashMap::new();
        let mut index: HashMap<Node, usize> = HashMap::new();
        {
            let f = Node::new_constant(false);
            let t = Node::new_constant(true);
            node.insert(0, f.clone());
            index.insert(f, 0);
            node.insert(1, t.clone());
            index.insert(t, 1);
        }
        let mut i: usize = 1;
        for root in nodes.iter() {
            for n in root.all_nodes().iter() {
                i += 1;
                node.insert(i, (*n).clone());
                index.insert(
                    (*n).clone(),
                    n.is_constant().map_or_else(|| i, |b| b as usize),
                );
            }
        }
        (index, node)
    }
}

pub mod example {
    use super::*;

    macro_rules! F {
        () => {
            Node::new_constant(false)
        };
    }
    macro_rules! T {
        () => {
            Node::new_constant(true)
        };
    }
    macro_rules! D {
        ($v: expr, $l: expr, $h: expr) => {
            Node::new_var($v, $l, $h)
        };
    }
    /// return the independent sets of 6 cyclic chain
    pub fn independent_set() -> Node {
        D!(
            1, //                                 1 -> {
            D!(
                2, //                               (2 -> {
                D!(
                    3, //                             (3 -> {
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), T!()), //           (6 -> {true. true}).
                            D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), T!()), //           (6 -> {true. true}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), T!()), //           (6 -> {true. true}).
                            D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }).
                        )
                    )
                ),
                D!(
                    3, //                             (3 -> {
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), T!()), //           (6 -> {true. true}).
                            D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), T!()), //           (6 -> {true. true}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }) }),
                        )
                    )
                )
            ),
            D!(
                2, //                               (2 -> {
                D!(
                    3, //                             (3 -> {
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), F!()), //           (6 -> {true. false}).
                            D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), F!()), //           (6 -> {true. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, T!(), F!()), //           (6 -> {true. false}).
                            D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }).
                        )
                    )
                ),
                D!(
                    3, //                             (3 -> {
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //                           (4 -> {
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }).
                        ),
                        D!(
                            5,                 //         (5 -> {
                            D!(6, F!(), F!()), //           (6 -> {false. false}).
                            D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }) }) })
                        )
                    )
                )
            )
        )
    }

    /// maximal independent subsets, is also called kernels.
    ///  the kernels correspond to such arrangements in which there also are no three consecutive 0s.
    pub fn kernels() -> Node {
        D!(
            1,
            D!(
                2,
                D!(
                    3,
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!())),
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!()))
                    ),
                    D!(
                        4,
                        D!(5, D!(6, F!(), T!()), D!(6, F!(), F!())),
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!()))
                    )
                ),
                D!(
                    3,
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, T!(), F!())),
                        D!(5, D!(6, F!(), T!()), D!(6, F!(), F!()))
                    ),
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!())),
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!()))
                    )
                )
            ),
            D!(
                2,
                D!(
                    3,
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!())),
                        D!(5, D!(6, T!(), F!()), D!(6, F!(), F!()))
                    ),
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, T!(), F!())),
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!()))
                    )
                ),
                D!(
                    3,
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!())),
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!()))
                    ),
                    D!(
                        4,
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!())),
                        D!(5, D!(6, F!(), F!()), D!(6, F!(), F!()))
                    )
                )
            )
        )
    }
    /// majority
    pub fn majority() -> Node {
        D!(
            1,
            D!(2, F!(), D!(3, F!(), T!())),
            D!(2, D!(3, F!(), T!()), T!())
        )
    }
    // From Figure 7 of Randal E. Bryant, Graph-Based Algorithms for Boolean
    // Function Manipulation, IEEE Trans. en Comp., C-35-8, pp.677-691, Aug. 1986.
    pub fn x1x3() -> Node {
        D!(1, T!(), D!(3, T!(), F!()))
    }
    // From Figure 7 of Randal E. Bryant, Graph-Based Algorithms for Boolean
    // Function Manipulation, IEEE Trans. en Comp., C-35-8, pp.677-691, Aug. 1986.
    pub fn x2x3() -> Node {
        D!(2, F!(), D!(3, F!(), T!()))
    }
    pub fn x1x2x4() -> Node {
        D!(1, D!(2, T!(), D!(4, T!(), F!())), D!(2, F!(), T!()))
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::{
            // node::{example, Node},
            types::{DecisionDiagram, DecisionDiagramNode},
        },
    };

    #[test]
    fn test_satisfy() {
        let f = Node::new_constant(false);
        assert_eq!(f.satisfy_one(), false);
        assert_eq!(f.satisfy_all(), 0);
        let t = Node::new_constant(true);
        assert_eq!(t.satisfy_one(), true);
        assert_eq!(t.satisfy_all(), 1);
        let tt: Node = Node::new_var(2, t.clone(), t.clone());
        assert_eq!(tt.satisfy_one(), true);
        assert_eq!(tt.satisfy_all(), 2);
        let major = example::majority();
        assert_eq!(major.satisfy_one(), true);
        assert_eq!(major.satisfy_all(), 3);
        let ind = example::independent_set();
        assert_eq!(ind.satisfy_one(), true);
        assert_eq!(ind.satisfy_all(), 18);
    }
}
