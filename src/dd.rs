use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    io, ptr,
    rc::Rc,
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DDT {
    pub(crate) graph: Node,
}

pub type Node = Rc<Box<Vertex>>;

#[derive(Clone, Debug, Hash)]
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

impl Default for Vertex {
    fn default() -> Self {
        Vertex::Bool(false)
    }
}

pub trait DecisionDiagramTrait {
    fn new_constant(b: bool) -> Self;
    fn new_var(var_index: usize, low: Node, high: Node) -> Self;
    fn is_constant(&self) -> Option<bool>;
    fn var_index(&self) -> Option<usize>;
    fn all_nodes(&self) -> HashSet<&Node>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn write_as_gv(&self, sink: impl io::Write) -> io::Result<()>;
}

impl DecisionDiagramTrait for DDT {
    fn new_constant(b: bool) -> Self {
        Self {
            graph: Node::new_constant(b),
        }
    }
    fn new_var(var_index: usize, low: Node, high: Node) -> Self {
        Self {
            graph: Node::new_var(var_index, low, high),
        }
    }
    fn is_constant(&self) -> Option<bool> {
        self.graph.is_constant()
    }
    fn var_index(&self) -> Option<usize> {
        self.graph.var_index()
    }
    fn all_nodes(&self) -> HashSet<&Node> {
        self.graph.all_nodes()
    }
    fn len(&self) -> usize {
        self.graph.len()
    }
    fn write_as_gv(&self, sink: impl io::Write) -> io::Result<()> {
        self.graph.write_as_gv(sink)
    }
}

impl DecisionDiagramTrait for Node {
    /// returns a new terminal node.
    fn new_constant(b: bool) -> Self {
        Rc::new(Box::new(Vertex::Bool(b)))
    }
    /// returns a new non-terminal node.
    fn new_var(var_index: usize, low: Node, high: Node) -> Self {
        Rc::new(Box::new(Vertex::Var {
            var_index,
            low,
            high,
        }))
    }
    /// returns `None` if self is a non-terminal node.
    ///```
    /// use ddir::dd::{DecisionDiagramTrait, Node};
    ///
    /// let f = Node::new_constant(false);
    /// assert!(f.is_constant().is_some());
    ///```
    fn is_constant(&self) -> Option<bool> {
        match ***self {
            Vertex::Bool(b) => Some(b),
            Vertex::Var { .. } => None,
        }
    }
    /// returns the number of nodes under self and self itself.
    ///```
    /// use ddir::dd::{DecisionDiagramTrait, Node};
    ///
    /// let f = Node::new_constant(false);
    /// let n = Node::new_var(2, f.clone(), f.clone());
    /// assert_eq!(n.var_index(), Some(2));
    ///```
    fn var_index(&self) -> Option<usize> {
        match ***self {
            Vertex::Bool(_) => None,
            Vertex::Var { var_index, .. } => Some(var_index),
        }
    }
    /// returns the number of nodes under self and self itself.
    ///```
    /// use ddir::dd::{DecisionDiagramTrait, Node};
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
    /// use ddir::dd::{DecisionDiagramTrait, Node};
    ///
    /// let f = Node::new_constant(false);
    /// let n = Node::new_var(2, f.clone(), f.clone());
    /// let k = Node::new_var(1, n.clone(), f.clone());
    /// assert_eq!(k.all_nodes().len(), 3);
    ///```
    fn all_nodes<'a>(&'a self) -> HashSet<&Node> {
        let mut map: HashSet<&'a Node> = HashSet::new();
        fn traverse<'a>(node: &'a Node, map: &mut HashSet<&'a Node>) {
            map.insert(node);
            if let Vertex::Var {
                ref low, ref high, ..
            } = ***node
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
        for (i, n) in self.all_nodes().iter().enumerate() {
            if let Vertex::Var { .. } = ****n {
                index.insert(*n, i + 2);
            }
        }
        // nodes
        sink.write_all(b"  0[style=filled,fillcolor=\"gray80\",label=\"false\",shape=\"box\"];\n")?;
        sink.write_all(b"  1[style=filled,fillcolor=\"gray95\",label=\"true\",shape=\"box\"];\n")?;
        for node in self.all_nodes().iter() {
            if let Vertex::Var { ref var_index, .. } = ****node {
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
            } = ****node
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
                    sink.write_all(format!("  {i} -> {j}[color=blue];\n").as_bytes())?;
                    sink.write_all(
                        format!("  {i} -> {k}[color=red,style=\"dotted\"];\n").as_bytes(),
                    )?;
                }
            }
        }
        sink.write_all(b"}}\n")?;
        Ok(())
    }
}

/// return the independence sets of 6 cyclic chain
pub fn sample1() -> DDT {
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
    DDT {
        graph: D!(
            1, // 1 -> {
            D!(
                2, //     (2 -> {
                D!(
                    3, //          (3 -> {
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), T!()), //                         (6 -> {true. true}).
                            D!(6, T!(), F!())  //                         (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), T!()), //                         (6 -> {true. true}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), T!()), //                         (6 -> {true. true}).
                            D!(6, T!(), F!())  //                         (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }) }).
                        )
                    )
                ),
                D!(
                    3, //          (3 -> {
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), T!()), //                         (6 -> {true. true}).
                            D!(6, T!(), F!())  //                         (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), T!()), //                         (6 -> {true. true}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!())  //                         (6 -> {false. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }) }) }),
                        )
                    )
                )
            ),
            D!(
                2, //     (2 -> {
                D!(
                    3, //          (3 -> {
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), F!()), //                         (6 -> {true. false}).
                            D!(6, T!(), F!())  //                         (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), F!()), //                         (6 -> {true. false}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, T!(), F!()), //                         (6 -> {true. false}).
                            D!(6, T!(), F!())  //                         (6 -> {true. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }) }).
                        )
                    )
                ),
                D!(
                    3, //          (3 -> {
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!())  //                         (6 -> {false. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }).
                        )
                    ),
                    D!(
                        4, //               (4 -> {
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!())  //                         (6 -> {false. false}) }).
                        ),
                        D!(
                            5,                 //                    (5 -> {
                            D!(6, F!(), F!()), //                         (6 -> {false. false}).
                            D!(6, F!(), F!()) //                         (6 -> {false. false}) }) }) }) }) }                        )
                        )
                    )
                )
            )
        ),
    }
}
