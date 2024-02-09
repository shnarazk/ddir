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
    fn satisfy_one(&self) -> bool {
        self.graph.satisfy_one()
    }
    fn satisfy_all(&self) -> usize {
        self.graph.satisfy_all()
    }
}

impl ReducedDecisionDiagram for ZDD<Node> {
    fn reduce(&mut self) {
        let root = &self.graph;
        let (mut index, mut node) = Node::build_indexer(&[root.clone()]);
        let mut vlist: HashMap<usize, Vec<&Node>> = HashMap::new();
        // put each vertex u on list vlist[u.var_index]
        for n in root.all_nodes().iter().cloned() {
            vlist.entry(n.unified_key()).or_default().push(n);
        }
        let mut next_id: usize = 2;
        for vi in vlist.keys().sorted().rev() {
            let mut q: Vec<((usize, usize), &Node)> = Vec::new();
            for node in vlist[vi].iter().cloned() {
                match **node {
                    Vertex::Bool(_) => (),
                    Vertex::Var {
                        ref low, ref high, ..
                    } => {
                        if index[high] == 0 {
                            // redundant vertex
                            index.insert(node.clone(), index[low]);
                        } else {
                            q.push(((index[low], index[high]), node));
                        }
                    }
                }
            }
            q.sort_unstable_by_key(|(k, _)| *k);
            let mut old_key: (usize, usize) = (usize::MAX, usize::MAX);
            for (key, n) in q.iter().cloned() {
                if key == old_key {
                    index.insert(n.clone(), next_id);
                } else {
                    next_id += 1;
                    match **n {
                        Vertex::Bool(_) => {
                            index.insert(n.clone(), next_id);
                            node.insert(next_id, n.clone());
                        }
                        Vertex::Var {
                            var_index,
                            ref low,
                            ref high,
                        } => {
                            let nn = Node::new_var(
                                var_index,
                                node[&index[low]].clone(),
                                node[&index[high]].clone(),
                            );
                            index.insert(n.clone(), next_id);
                            index.insert(nn.clone(), next_id);
                            node.insert(next_id, nn);
                        }
                    }
                    old_key = key;
                }
            }
        }
        // pick up a tree from the hash-table
        self.graph = node[&next_id].clone();
    }
    fn apply(&self, _op: Box<dyn Fn(bool, bool) -> bool>, _unit: bool, _other: &Self) -> Self {
        unimplemented!()
    }
    fn compose(&self, _other: &Self, _at: usize) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        node::{example, Node},
        types::{DecisionDiagram, DecisionDiagramNode},
        zdd::ZDD,
    };

    #[test]
    fn test_satisfy_one() {
        let f = Node::new_constant(false);
        assert_eq!(ZDD::new_from(f.clone()).satisfy_one(), false);
        let ff: Node = Node::new_var(2, f.clone(), f.clone());
        let zdd: ZDD<Node> = ZDD::new_from(ff);
        assert_eq!(zdd.satisfy_one(), false);
        let major = ZDD::new_from(example::majority());
        assert_eq!(major.satisfy_one(), true);
    }
    #[test]
    fn test_satisfy_all() {
        let major = ZDD::new_from(example::majority());
        assert_eq!(major.satisfy_one(), true);
        assert_eq!(major.satisfy_all(), 3);
        let ind = ZDD::new_from(example::independent_set());
        assert_eq!(ind.satisfy_one(), true);
        assert_eq!(ind.satisfy_all(), 18);
    }
}
