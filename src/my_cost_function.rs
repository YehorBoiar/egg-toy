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

pub struct SillyCostFn<'a> {
    pub egraph: &'a EGraph,
}

impl<'a> CostFunction<Math> for SillyCostFn<'a> {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &Math, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        /*
         The problem with the current cost function is that it decides that auxdiv
         cost less if 2 nodes point at it.

         We create a dependency. A promise that if we take one auxdiv we take an another as
         well. But would that be a valid thing to do in our e-graph? How does cost function
         actually work.

         Egg's extractor is doing a bottom-up extraction over the e-classes. So it can take both a case
         where div is there, and a case where auxdiv is there just like in our toy example.

         To make auxdiv work we would need a CostFunction that would understand dependent costs
        */
        let op_cost = match enode {
            // we need it to say "auxdiv is worse than div when only 1 e-class points at the e-class where the
            // division is placed"
            Math::Div(_) => 100.1,
            Math::AuxDiv(_) => {
                // this looksup an id of current enode
                let id = self.egraph.lookup(enode.clone()).unwrap();

                // this counts how many canonicalised enodes point to our division
                // in the saturated graph
                let parent_classes: HashSet<Id> = self.egraph[id]
                    .parents()
                    .map(|parent_enode_id| self.egraph.find(parent_enode_id))
                    .collect();

                let parent_count = parent_classes.len();
                // this weight is actually correct.
                if parent_count > 1 { 20.1 } else { 200.1 }
            }
            _ => 1.1,
        };

        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}
