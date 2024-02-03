//! Zero-suppressed Decision Diagram

use {
    crate::{
        node::{Node, Vertex},
        types::{DecisionDiagram, DecisionDiagramNode, ReducedDecisionDiagram},
    },
    itertools::Itertools,
    std::{
        collections::{HashMap, HashSet},
        io,
        marker::PhantomData,
    },
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ZDD<N> {
    graph: N,
    phantom: PhantomData<()>,
}

impl ZDD<Node> {
    pub fn new_from(graph: Node) -> ZDD<Node> {
        let mut zdd = ZDD {
            graph: graph.clone(),
            ..Default::default()
        };
        zdd.reduce();
        zdd
    }
}

impl<N: DecisionDiagram<N> + DecisionDiagramNode> DecisionDiagram<N> for ZDD<N> {
    fn all_nodes(&self) -> HashSet<&N> {
        self.graph.all_nodes()
    }
    fn len(&self) -> usize {
        self.graph.len()
    }
    fn write_as_gv(&self, sink: impl io::Write) -> io::Result<()> {
        self.graph.write_as_gv(sink)
    }
}

impl ReducedDecisionDiagram for ZDD<Node> {
    fn reduce(&mut self) {
        let root = self.graph.clone();
        let nodes = root.all_nodes();
        let mut to_index: HashMap<Node, usize> = HashMap::new();
        let mut from_index: HashMap<usize, Node> = HashMap::new();
        from_index.insert(0, Node::new_constant(false));
        from_index.insert(1, Node::new_constant(true));
        let mut vlist: HashMap<usize, Vec<&Node>> = HashMap::new();
        // put each vertex u on list vlist[u.var_index]
        for n in nodes.iter().cloned() {
            if let Some(v) = n.var_index() {
                to_index.insert(n.clone(), v + 2);
                vlist.entry(v).or_default().push(n);
            } else if let Some(b) = n.is_constant() {
                to_index.insert(n.clone(), b as usize);
            }
        }
        let mut next_id: usize = 2;
        for vi in vlist.keys().sorted().rev() {
            let lst = vlist.get(vi).unwrap();
            let mut q: Vec<((usize, usize), &Node)> = Vec::new();
            let mut old_key: (usize, usize) = (0, 0);
            for node in lst.iter().cloned() {
                match **node {
                    Vertex::Bool(_) => (),
                    Vertex::Var {
                        ref low, ref high, ..
                    } => {
                        if high.is_constant() == Some(false) {
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
                    match **node {
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
                            to_index.insert(node.clone(), next_id);
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
    }
    fn apply(&self, _op: Box<dyn Fn(bool, bool) -> bool>, _unit: bool, _other: &Self) -> Self {
        todo!()
    }
}
