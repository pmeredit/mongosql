use crate::air;
use thiserror::Error;

mod accumulators;
use crate::air::desugarer::accumulators::AccumulatorsDesugarerPass;
mod join;
use crate::air::desugarer::join::JoinDesugarerPass;
mod match_null_semantics;
use crate::air::desugarer::match_null_semantics::MatchDesugarerPass;
mod lookup;
use crate::air::desugarer::lookup::LookupDesugarerPass;
mod sql_null_semantics_operators;
use crate::air::desugarer::sql_null_semantics_operators::SQLNullSemanticsOperatorsDesugarerPass;
mod subquery;
use crate::air::desugarer::subquery::SubqueryExprDesugarerPass;
mod unsupported_operators;

#[cfg(test)]
mod test;

use crate::air::desugarer::unsupported_operators::UnsupportedOperatorsDesugarerPass;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during desugarer passes
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[allow(dead_code)]
    #[error("TODO replace error when passes are implemented")]
    TodoError,
    #[error("pattern for $like must be literal")]
    InvalidLikePatternError,
}

/// A fallible transformation that can be applied to a pipeline
pub trait Pass {
    fn apply(&self, pipeline: air::Stage) -> Result<air::Stage>;
}

/// Desugar the provided pipeline by applying desugarer passes.
#[allow(dead_code)]
pub fn desugar_pipeline(pipeline: air::Stage) -> Result<air::Stage> {
    let passes: Vec<&dyn Pass> = vec![
        &JoinDesugarerPass,
        &AccumulatorsDesugarerPass,
        &LookupDesugarerPass,
        &SubqueryExprDesugarerPass,
        &MatchDesugarerPass,
        &UnsupportedOperatorsDesugarerPass,
        &SQLNullSemanticsOperatorsDesugarerPass,
    ];

    let mut desugared = pipeline;
    for pass in passes {
        desugared = pass.apply(desugared)?
    }
    Ok(desugared)
}
