use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use egg::{
    Applier, AstSize, CostFunction, ENodeOrVar::Var, Extractor, FromOp, Id, Language, PatternAst,
    RecExpr, Rewrite, Runner, RunnerResult, Subst, Symbol, rewrite,
};

use crate::{
    IntRange, IntervalAnalysis,
    language_definition::{EGraph, Math},
};
use crate::{language_definition::MathDiscriminant, my_cost_function::SillyCostFn};

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

            Math::SaveDiv([a, b]) => {
                final_expression.add(Math::SaveDiv([a.into(), b.into()]));
            }

            Math::Not(a) => {
                final_expression.add(Math::Not(a.into()));
            }

            Math::Bubble([a, b]) => {
                final_expression.add(Math::Bubble([a.into(), b.into()]));
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

/*
    BEFORE THAT FUNCTION
    let expr: RecExpr<Math> = vec![
        Math::Symbol("x".into()),
        Math::Symbol("y".into()),
        Math::Div([0.into(), 1.into()]),
        Math::Symbol("z".into()),
        Math::Eq([2.into(), 3.into()]),
        Math::Not([4.into()]),
    ]
    .into();
    AFTER THAT FUNCTION
    let expr_fin: RecExpr<Math> = vec![
        Math::Symbol("x".into()),
        Math::Symbol("y".into()),
        Math::SaveDiv([0.into(), 1.into()]),
        Math::Num(0.into()),
        Math::Eq([1.into(), 3.into()]),
        Math::Not([4.into()]),
        Math::Bubble([2.into(), 5.into()]),
        Math::Symbol("z".into()),
        Math::Eq([6.into(), 7.into()]),
        Math::Not([8.into()]),
    ]
    .into();
*/
/// Takes in a tree and turns all divs into bubble safediv
pub fn div_to_bubble(final_tree: RecExpr<Math>) -> RecExpr<Math> {
    // in conjure-oxide we make some more safety checks here
    // but for just skip
    let mut final_expression: RecExpr<Math> = vec![].into();
    let mut id_map: Vec<Id> = vec![];

    for node in final_tree.as_ref() {
        let mut new_node = node.clone();

        // make child point at a correct node
        for child in new_node.children_mut() {
            *child = id_map[usize::from(*child)];
        }

        if let Math::Div([a, b]) = new_node {
            let savediv_id = final_expression.add(Math::SaveDiv([a, b]));
            let zero_id = final_expression.add(Math::Num(0));
            let eq_id = final_expression.add(Math::Eq([b, zero_id]));
            let not_id = final_expression.add(Math::Not([eq_id]));
            let bubble_id = final_expression.add(Math::Bubble([savediv_id, not_id]));
            id_map.push(bubble_id);
        } else {
            let new_id = final_expression.add(new_node);
            id_map.push(new_id);
        }
    }

    final_expression
}

/*
    Bring bubbles with a non-boolean expression higher up the tree.

    E.g. ((a / b) @ (b != 0)) = c => (a / b = c) @ (b != 0)

    BEFORE THIS FUNCTION
    let expr_fin: RecExpr<Math> = vec![
        Math::Symbol("x".into()),
        Math::Symbol("y".into()),
        Math::SaveDiv([0.into(), 1.into()]),
        Math::Num(0.into()),
        Math::Eq([1.into(), 3.into()]),
        Math::Not([4.into()]),
        Math::Bubble([2.into(), 5.into()]),
        Math::Symbol("z".into()),
        Math::Eq([6.into(), 7.into()]),
        Math::Not([8.into()]),
    ]
    .into();

    AFTER THIS FUNCTION

    let expr_bubbled: RecExpr<Math> = vec![
        Math::Symbol("x".into()),            // 0: x
        Math::Symbol("y".into()),            // 1: y
        Math::SaveDiv([0.into(), 1.into()]), // 2: x / y
        Math::Num(0.into()),                 // 3: 0
        Math::Eq([1.into(), 3.into()]),      // 4: y == 0
        Math::Not([4.into()]),               // 5: !(y == 0)  ->  (y != 0)
        Math::Symbol("z".into()),            // 6: z
        Math::Eq([2.into(), 6.into()]),      // 7: (x / y) == z
        Math::Bubble([7.into(), 5.into()]),  // 8: ((x / y) == z) @ (y != 0) <- Bubble lifted to Eq
        Math::Not([8.into()]),               // 9: !(((x / y) == z) @ (y != 0))
    ]
    .into();
*/

/*

take in final tree

let conditinos_poket = []
let vector = []

for each node in final_tree:
    // first case A:
    the node is a bubble([expr, cond])
        unwrap expr
        put cond in conditions_pocket
        // I would need to write new expr in a vector so that we match
        vector.add(expr.id)
    // second case B:
    the node is a boolean
        add the boolean node to the new tree
        if conditions has something
            let bubbled_condtions = AND all conditions_poket with and
            create a new Bubble([node.id, bubbled_condtions])
            clear poket vector
            vector.add(node.id)
        else
            vector.add(node.id)
    // third case C:
    otherwise
        add to new tree
        vector.add(node.id)

*/
fn is_boolean(id: Id, tree: &RecExpr<Math>) -> bool {
    matches!(
        tree[id],
        Math::Eq(_) | Math::Lt(_) | Math::Not(_) | Math::And(_)
    )
}

// This function bubbles non-boolean bubble conditions up to the nearest enclosing boolean node
pub fn bubble_up(final_tree: RecExpr<Math>) -> Result<RecExpr<Math>, &'static str> {
    // if final_tree.root is of type Math::Bubble
    // than throw an error. that should never happen
    if matches!(final_tree.as_ref().last(), Some(Math::Bubble(_))) {
        return Err("Root cannot be a Bubble");
    }

    let mut final_expression: RecExpr<Math> = vec![].into();
    let mut id_map: Vec<Id> = vec![];
    let mut conditions_pocket: Vec<Id> = vec![].into();

    for node in final_tree.as_ref() {
        let mut new_node = node.clone();

        // Remap children to point to the newly constructed IDs
        for child in new_node.children_mut() {
            *child = id_map[usize::from(*child)];
        }

        match new_node {
            Math::Bubble([expr, cond]) if !is_boolean(expr, &final_expression) => {
                conditions_pocket.push(cond);
                id_map.push(expr);
            }

            Math::Eq(_) | Math::Lt(_) | Math::Not(_) | Math::And(_) => {
                let bool_id = final_expression.add(new_node);

                if !conditions_pocket.is_empty() {
                    // combine all conditions in the pocket with And
                    let mut combined_cond = conditions_pocket[0];
                    for &c in &conditions_pocket[1..] {
                        combined_cond = final_expression.add(Math::And([combined_cond, c]));
                    }

                    let bubble_id = final_expression.add(Math::Bubble([bool_id, combined_cond]));
                    conditions_pocket.clear();
                    id_map.push(bubble_id);
                } else {
                    id_map.push(bool_id);
                }
            }

            _ => {
                let new_id = final_expression.add(new_node);
                id_map.push(new_id);
            }
        }
    }

    Ok(final_expression)
}

