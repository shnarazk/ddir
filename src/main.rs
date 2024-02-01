use {
    ddir::{
        bdd::{ToBinaryDecisionDiagram, BDD},
        dd::{sample1, DecisionDiagramTrait, DDT},
    },
    std::io,
};

fn main() {
    let stdout = io::stdout();
    // let f = Node::new_constant(false);
    // let n = Node::new_var(2, f.clone(), f.clone());
    // let k = Node::new_var(1, n.clone(), f.clone());

    // k.write_as_graphvis(stdout).expect("");
    let idp: DDT = sample1();
    // let idp: BDD = sample1().to_bdd();
    idp.write_as_graphvis(stdout).expect("");
}
