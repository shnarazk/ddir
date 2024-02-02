use {
    crate::{dd::DecisionDiagramTrait, node::Node},
    std::{collections::HashSet, io},
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct DDT {
    pub(crate) graph: Node,
}

impl DecisionDiagramTrait for DDT {
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

pub mod example {
    use {
        super::*,
        crate::node::{DecisionDiagramNode, Node},
    };

    macro_rules! F {
        () => {
            Node::new_constant(false)
        };
    }
    macro_rules! T {
        () => {
            Node::new_constant(true)
        };
    }
    macro_rules! D {
        ($v: expr, $l: expr, $h: expr) => {
            Node::new_var($v, $l, $h)
        };
    }
    /// return the independent sets of 6 cyclic chain
    pub fn independent_set() -> DDT {
        DDT {
            graph: D!(
                1, //                                 1 -> {
                D!(
                    2, //                               (2 -> {
                    D!(
                        3, //                             (3 -> {
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), T!()), //           (6 -> {true. true}).
                                D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), T!()), //           (6 -> {true. true}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                            )
                        ),
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), T!()), //           (6 -> {true. true}).
                                D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }).
                            )
                        )
                    ),
                    D!(
                        3, //                             (3 -> {
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), T!()), //           (6 -> {true. true}).
                                D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), T!()), //           (6 -> {true. true}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                            )
                        ),
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }) }),
                            )
                        )
                    )
                ),
                D!(
                    2, //                               (2 -> {
                    D!(
                        3, //                             (3 -> {
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), F!()), //           (6 -> {true. false}).
                                D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), F!()), //           (6 -> {true. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                            )
                        ),
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, T!(), F!()), //           (6 -> {true. false}).
                                D!(6, T!(), F!())  //           (6 -> {true. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }).
                            )
                        )
                    ),
                    D!(
                        3, //                             (3 -> {
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }).
                            )
                        ),
                        D!(
                            4, //                           (4 -> {
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }).
                            ),
                            D!(
                                5,                 //         (5 -> {
                                D!(6, F!(), F!()), //           (6 -> {false. false}).
                                D!(6, F!(), F!())  //           (6 -> {false. false}) }) }) }) }) })
                            )
                        )
                    )
                )
            ),
        }
    }

    /// majority
    pub fn majority() -> DDT {
        DDT {
            graph: D!(
                1,
                D!(2, F!(), D!(3, F!(), T!())),
                D!(2, D!(3, F!(), T!()), T!())
            ),
        }
    }
    // From Figure 7 of Randal E. Bryant, Graph-Based Algorithms for Boolean
    // Function Manipulation, IEEE Trans. en Comp., C-35-8, pp.677-691, Aug. 1986.
    pub fn x1x3() -> DDT {
        DDT {
            graph: D!(1, T!(), D!(3, T!(), F!())),
        }
    }
    // From Figure 7 of Randal E. Bryant, Graph-Based Algorithms for Boolean
    // Function Manipulation, IEEE Trans. en Comp., C-35-8, pp.677-691, Aug. 1986.
    pub fn x2x3() -> DDT {
        DDT {
            graph: D!(2, F!(), D!(3, F!(), T!())),
        }
    }
}