/*
    Reduce bubbles with a boolean expression to a conjunction with their condition.

    e.g. (a / b = c) @ (b != 0) => (a / b = c) & (b != 0)
*/
pub fn expand_bubble(final_tree: RecExpr<Math>) -> RecExpr<Math> {
    let mut final_expression: RecExpr<Math> = vec![].into();

    for node in final_tree.as_ref() {
        if let Math::Bubble([a, b]) = node {
            final_expression.add(Math::And([*a, *b]));
        } else {
            final_expression.add(node.clone());
        }
    }

    final_expression
}

/*
// conjure oxide function
#[register_rule("Bubble", 8900, [Bubble])]
fn expand_bubble(expr: &Expression, _: &SymbolTable) -> ApplicationResult {
    match expr {
        Expression::Bubble(_, a, b) if a.return_type() == ReturnType::Bool => {
            let a = Moo::unwrap_or_clone(Moo::clone(a));
            let b = Moo::unwrap_or_clone(Moo::clone(b));
            Ok(Reduction::pure(Expression::And(
                Metadata::new(),
                Moo::new(matrix_expr![a, b]),
            )))
        }
        _ => Err(ApplicationError::RuleNotApplicable),
    }
}
*/

// do not put root inside a bubble
//

/*
    #[register_rule("Bubble", 6000, [UnsafeDiv])]
    fn div_to_bubble(expr: &Expression, _: &SymbolTable) -> ApplicationResult {
        if is_all_constant(expr) {
            return Err(RuleNotApplicable);
        }
        if let Expression::UnsafeDiv(_, a, b) = expr {
            // bubble bottom up
            if !a.is_safe() || !b.is_safe() {
                return Err(RuleNotApplicable);
            }

            return Ok(Reduction::pure(Expression::Bubble(
                Metadata::new(),
                Moo::new(Expression::SafeDiv(Metadata::new(), a.clone(), b.clone())),
                Moo::new(Expression::Neq(
                    Metadata::new(),
                    b.clone(),
                    Moo::new(Expression::from(0)),
                )),
            )));
        }
        Err(ApplicationError::RuleNotApplicable)
    }
*/
