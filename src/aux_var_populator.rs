use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use egg::{
    Applier, AstSize, CostFunction, ENodeOrVar::Var, Extractor, FromOp, Id, Language, PatternAst,
    RecExpr, Rewrite, Runner, RunnerResult, Subst, Symbol, rewrite,
};

use crate::language_definition::{EGraph, Math};
use crate::my_cost_function::SillyCostFn;

// This function would go through the final tree, replace all
// AuxDiv with variables (if count condition holds) and return
// back a populated RecExpr<Math>
// Input: (* (aux ?a ?b) ?c)
//
//
// Output: (/\ (* v_0 ?c) (== (* v_0 ?a) ?b))
// How do we keep track of ids because when we replace ?a and ?b. They exist in the
// array at the very start.
pub fn populate_auxiliary_division(final_tree: RecExpr<Math>) -> RecExpr<Math> {
    let mut final_expression: RecExpr<Math> = vec![].into();
    // each set represents == (* v_0 ?a) ?b
    let mut constraints: Vec<(Id, Id, Id)> = vec![];

    for node in final_tree {
        match node {
            Math::Num(n) => {
                final_expression.add(Math::Num(n));
            }

            Math::Symbol(s) => {
                final_expression.add(Math::Symbol(s));
            }

            Math::Add([a, b]) => {
                final_expression.add(Math::Add([a.into(), b.into()]));
            }

            Math::Mul([a, b]) => {
                final_expression.add(Math::Mul([a.into(), b.into()]));
            }

            Math::Div([a, b]) => {
                final_expression.add(Math::Div([a.into(), b.into()]));
            }

            Math::Shl([a, b]) => {
                final_expression.add(Math::Shl([a.into(), b.into()]));
            }

            Math::And([a, b]) => {
                final_expression.add(Math::And([a.into(), b.into()]));
            }

            Math::Lt([a, b]) => {
                final_expression.add(Math::Lt([a.into(), b.into()]));
            }

            Math::Eq([a, b]) => {
                final_expression.add(Math::Eq([a.into(), b.into()]));
            }
            Math::AuxDiv([a, b]) => {
                // let equality = Math::Eq([])
                // here we would create an expression for the auxdiv that than we would put
                // into the array. than we would just and everything in that expression into
                // the array
                // see auxdiv -> remember a and b
                // replace auxdiv with variable
                // create the aux * y
                // create the aux * y = x
                // write it in the list of final var constraints
                println!("{} {}", a, b);

                let var_name = format!("v_{}", constraints.len());
                let var_id = final_expression.add(Math::Symbol(var_name.into()));
                println!("{:?}", var_id);

                constraints.push((var_id, a, b));
            }
        }
    }

    let mut root_id = Id::from(final_expression.as_ref().len() - 1);

    for (var_id, a, b) in constraints {
        // Enforce: (var_id * b) == a
        let mul_id = final_expression.add(Math::Mul([var_id, b]));
        let eq_id = final_expression.add(Math::Eq([mul_id, a]));
        root_id = final_expression.add(Math::And([root_id, eq_id]));
    }

    final_expression
}
