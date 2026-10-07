
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use egg::{
    Applier, AstSize, CostFunction, ENodeOrVar::Var, Extractor, FromOp, Id, Language, PatternAst,
    RecExpr, Rewrite, Runner, RunnerResult, Subst, Symbol, rewrite,
};


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
    Eq([Id; 2]),
}

pub type EGraph = egg::EGraph<Math, ()>;

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
    Eq(),
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
            Math::Eq(_) => MathDiscriminant::Eq(),
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
            | Math::Lt(ids)
            | Math::Eq(ids) => ids,
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
            | Math::Lt(ids)
            | Math::Eq(ids) => ids,
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
            ("auxdiv", 2) => Ok(Math::AuxDiv([children[0], children[1]])),
            ("and", 2) => Ok(Math::And([children[0], children[1]])),
            ("lt", 2) => Ok(Math::Lt([children[0], children[1]])),
            ("eq", 2) => Ok(Math::Eq([children[0], children[1]])),
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
            Math::Eq(_) => write!(f, "Eq"),
        }
    }
}
