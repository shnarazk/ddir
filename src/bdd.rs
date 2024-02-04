//! Binary Decision Diagram
use {
    crate::{
        node::{Node, Vertex},
        types::{
            BooleanOperator, DecisionDiagram, DecisionDiagramNode, Indexer, ReducedDecisionDiagram,
        },
    },
    itertools::Itertools,
    std::{
        collections::{HashMap, HashSet},
        io,
        marker::PhantomData,
    },
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BDD<N: DecisionDiagramNode> {
    graph: N,
    phantom: PhantomData<()>,
}

impl BDD<Node> {
    pub fn new_from(graph: Node) -> Self {
        let mut bdd = BDD {
            graph: graph.clone(),
            ..Default::default()
        };
        bdd.reduce();
        bdd
    }
}

impl<N: DecisionDiagram<N> + DecisionDiagramNode> DecisionDiagram<N> for BDD<N> {
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

impl ReducedDecisionDiagram for BDD<Node> {
    // convert tree to BDD
    fn reduce(&mut self) {
        let root = &self.graph;
        let (mut index, mut node) = Node::build_indexer(&[root.clone()]);
        let mut vlist: HashMap<usize, Vec<&Node>> = HashMap::new();
        // put each vertex u on list vlist[u.var_index]
        for n in root.all_nodes().iter().cloned() {
            if let Some(vi) = n.var_index() {
                vlist.entry(vi).or_default().push(n);
            }
        }
        let mut next_id: usize = index.len();
        for vi in vlist.keys().sorted().rev() {
            let mut q: Vec<((usize, usize), &Node)> = Vec::new();
            for n in vlist[vi].iter().cloned() {
                match **n {
                    Vertex::Bool(_) => (),
                    Vertex::Var {
                        ref low, ref high, ..
                    } => {
                        if index[low] == index[high] {
                            // redundant vertex
                            index.insert(n.clone(), index[low]);
                        } else {
                            q.push(((index[low], index[high]), n));
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
    fn apply(&self, op: Box<dyn Fn(bool, bool) -> bool>, unit: bool, other: &Self) -> BDD<Node> {
        fn aux(
            operator @ (op, unit): &BooleanOperator,
            (v1, v2): (Node, Node),
            indexer @ (index, node): &Indexer<Node>,
            evaluation: &mut HashMap<Node, bool>,
            merged: &mut HashMap<(usize, usize), Node>,
        ) -> Node {
            let hash_key = (index[&v1], index[&v2]);
            if let Some(n) = merged.get(&hash_key) {
                return n.clone(); // have already evaluaten
            }
            let value1 = evaluation.get(&v1);
            let value2 = evaluation.get(&v2);
            let value = match (value1, value2) {
                (Some(a), _) if *a == *unit => Some(*unit),
                (_, Some(b)) if *b == *unit => Some(*unit),
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some(op(*a, *b)),
            };
            if let Some(b) = value {
                return node[&(b as usize)].clone();
            }
            let v1key = v1.unified_key();
            let v2key = v2.unified_key();
            let key = match (v1key < 2, v2key < 2) {
                (false, false) => v1key.min(v2key),
                (false, true) => v1key,
                (true, false) => v2key,
                (true, true) => op(v1key == 1, v2key == 1) as usize,
            };
            let u = if key < 2 {
                Node::new_constant(key == 1)
            } else {
                let (vlow1, vhigh1) = if v1key == key {
                    (v1.low().unwrap().clone(), v1.high().unwrap().clone())
                } else {
                    (v1.clone(), v1.clone())
                };
                let (vlow2, vhigh2) = if v2key == key {
                    (v2.low().unwrap().clone(), v2.high().unwrap().clone())
                } else {
                    (v2.clone(), v2.clone())
                };
                Node::new_var(
                    key - 2,
                    aux(operator, (vlow1, vlow2), indexer, evaluation, merged),
                    aux(operator, (vhigh1, vhigh2), indexer, evaluation, merged),
                )
            };
            if let Some(b) = value {
                evaluation.insert(u.clone(), b);
            }
            merged.insert(hash_key, u.clone());
            u
        }
        // mapping from index pair to index
        let mut merged: HashMap<(usize, usize), Node> = HashMap::new();
        // mapping from node to bool
        let mut evaluation: HashMap<Node, bool> = HashMap::new();
        let mut applied = BDD {
            graph: aux(
                &(op, unit),
                (self.graph.clone(), other.graph.clone()),
                &Node::build_indexer(&[self.graph.clone(), other.graph.clone()]),
                &mut evaluation,
                &mut merged,
            ),
            ..Default::default()
        };
        applied.reduce();
        applied
    }
    /// return a new diagram by composing this and other
    fn compose(&self, other: &Self, var_index: usize) -> Self {
        let v1 = self.graph.clone();
        let v2 = other.graph.clone();
        let mut node: HashMap<usize, Node> = HashMap::new();
        node.insert(0, Node::new_constant(false));
        node.insert(1, Node::new_constant(true));
        let mut index: HashMap<Node, usize> = HashMap::new();
        for (i, n) in self
            .graph
            .all_nodes()
            .iter()
            .chain(other.graph.all_nodes().iter())
            .enumerate()
        {
            node.insert(i + 2, (*n).clone());
            if let Some(b) = n.is_constant() {
                index.insert((*n).clone(), b as usize);
            } else {
                index.insert((*n).clone(), i + 2);
            }
        }
        let mut links: HashMap<(usize, usize, usize), Node> = HashMap::new();
        let mut values: HashMap<Node, bool> = HashMap::new();
        BDD::new_from(compose_aux(
            &v1,
            &v1,
            &v2,
            var_index,
            &mut (index, node),
            &mut links,
            &mut values,
        ))
    }
}

fn compose_aux(
    low: &Node,
    high: &Node,
    other: &Node,
    control: usize,
    (index, node): &mut Indexer<Node>,
    links: &mut HashMap<(usize, usize, usize), Node>,
    values: &mut HashMap<Node, bool>,
) -> Node {
    // let nodes = vec![low, high, other];
    let hash_key = (index[low], index[high], index[other]);
    let vlow1 = if low.var_index() == Some(control) {
        low.low().unwrap()
    } else {
        low
    };
    let vhigh1 = if high.var_index() == Some(control) {
        high.high().unwrap()
    } else {
        high
    };
    if let Some(evaluated) = links.get(&hash_key) {
        return evaluated.clone();
    }
    if let (Some(bl), Some(bh), Some(b2)) =
        (values.get(vlow1), values.get(vhigh1), values.get(other))
    {
        let val = ((!b2) & bl) | (b2 & bh);
        links.insert(hash_key, node[&(val as usize)].clone());
        todo!()
    } else {
        todo!()
    };
}

#[cfg(test)]
mod test {
    use crate::{
        bdd::BDD,
        node::Node,
        types::{DecisionDiagram, DecisionDiagramNode},
    };

    #[test]
    fn test() {
        let f = Node::new_constant(false);
        let n: Node = Node::new_var(2, f.clone(), f.clone());
        let bdd: BDD<Node> = BDD::new_from(n);
        assert_eq!(bdd.len(), 1);
    }
}
