use {
    ddir::{
        bdd::{ToBinaryDecisionDiagram, BDD},
        dd::{example1, example2, DecisionDiagramTrait, DDT},
    },
    std::fs::File,
};

fn main() {
    // let stdout = io::stdout();
    let idp1: DDT = example1();
    assert_eq!(idp1.len(), 127);
    let f1 = File::create("ind-tree.gv").expect("fail to create");
    idp1.write_as_gv(f1).expect("fail to serialize");

    let idp2: BDD = example1().to_bdd();
    let f2 = File::create("ind-bdd.gv").expect("fail to create");
    idp2.write_as_gv(f2).expect("fail to serialize");

    let majority1: DDT = example2();
    let f3 = File::create("maj-tree.gv").expect("fail to create");
    majority1.write_as_gv(f3).expect("fail to serialize");
    let f4 = File::create("maj-bdd.gv").expect("fail to create");
    majority1
        .to_bdd()
        .write_as_gv(f4)
        .expect("fail to serialize");
}
