use {
    crate::node::Node,
    std::{collections::HashSet, io},
};

pub trait ReducedDecisionDiagram {
    /// convert the current graph to one which is a reduced diagram
    fn reduce(&mut self);
    /// return a new graph generated by apply `op` to this and the other graph
    fn apply(&self, op: Box<dyn Fn(bool, bool) -> bool>, unit: bool, _other: &Self) -> Self;
}

pub trait DecisionDiagram {
    // return the hashset of all (non)terminal nodes in graph.
    fn all_nodes(&self) -> HashSet<&Node>;
    // return the number of (non)terminal nodes in graph.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    // write the graph in graphvis format
    fn write_as_gv(&self, sink: impl io::Write) -> io::Result<()>;
}
