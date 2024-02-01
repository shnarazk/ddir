use {
    ddir::{
        bdd::ToBinaryDecisionDiagram,
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
}
