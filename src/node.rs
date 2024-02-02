use {
    crate::dd::DecisionDiagramTrait,
    std::{
        collections::{HashMap, HashSet},
        hash::Hash,
        io, ptr,
        rc::Rc,
    },
};

pub type Node = Rc<Vertex>;

pub trait DecisionDiagramNode {
    fn new_constant(b: bool) -> Self;
    fn new_var(var_index: usize, low: Node, high: Node) -> Self;
    fn is_constant(&self) -> Option<bool>;
    // return 0 or 1 for terminal nodes, and `vi + 2` for nonterminal node which var_index is `vi`.
    fn unified_key(&self) -> usize;
    fn var_index(&self) -> Option<usize>;
    fn low(&self) -> Option<&Self>;
    fn high(&self) -> Option<&Self>;
}

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

impl DecisionDiagramTrait for Node {
    /// returns the number of nodes under self and self itself.
    ///```
    /// use ddir::dd::{DecisionDiagramNode, DecisionDiagramTrait, Node};
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
    /// use ddir::dd::{DecisionDiagramNode, DecisionDiagramTrait, Node};
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
        for (i, n) in self.all_nodes().iter().enumerate() {
            if let Vertex::Var { .. } = ***n {
                index.insert(*n, i + 2);
            }
        }
        // nodes
        sink.write_all(b"  0[style=filled,fillcolor=\"gray80\",label=\"false\",shape=\"box\"];\n")?;
        sink.write_all(b"  1[style=filled,fillcolor=\"gray95\",label=\"true\",shape=\"box\"];\n")?;
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
}

impl DecisionDiagramNode for Node {
    /// returns a new terminal node.
    fn new_constant(b: bool) -> Self {
        Rc::new(Vertex::Bool(b))
    }
    /// returns a new non-terminal node.
    fn new_var(var_index: usize, low: Node, high: Node) -> Self {
        Rc::new(Vertex::Var {
            var_index,
            low,
            high,
        })
    }
    /// returns `None` if self is a non-terminal node.
    ///```
    /// use ddir::dd::{DecisionDiagramNode, DecisionDiagramTrait, Node};
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
    /// use ddir::dd::{DecisionDiagramNode, DecisionDiagramTrait, Node};
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
    fn low(&self) -> Option<&Self> {
        match **self {
            Vertex::Bool(_) => None,
            Vertex::Var { ref low, .. } => Some(low),
        }
    }
    fn high(&self) -> Option<&Self> {
        match **self {
            Vertex::Bool(_) => None,
            Vertex::Var { ref high, .. } => Some(high),
        }
    }
}
