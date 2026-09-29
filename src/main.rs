mod serialiser;
use std::{fmt::Display, str::FromStr};

use egg::{
    Applier, AstSize, CostFunction, Extractor, FromOp, Id, Language, PatternAst, RecExpr, Rewrite,
    Runner, RunnerResult, Subst, Symbol, Var, rewrite,
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
    // Represents an aux variable for the bubble rule described in example_equation
    AuxDiv([Id; 2]),
    And([Id; 2]),
    Lt([Id; 2]),
}

type EGraph = egg::EGraph<Math, ()>;

#[derive(Debug, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum MathDiscriminant {
    Num,
    Symbol,
    Add(),
    Mul(),
    Div(),
    Shl(),
    And(),
    Lt(),
    AuxDiv(),
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
            Math::AuxDiv(_) => MathDiscriminant::AuxDiv(),
            Math::And(_) => MathDiscriminant::And(),
            Math::Lt(_) => MathDiscriminant::Lt(),
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
            Math::Add(ids)
            | Math::Mul(ids)
            | Math::Div(ids)
            | Math::Shl(ids)
            | Math::AuxDiv(ids)
            | Math::And(ids)
            | Math::Lt(ids) => ids,
        }
    }

    fn children_mut(&mut self) -> &mut [Id] {
        match self {
            Math::Num(_) | Math::Symbol(_) => &mut [],
            Math::Add(ids)
            | Math::Mul(ids)
            | Math::Div(ids)
            | Math::Shl(ids)
            | Math::AuxDiv(ids)
            | Math::And(ids)
            | Math::Lt(ids) => ids,
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
            ("aux", 2) => Ok(Math::AuxDiv([children[0], children[1]])),
            ("and", 2) => Ok(Math::And([children[0], children[1]])),
            ("lt", 2) => Ok(Math::Lt([children[0], children[1]])),
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
            Math::AuxDiv(_) => write!(f, "AuxDiv"),
            Math::And(_) => write!(f, "And"),
            Math::Lt(_) => write!(f, "Lt"),
        }
    }
}

struct SillyCostFn;
impl CostFunction<Math> for SillyCostFn {
    type Cost = f64;
    fn cost<C>(&mut self, enode: &Math, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        let op_cost = match enode {
            Math::Div(_) => 100.1,
            Math::AuxDiv(_) => 1.1,
            _ => 1.1,
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}

/// this right now produces json files for each iteration of the e-graph
/// we can visualise them in here [https://egraphs-good.github.io/egraph-visualizer/](https://egraphs-good.github.io/egraph-visualizer/)
pub fn main() {
    env_logger::init();

    let rules: &[Rewrite<Math, ()>] = &[
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mult-zero";   "(* ?a 0)"  => "0"),
        // rewrite!("mult-one";   "?a"  => "(* ?a 1)"), // Why does it reduce to 0?
        rewrite!("add-zero";    "(+ ?a 0)"  => "?a"),
        rewrite!("factor";     "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

        // would need to add domain in future where x has a domain
        // a new name has to appear 
        rewrite!("div-to-aux"; "(/ ?x ?y)" => "(aux ?x ?y)"),
        // (a + c*x) / (c*y) ~~> a / (c*y) + x / y
        rewrite!("split-div";
            "(/ (+ ?a (* ?c ?x)) (* ?c ?y))" =>
            "(+ (/ ?a (* ?c ?y)) (/ ?x ?y))"
        ),
    ];

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

    let expr: RecExpr<Math> = vec![
        Math::Symbol("a".into()),          // 0
        Math::Num(2),                      // 1
        Math::Mul([1.into(), 0.into()]),   // 2: 2a
        Math::Symbol("x".into()),          // 3
        Math::Num(3),                      // 4
        Math::Mul([4.into(), 3.into()]),   // 5: 3x
        Math::Add([2.into(), 5.into()]),   // 6: 2a + 3x
        Math::Num(3),                      // 7
        Math::Symbol("y".into()),          // 8
        Math::Mul([7.into(), 8.into()]),   // 9: 3y
        Math::Div([6.into(), 9.into()]),   // 10: (2a + 3x) / 3y
        Math::Symbol("b".into()),          // 11
        Math::Lt([10.into(), 11.into()]),  // 12: (2a + 3x)/3y < b
        Math::Symbol("c".into()),          // 13
        Math::Symbol("x".into()),          // 14
        Math::Symbol("y".into()),          // 15
        Math::Div([14.into(), 15.into()]), // 16: x/y
        Math::Add([13.into(), 16.into()]), // 17: c + x/y
        Math::Symbol("d".into()),          // 18
        Math::Lt([17.into(), 18.into()]),  // 19: c + x/y < d
        Math::And([12.into(), 19.into()]), // 20: first inequality /\ second inequality
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

    let extractor = Extractor::new(&egraph, SillyCostFn);
    let (best_cost, best) = extractor.find_best(root);
    let dummy_runner: Runner<Math, ()> = Runner::default().with_expr(&best).run(&[]);

    let serialised = egg_to_serialized_egraph(&dummy_runner.egraph);
    serialised.to_json_file("filename.json").unwrap();

    println!("{:?}", best);
    println!("{}", best_cost);
}
