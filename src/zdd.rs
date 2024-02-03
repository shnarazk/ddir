//! Zero-suppressed Decision Diagram

use {
    crate::{
        node::Node,
        types::{DecisionDiagram, DecisionDiagramNode, ReducedDecisionDiagram},
    },
    std::{collections::HashSet, io, marker::PhantomData},
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
        todo!()
    }
    fn apply(&self, _op: Box<dyn Fn(bool, bool) -> bool>, _unit: bool, _other: &Self) -> Self {
        todo!()
    }
}
