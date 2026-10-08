mod language_definition;
mod my_cost_function;
mod post_extraction_functions;
mod serialiser;

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use egg::{
    Analysis, Applier, AstSize, CostFunction, ENodeOrVar::Var, Extractor, FromOp, Id, Language,
    PatternAst, RecExpr, Rewrite, Runner, RunnerResult, Subst, Symbol, rewrite,
};

use serialiser::egg_to_serialized_egraph;

use crate::{
    language_definition::{EGraph, Math},
    post_extraction_functions::{bubble_up, div_to_bubble, expand_bubble, populate_auxiliary_division},
};
use crate::my_cost_function::SillyCostFn;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct IntRange {
    min_value: i32,
    max_value: i32,
}

#[derive(Default, Clone, Debug)]
struct IntervalAnalysis {
    domains: HashMap<Symbol, IntRange>,
}

impl Analysis<Math> for IntervalAnalysis {
    type Data = Option<IntRange>;

    fn make(egraph: &mut egg::EGraph<Math, Self>, enode: &Math, id: Id) -> Self::Data {
        match enode {
            Math::Num(n) => Some(IntRange {
                min_value: *n,
                max_value: *n,
            }),

            Math::Symbol(sym) => {
                if let Some(range) = egraph.analysis.domains.get(sym) {
                    println!("{:?}, {:?}", sym, range);
                    Some(range.clone())
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> egg::DidMerge {
        egg::DidMerge(true, true)
    }
}

/// this right now produces json files for each iteration of the e-graph
/// we can visualise them in here:
/// https://egraphs-good.github.io/egraph-visualizer/
pub fn main() {
    env_logger::init();

    let rules: &[Rewrite<Math, IntervalAnalysis>] = &[
        // rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        // rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        // rewrite!("mult-zero";   "(* ?a 0)"  => "0"),
        // // rewrite!("mult-one";   "?a"  => "(* ?a 1)"), // Why does it reduce to 0?
        // rewrite!("add-zero";    "(+ ?a 0)"  => "?a"),
        // rewrite!(
        //     "factor";
        //     "(+ (* ?a ?b) (* ?a ?c))" =>
        //     "(* ?a (+ ?b ?c))"
        // ),
        // // would need to add domain in future where x has a domain
        // // a new name has to appear
        // rewrite!("div-to-aux"; "(/ ?x ?y)" => "(auxdiv ?x ?y)"),
        // // (a + c*x) / (c*y) ~~> a / (c*y) + x / y
        // rewrite!(
        //     "split-div";
        //     "(/ (+ ?a (* ?c ?x)) (* ?c ?y))" =>
        //     "(+ (/ ?a (* ?c ?y)) (/ ?x ?y))"
        // ),
    ];

    let expr: RecExpr<Math> = vec![
        Math::Symbol("x".into()),
        Math::Symbol("y".into()),
        Math::Div([0.into(), 1.into()]),
        Math::Symbol("z".into()),
        Math::Eq([2.into(), 3.into()]),
        Math::Not([4.into()]),
    ]
    .into();

    let mut domains = HashMap::new();
    // 2. Put your domains into it using the variable names
    domains.insert(
        Symbol::from("x"),
        IntRange {
            min_value: -4,
            max_value: 4,
        },
    );
    domains.insert(
        Symbol::from("y"),
        IntRange {
            min_value: -4,
            max_value: 4,
        },
    );
    domains.insert(
        Symbol::from("z"),
        IntRange {
            min_value: 1,
            max_value: 2,
        },
    );

    // 3. Package the HashMap inside your custom Analysis struct
    let analysis: IntervalAnalysis = IntervalAnalysis { domains: domains };

    let runner: Runner<Math, IntervalAnalysis> = Runner::new(analysis.clone())
        .with_expr(&expr)
        .with_hook(|runner| {
            let serialised = egg_to_serialized_egraph(&runner.egraph);
            let iterations_done = &runner.iterations.len();
            let filename = format!("iteration_{}.json", iterations_done);

            serialised.to_json_file(filename).unwrap();

            println!("Egraph is this big: {}", runner.egraph.total_size());

            Ok(())
        })
        .run(rules);

    let (egraph, root) = (runner.egraph, runner.roots[0]);

    let serialised = egg_to_serialized_egraph(&egraph);
    serialised.to_json_file("saturated.json").unwrap();

    let extractor = Extractor::new(&egraph, SillyCostFn { egraph: &egraph });

    let (best_cost, best) = extractor.find_best(root);

    println!("{:?}", analysis);

    let dummy_run = div_to_bubble(best);

    let dummy_runner: Runner<Math, ()> = Runner::default().with_expr(&dummy_run).run(&[]);

    let serialised = egg_to_serialized_egraph(&dummy_runner.egraph);
    serialised.to_json_file("filename.json").unwrap();

    /*
    we've got a result that says this
    we basically made aux variable to rewrite everything
    (
        ((v_1 < b) /\ (c + v_0 < d))
        /\ (v_0 * y == x)
    )
    /\ (v_1 * (3 * y) == (a * 2) + (x * 3))
     */
    println!("{:?}", dummy_run);
    println!("{}", best_cost);
}

// ((a * c) + (a * d)) + ((b * c) + (b * d))
// let expr: RecExpr<Math> = vec![
//     Math::Symbol("a".into()),          // 0
//     Math::Symbol("c".into()),          // 1
//     Math::Mul([0.into(), 1.into()]),   // 2: a * c
//     Math::Symbol("a".into()),          // 3
//     Math::Symbol("d".into()),          // 4
//     Math::Mul([3.into(), 4.into()]),   // 5: a * d
//     Math::Add([2.into(), 5.into()]),   // 6: (a * c) + (a * d)
//     Math::Symbol("b".into()),          // 7
//     Math::Symbol("c".into()),          // 8
//     Math::Mul([7.into(), 8.into()]),   // 9: b * c
//     Math::Symbol("b".into()),          // 10
//     Math::Symbol("d".into()),          // 11
//     Math::Mul([10.into(), 11.into()]), // 12: b * d
//     Math::Add([9.into(), 12.into()]),  // 13: (b * c) + (b * d)
//     Math::Add([6.into(), 13.into()]),  // 14: ((a * c) + (a * d)) + ((b * c) + (b * d))
// ]
// .into();

// let expr: RecExpr<Math> = vec![
//     Math::Symbol("a".into()),          // 0
//     Math::Num(2),                      // 1
//     Math::Mul([0.into(), 1.into()]),   // 2: 2a
//     Math::Symbol("x".into()),          // 3
//     Math::Num(3),                      // 4
//     Math::Mul([4.into(), 3.into()]),   // 5: 3x
//     Math::Add([2.into(), 5.into()]),   // 6: 2a + 3x
//     Math::Num(3),                      // 7
//     Math::Symbol("y".into()),          // 8
//     Math::Mul([7.into(), 8.into()]),   // 9: 3y
//     Math::Div([6.into(), 9.into()]),   // 10: (2a + 3x) / 3y
//     Math::Symbol("b".into()),          // 11
//     Math::Lt([10.into(), 11.into()]),  // 12: (2a + 3x)/3y < b
//     Math::Symbol("c".into()),          // 13
//     Math::Symbol("x".into()),          // 14
//     Math::Symbol("y".into()),          // 15
//     Math::Div([14.into(), 15.into()]), // 16: x/y
//     Math::Add([13.into(), 16.into()]), // 17: c + x/y
//     Math::Symbol("d".into()),          // 18
//     Math::Lt([17.into(), 18.into()]),  // 19: c + x/y < d
//     Math::And([12.into(), 19.into()]), // 20: first inequality /\ second inequality
// ]
// .into();
