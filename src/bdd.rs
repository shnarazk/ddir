use {
    crate::dd::{DecisionDiagramTrait, Node, Vertex, DDT},
    itertools::Itertools,
    std::{
        collections::{HashMap, HashSet},
        io,
        marker::PhantomData,
    },
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BDD {
    graph: Node,
    phantom: PhantomData<()>,
}

impl DecisionDiagramTrait for BDD {
    fn new_constant(b: bool) -> Self {
        Self {
            graph: Node::new_constant(b),
            ..Default::default()
        }
    }
    fn new_var(var_index: usize, low: Node, high: Node) -> Self {
        Self {
            graph: Node::new_var(var_index, low, high),
            ..Default::default()
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
    fn write_as_graphvis(&self, sink: impl io::Write) -> io::Result<()> {
        self.graph.write_as_graphvis(sink)
    }
}

impl BDD {
    pub fn new_from(graph: DDT) -> BDD {
        Self {
            graph: graph.graph,
            ..Default::default()
        }
    }
}

pub trait ToBinaryDecisionDiagram {
    fn to_bdd(&self) -> BDD;
}

impl ToBinaryDecisionDiagram for BDD {
    fn to_bdd(&self) -> BDD {
        self.clone()
    }
}

impl ToBinaryDecisionDiagram for DDT {
    fn to_bdd(&self) -> BDD {
        let mut bdd = BDD {
            graph: self.graph.clone(),
            ..Default::default()
        };
        bdd.reduce();
        bdd
    }
}

impl ToBinaryDecisionDiagram for Node {
    fn to_bdd(&self) -> BDD {
        let mut bdd = BDD {
            graph: self.clone(),
            ..Default::default()
        };
        bdd.reduce();
        bdd
    }
}

pub trait BinaryDecisionDiagram {
    fn reduce(&mut self);
}

impl BinaryDecisionDiagram for BDD {
    // convert tree to BDD
    fn reduce(&mut self) {
        let root = self.graph.clone();
        let nodes = root.all_nodes();
        let mut to_index: HashMap<Node, usize> = HashMap::new();
        let mut from_index: HashMap<usize, Node> = HashMap::new();
        from_index.insert(0, Node::new_constant(false));
        from_index.insert(1, Node::new_constant(true));
        let mut vlist: HashMap<usize, Vec<&Node>> = HashMap::new();
        // put each vertex u on list vlist[u.var_index]
        for (i, n) in nodes.iter().cloned().enumerate() {
            if let Some(v) = n.var_index() {
                to_index.insert(n.clone(), 1 + 2);
                from_index.insert(i + 2, n.clone());
                vlist.entry(v).or_default().push(n);
            } else if let Some(b) = n.is_constant() {
                to_index.insert(n.clone(), b as usize);
                from_index.insert(i, n.clone());
            }
        }
        let mut next_id: usize = 2;
        for vi in vlist.keys().sorted().rev() {
            let lst = vlist.get(vi).unwrap();
            let mut q: Vec<((usize, usize), &Node)> = Vec::new();
            let mut old_key: (usize, usize) = (0, 0);
            for node in lst.iter().cloned() {
                match ***node {
                    Vertex::Bool(_) => (),
                    Vertex::Var {
                        ref low, ref high, ..
                    } => {
                        if to_index.get(low) == to_index.get(high) {
                            // redundant vertex
                            to_index.insert(node.clone(), *to_index.get(low).unwrap());
                        } else {
                            q.push((
                                (*to_index.get(low).unwrap(), *to_index.get(high).unwrap()),
                                node,
                            ));
                        }
                    }
                }
            }
            q.sort_unstable_by_key(|(k, _)| *k);
            for (key, node) in q.iter().cloned() {
                if key == old_key {
                    to_index.insert(node.clone(), next_id);
                } else {
                    next_id += 1;
                    match ***node {
                        Vertex::Bool(_) => {
                            to_index.insert(node.clone(), next_id);
                            from_index.insert(next_id, node.clone());
                        }
                        Vertex::Var {
                            var_index,
                            ref low,
                            ref high,
                        } => {
                            let l = from_index.get(to_index.get(low).unwrap()).unwrap();
                            let h = from_index.get(to_index.get(high).unwrap()).unwrap();
                            let n = Node::new_var(var_index, (*l).clone(), (*h).clone());
                            to_index.insert(n.clone(), next_id);
                            from_index.insert(next_id, n);
                            // Rc::get_mut(&mut **node);
                        }
                    }
                    old_key = key;
                }
            }
        }
        // pick up a tree from the hash-table
        self.graph = from_index
            .get(to_index.get(&root).unwrap())
            .unwrap()
            .clone();
        // dbg!(&self.graph);
    }
}

#[cfg(test)]
mod test {
    use crate::bdd::{ToBinaryDecisionDiagram, BDD};
    use crate::dd::{DecisionDiagramTrait, Node, DDT};

    #[test]
    fn test() {
        let f = Node::new_constant(false);
        let n: Node = Node::new_var(2, f.clone(), f.clone());
        let tree: DDT = DDT { graph: n };
        let bdd: BDD = tree.to_bdd();
        assert_eq!(bdd.len(), 1);
    }
}
