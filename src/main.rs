use {
    ddir::{
        bdd::BDD,
        node::{example, Node},
        types::{DecisionDiagram, ReducedDecisionDiagram},
    },
    std::fs::File,
};

macro_rules! dump {
    ($node: expr, $file: expr) => {{
        let node = $node;
        node.write_as_gv(File::create($file).expect(""))
            .expect("fail to serialize");
        node
    }};
}

fn main() {
    let idepend: Node = dump!(example::independent_set(), "ind-tree.gv");
    dump!(BDD::new_from(idepend), "ind-bdd.gv");
    let majority: Node = dump!(example::majority(), "maj-tree.gv");
    dump!(BDD::new_from(majority), "maj-bdd.gv");
    let x1x3: BDD<Node> = dump!(BDD::new_from(example::x1x3()), "x1x3-bdd.gv");
    let x2x3: BDD<Node> = dump!(BDD::new_from(example::x2x3()), "x2x3-bdd.gv");
    dump!(
        x1x3.apply(Box::new(|a, b| a | b), true, &x2x3),
        "apply-bdd.gv"
    );
}
