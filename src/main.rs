use {
    ddir::{
        bdd::{ToBinaryDecisionDiagram, BDD},
        dd::{sample1, DecisionDiagramTrait, DDT},
    },
    std::fs::File,
};

fn main() {
    // let stdout = io::stdout();
    let idp1: DDT = sample1();
    assert_eq!(idp1.len(), 127);
    let f1 = File::create("ind-tree.gv").expect("fail to create");
    idp1.write_as_gv(f1).expect("fail to serialize");

    let idp2: BDD = sample1().to_bdd();
    let f2 = File::create("ind-bdd.gv").expect("fail to create");
    idp2.write_as_gv(f2).expect("fail to serialize")
}
