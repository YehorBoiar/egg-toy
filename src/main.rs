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
        self.discriminant() == other.discriminant()
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

pub fn main() {
    env_logger::init();
    
    let rules: &[Rewrite<Math, ()>] = &[
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("mult-zero"; "(* ?a 0)" => "0"),
        rewrite!("add-zero"; "(+ ?a 0)" => "?a"),
    ];

    let expr: RecExpr<Math> = vec![
        Math::Symbol("x".into()),
        Math::Num(0),
        Math::Mul([0.into(), 1.into()]),
        Math::Add([2.into(), 0.into()]),
    ]
    .into();

    let runner = Runner::default().with_expr(&expr).run(rules);

    let egraph = runner.egraph;

    let serialised = egg_to_serialized_egraph(&egraph);

    println!("{:?}\n\n----\n\n", egraph);

    serialised.to_json_file("egraph.json").unwrap();
}
