use {
    ddir::{
        bdd::BDD,
        node::{example, Node},
        types::{DecisionDiagram, DecisionDiagramNode, ReducedDecisionDiagram},
        zdd::ZDD,
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
    let f = dump!(Node::new_constant(false), "false-node.gv");
    dump!(BDD::new_from(f.clone()), "false-bdd.gv");
    let independ: Node = dump!(example::independent_set(), "ind-tree.gv");
    dump!(BDD::new_from(independ.clone()), "ind-bdd.gv");
    dump!(ZDD::new_from(independ), "ind-zdd.gv");

    let kernels: Node = dump!(example::kernels(), "krn-tree.gv");
    dump!(BDD::new_from(kernels.clone()), "krn-bdd.gv");
    dump!(ZDD::new_from(kernels), "krn-zdd.gv");

    let majority: Node = dump!(example::majority(), "maj-tree.gv");
    dump!(BDD::new_from(majority.clone()), "maj-bdd.gv");
    dump!(ZDD::new_from(majority), "maj-zdd.gv");

    let x1x3: BDD<Node> = dump!(BDD::new_from(example::x1x3()), "x1x3-bdd.gv");
    let x2x3: BDD<Node> = dump!(BDD::new_from(example::x2x3()), "x2x3-bdd.gv");
    dump!(
        x1x3.apply(Box::new(|a, b| a | b), true, &x2x3),
        "apply-bdd.gv"
    );
    let x1x2x4: BDD<Node> = dump!(BDD::new_from(example::x1x2x4()), "x1x2x4-bdd.gv");
    dump!(x1x2x4.compose(&x2x3, 2), "composed.gv");
}
