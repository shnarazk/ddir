use {
    ddir::{
        bdd::{BinaryDecisionDiagram, ToBinaryDecisionDiagram, BDD},
        dd::{example, DecisionDiagramTrait, DDT},
    },
    std::fs::File,
};

fn main() {
    // let stdout = io::stdout();
    let idp_set: DDT = example::independent_set();
    assert_eq!(idp_set.len(), 127);
    let f1 = File::create("ind-tree.gv").expect("fail to create");
    idp_set.write_as_gv(f1).expect("fail to serialize");
    let f2 = File::create("ind-bdd.gv").expect("fail to create");
    idp_set.to_bdd().write_as_gv(f2).expect("fail to serialize");

    let majority1: DDT = example::majority();
    let f3 = File::create("maj-tree.gv").expect("fail to create");
    majority1.write_as_gv(f3).expect("fail to serialize");
    let f4 = File::create("maj-bdd.gv").expect("fail to create");
    majority1
        .to_bdd()
        .write_as_gv(f4)
        .expect("fail to serialize");

    let x1x3: BDD = example::x1x3().to_bdd();
    let x1x3f = File::create("x1x2-bdd.gv").expect("fail to create");
    x1x3.write_as_gv(x1x3f).expect("fail to save");
    let x2x3: BDD = example::x2x3().to_bdd();
    let x2x3f = File::create("x2x3-bdd.gv").expect("fail to create");
    x2x3.write_as_gv(x2x3f).expect("fail to save");
    let applied: BDD = x1x3.apply(Box::new(|a, b| a | b), true, &x2x3);
    let applyf = File::create("apply-bdd.gv").expect("fail to create");
    applied.write_as_gv(applyf).expect("fail to save");
}
