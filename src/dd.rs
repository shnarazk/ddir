use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    io,
    rc::Rc,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DDT {
    graph: Node,
}

pub type Node = Rc<Box<Vertex>>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Vertex {
    Bool(bool),
    Var {
        var_index: usize,
        low: Node,
        high: Node,
    },
}

pub trait DecisionDiagramTrait {
    fn new_constant(b: bool) -> Self;
    fn new_var(var_index: usize, low: Node, high: Node) -> Self;
    fn is_constant(&self) -> Option<bool>;
    fn all_nodes<'a>(&'a self) -> HashSet<&Node>;
    fn len(&self) -> usize;
    fn write_as_graphvis(&self, sink: impl io::Write) -> io::Result<()>;
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
    /// assert_eq!(f.len(), 1);
    /// let n = Node::new_var(2, f.clone(), f.clone());
    /// assert_eq!(n.len(), 2);
    /// let k = Node::new_var(1, n.clone(), f.clone());
    /// assert_eq!(k.len(), 3);
    ///```
    fn len<'a>(&'a self) -> usize {
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
    fn write_as_graphvis(&self, mut sink: impl io::Write) -> io::Result<()> {
        sink.write(
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
        sink.write(b"  0[label=\"false\"];\n")?;
        sink.write(b"  1[label=\"true\"];\n")?;
        for node in self.all_nodes().iter() {
            if let Vertex::Var { ref var_index, .. } = ****node {
                let i = if let Some(b) = node.is_constant() {
                    b as usize
                } else {
                    *index.get(node).unwrap()
                };
                sink.write(format!("  {i}[label=\"{var_index}\"];\n").as_bytes())?;
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
                    sink.write(format!("  {i} -> {j}[color=black];\n").as_bytes())?;
                } else {
                    sink.write(
                        format!("  {i} -> {j}[color=blue, style=\"dotted\"];\n").as_bytes(),
                    )?;
                    sink.write(format!("  {i} -> {k}[color=red, style=\"dotted\"];\n").as_bytes())?;
                }
            }
        }
        sink.write(b"}}\n")?;
        Ok(())
    }
}
