use {
    crate::dd::{DecisionDiagramTrait, Node, DDT},
    std::{collections::HashSet, io, marker::PhantomData},
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BDD {
    graph: DDT,
    phantom: PhantomData<()>,
}

impl DecisionDiagramTrait for BDD {
    fn new_constant(b: bool) -> Self {
        Self {
            graph: DDT::new_constant(b),
            ..Default::default()
        }
    }
    fn new_var(var_index: usize, low: Node, high: Node) -> Self {
        Self {
            graph: DDT::new_var(var_index, low, high),
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
    fn to_bdd(&self) -> BDD {
        self.clone()
    }
}

impl BDD {
    pub fn new_from(graph: DDT) -> BDD {
        Self {
            graph,
            ..Default::default()
        }
    }
}
