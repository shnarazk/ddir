use {
    crate::{dd::DecisionDiagramTrait, node::Node},
    std::{collections::HashSet, io, marker::PhantomData},
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ZDD {
    graph: Node,
    phantom: PhantomData<()>,
}

impl DecisionDiagramTrait for ZDD {
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
