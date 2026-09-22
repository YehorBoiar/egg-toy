mod serialiser;
use std::{fmt::Display, str::FromStr};

use egg::{
    AstSize, Extractor, FromOp, Id, Language, RecExpr, Rewrite, Runner, RunnerResult, Symbol,
    rewrite,
};
use serialiser::egg_to_serialized_egraph;

#[derive(Debug, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Math {
    Num(i32),
    Symbol(Symbol),
    Add([Id; 2]),
    Mul([Id; 2]),
    Div([Id; 2]),
    Shl([Id; 2]),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum MathDiscriminant {
    Num,
    Symbol,
    Add(),
    Mul(),
    Div(),
    Shl(),
}

impl Language for Math {
    type Discriminant = MathDiscriminant;

    fn discriminant(&self) -> Self::Discriminant {
        match self {
            Math::Num(_) => MathDiscriminant::Num,
            Math::Symbol(_) => MathDiscriminant::Symbol,
            Math::Add(_) => MathDiscriminant::Add(),
            Math::Mul(_) => MathDiscriminant::Mul(),
            Math::Div(_) => MathDiscriminant::Div(),
            Math::Shl(_) => MathDiscriminant::Shl(),
        }
    }

    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Math::Num(a), Math::Num(b)) => a == b,
            (Math::Symbol(a), Math::Symbol(b)) => a == b,
            _ => self.discriminant() == other.discriminant(),
        }
    }

    fn children(&self) -> &[Id] {
        match self {
            Math::Num(_) | Math::Symbol(_) => &[],
            Math::Add(ids) => ids,
            Math::Mul(ids) => ids,
            Math::Div(ids) => ids,
            Math::Shl(ids) => ids,
        }
    }

    fn children_mut(&mut self) -> &mut [Id] {
        match self {
            Math::Num(_) | Math::Symbol(_) => &mut [],
            Math::Add(ids) => ids,
            Math::Mul(ids) => ids,
            Math::Div(ids) => ids,
            Math::Shl(ids) => ids,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseMathError;

impl FromOp for Math {
    type Error = ParseMathError;

    fn from_op(op: &str, children: Vec<Id>) -> Result<Self, Self::Error> {
        match (op, children.len()) {
            ("+", 2) => Ok(Math::Add([children[0], children[1]])),
            ("*", 2) => Ok(Math::Mul([children[0], children[1]])),
            ("/", 2) => Ok(Math::Div([children[0], children[1]])),
            (s, 0) => {
                if let Ok(n) = s.parse::<i32>() {
                    Ok(Math::Num(n))
                } else {
                    Ok(Math::Symbol(Symbol::from(s)))
                }
            }

            _ => Err(ParseMathError),
        }
    }
}

impl Display for Math {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Math::Num(num) => write!(f, "{}", num),
            Math::Symbol(global_symbol) => write!(f, "{}", global_symbol),
            Math::Add(_) => write!(f, "Add"),
            Math::Mul(_) => write!(f, "Mul"),
            Math::Div(_) => write!(f, "Div"),
            Math::Shl(_) => write!(f, "Shl"),
        }
    }
}

/// this right now produces json files for each iteration of the e-graph
/// we can visualise them in here https://egraphs-good.github.io/egraph-visualizer/
pub fn main() {
    env_logger::init();

    let rules: &[Rewrite<Math, ()>] = &[
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mult-zero";   "(* ?a 0)"  => "0"),
        // rewrite!("mult-one";   "?a"  => "(* ?a 1)"), // Why does it reduce to 0?
        rewrite!("add-zero";    "(+ ?a 0)"  => "?a"),
        rewrite!("factor";     "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
    ];

    // ((a * c) + (a * d)) + ((b * c) + (b * d))
    let expr: RecExpr<Math> = vec![
        Math::Symbol("a".into()),          // 0
        Math::Symbol("c".into()),          // 1
        Math::Mul([0.into(), 1.into()]),   // 2: a * c
        Math::Symbol("a".into()),          // 3
        Math::Symbol("d".into()),          // 4
        Math::Mul([3.into(), 4.into()]),   // 5: a * d
        Math::Add([2.into(), 5.into()]),   // 6: (a * c) + (a * d)
        Math::Symbol("b".into()),          // 7
        Math::Symbol("c".into()),          // 8
        Math::Mul([7.into(), 8.into()]),   // 9: b * c
        Math::Symbol("b".into()),          // 10
        Math::Symbol("d".into()),          // 11
        Math::Mul([10.into(), 11.into()]), // 12: b * d
        Math::Add([9.into(), 12.into()]),  // 13: (b * c) + (b * d)
        Math::Add([6.into(), 13.into()]),  // 14: ((a * c) + (a * d)) + ((b * c) + (b * d))
    ]
    .into();

    let runner: Runner<Math, ()> = Runner::default()
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

    let extractor = Extractor::new(&egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);
    let dummy_runner: Runner<Math, ()> = Runner::default().with_expr(&best).run(&[]);

    let serialised = egg_to_serialized_egraph(&dummy_runner.egraph);
    serialised.to_json_file("filename.json").unwrap();

    println!("{:?}", best);
    println!("{}", best_cost);
}
