use {
    crate::{
        node::Node,
        types::{DecisionDiagram, ReducedDecisionDiagram},
    },
    std::{collections::HashSet, io, marker::PhantomData},
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ZDD {
    graph: Node,
    phantom: PhantomData<()>,
}

impl ZDD {
    pub fn new_from(graph: Node) -> ZDD {
        let mut zdd = ZDD {
            graph: graph.clone(),
            ..Default::default()
        };
        zdd.reduce();
        zdd
    }
}

impl DecisionDiagram for ZDD {
    type Element = Node;
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

impl ReducedDecisionDiagram for ZDD {
    fn reduce(&mut self) {
        todo!()
    }
    fn apply(&self, _op: Box<dyn Fn(bool, bool) -> bool>, _unit: bool, _other: &Self) -> Self {
        todo!()
    }
}
