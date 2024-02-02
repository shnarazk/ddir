use {
    crate::{
        dd::{DecisionDiagramTrait, ReducedDecisionDiagram},
        ddt::DDT,
        node::{DecisionDiagramNode, Node, Vertex},
    },
    itertools::Itertools,
    std::{
        collections::{HashMap, HashSet},
        io,
        marker::PhantomData,
    },
};

type BooleanOperator = (Box<dyn Fn(bool, bool) -> bool>, bool);

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BDD {
    graph: Node,
    phantom: PhantomData<()>,
}

impl DecisionDiagramTrait for BDD {
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

impl ReducedDecisionDiagram for BDD {
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
    fn apply(&self, op: Box<dyn Fn(bool, bool) -> bool>, unit: bool, other: &Self) -> BDD {
        let mut from_index: HashMap<usize, Node> = HashMap::new();
        from_index.insert(0, Node::new_constant(false));
        from_index.insert(1, Node::new_constant(true));
        let mut to_index: HashMap<Node, usize> = HashMap::new();
        for (i, node) in self
            .graph
            .all_nodes()
            .iter()
            .chain(other.graph.all_nodes().iter())
            .enumerate()
        {
            from_index.insert(i + 2, (*node).clone());
            if let Some(b) = node.is_constant() {
                to_index.insert((*node).clone(), b as usize);
            } else {
                to_index.insert((*node).clone(), i + 2);
            }
        }
        // mapping from index pair to index
        let mut merged: HashMap<(usize, usize), Node> = HashMap::new();
        // mapping from node to bool
        let mut evaluation: HashMap<Node, bool> = HashMap::new();
        // | unit hashKey value1 value2 value u |
        //   hashKey := (to_index at: v1) @ (to_index at: v2).
        //   merged at: hashKey ifPresent: [ :node | node ifNotNil: [ ^ node "have already evaluated" ] ].
        //   value1 := evaluation at: v1 ifAbsent: [ nil ].
        //   value2 := evaluation at: v2 ifAbsent: [ nil ].
        //   "u.value := v1.value <op> v2.value"
        //   value := (value1 = unit or: [ value2 = unit ]) ifTrue: [ unit ]
        //                ifFalse: [
        //                value1 ifNotNil: [ value2 ifNotNil: [ operator value: value1 value: value2 ] ] ].
        //   u := value ifNil: [
        //            | v1Index v2Index vlow1 vlow2 vhigh1 vhigh2 w |
        //            "create a nonterminal vertex and evaluate further down"
        //            w := DDNode new.
        //            evaluation at: w put: value.
        //            v1Index := v1 isLiteral ifTrue: [ v1 ] ifFalse: [ v1 varIndex ].
        //            v2Index := v2 isLiteral ifTrue: [ v2 ] ifFalse: [ v2 varIndex ].
        //            w varIndex: (v1 isLiteral ifTrue: [
        //                     v2 isLiteral ifTrue: [ operator value: v1 value: v2 ] ifFalse: [ v2Index ] ]
        //                     ifFalse: [
        //                     v2 isLiteral ifTrue: [ v1Index ] ifFalse: [ v1 varIndex min: v2 varIndex ] ]).
        //            vlow1 := v1Index = w varIndex ifTrue: [ v1 low ] ifFalse: [ v1 ].
        //            vlow2 := v2Index = w varIndex ifTrue: [ v2 low ] ifFalse: [ v2 ].
        //            vhigh1 := v1Index = w varIndex ifTrue: [ v1 high ] ifFalse: [ v1 ].
        //            vhigh2 := v2Index = w varIndex ifTrue: [ v2 high ] ifFalse: [ v2 ].
        //            w low: (self apply: operator
        //                     on: vlow1
        //                     and: vlow2).
        //            w high: (self apply: operator
        //                     on: vhigh1
        //                     and: vhigh2).
        //            w ].
        //   aMapping at: hashKey put: u.
        //   ^ u
        fn aux(
            operator @ (op, unit): &BooleanOperator,
            (v1, v2): (Node, Node),
            (to_index, from_index): (&mut HashMap<Node, usize>, &mut HashMap<usize, Node>),
            evaluation: &mut HashMap<Node, bool>,
            merged: &mut HashMap<(usize, usize), Node>,
        ) -> Node {
            let hash_key = (*to_index.get(&v1).unwrap(), *to_index.get(&v2).unwrap());
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
                return from_index.get(&(b as usize)).unwrap().clone();
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
                    aux(
                        operator,
                        (vlow1, vlow2),
                        (to_index, from_index),
                        evaluation,
                        merged,
                    ),
                    aux(
                        operator,
                        (vhigh1, vhigh2),
                        (to_index, from_index),
                        evaluation,
                        merged,
                    ),
                )
            };
            if let Some(b) = value {
                evaluation.insert(u.clone(), b);
            }
            merged.insert(hash_key, u.clone());
            u
        }
        let mut applied = BDD {
            graph: aux(
                &(op, unit),
                (self.graph.clone(), other.graph.clone()),
                (&mut to_index, &mut from_index),
                &mut evaluation,
                &mut merged,
            ),
            ..Default::default()
        };
        applied.reduce();
        applied
    }
}

#[cfg(test)]
mod test {
    use crate::{
        bdd::{ToBinaryDecisionDiagram, BDD},
        dd::DecisionDiagramTrait,
        ddt::DDT,
        node::{DecisionDiagramNode, Node},
    };

    #[test]
    fn test() {
        let f = Node::new_constant(false);
        let n: Node = Node::new_var(2, f.clone(), f.clone());
        let tree: DDT = DDT { graph: n };
        let bdd: BDD = tree.to_bdd();
        assert_eq!(bdd.len(), 1);
    }
}
