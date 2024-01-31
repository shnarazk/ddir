use {
    crate::dd::{DecisionDiagramTrait, Node, DDT},
    std::{collections::HashSet, io, marker::PhantomData},
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

pub trait BinaryDecisionDiagram {
    fn reduce(&self);
}

impl BinaryDecisionDiagram for BDD {
    fn reduce(&self) {
        todo!()
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
        let bdd = BDD {
            graph: self.graph.clone(),
            ..Default::default()
        };
        bdd.reduce();
        bdd
    }
}

impl ToBinaryDecisionDiagram for Node {
    fn to_bdd(&self) -> BDD {
        let bdd = BDD {
            graph: self.clone(),
            ..Default::default()
        };
        bdd.reduce();
        bdd
    }
}
