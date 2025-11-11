use crate::ast;
use thiserror::Error;

mod substitute_parameters;
pub use substitute_parameters::SubstituteParametersRewritePass;
mod alias;
pub use alias::AddAliasRewritePass;
mod extended_unwind_rewrite;
pub use extended_unwind_rewrite::ExtendedUnwindRewritePass;
mod select;
pub use select::SelectRewritePass;
pub mod tuples;
pub use tuples::InTupleRewritePass;
pub use tuples::SingleTupleRewritePass;
mod from;
pub use from::ImplicitFromRewritePass;
mod order_by;
pub use order_by::PositionalSortKeyRewritePass;
mod aggregate;
pub use aggregate::AggregateRewritePass;
mod table_subquery;
use table_subquery::TableSubqueryRewritePass;
mod group_by_select_alias;
use group_by_select_alias::GroupBySelectAliasRewritePass;
mod not;
use not::NotComparisonRewritePass;
mod optional_parameters;
use optional_parameters::OptionalParameterRewritePass;
mod scalar_functions;
use scalar_functions::ScalarFunctionsRewritePass;
mod with_query;
pub use with_query::WithQueryRewritePass;

#[cfg(test)]
mod test;

const PASSES_LIST: &[&dyn Pass] = &[
    &ExtendedUnwindRewritePass,
    &InTupleRewritePass,
    &SingleTupleRewritePass,
    &GroupBySelectAliasRewritePass,
    &AddAliasRewritePass,
    &PositionalSortKeyRewritePass,
    &AggregateRewritePass,
    &SelectRewritePass,
    &ImplicitFromRewritePass,
    &TableSubqueryRewritePass,
    &OptionalParameterRewritePass,
    &NotComparisonRewritePass,
    &ScalarFunctionsRewritePass,
    // WithQueryRewritePass can introduce duplicated queries, so it should be the last pass so
    // any rewrites that apply in the WithQuery queries are applied only once.
    &WithQueryRewritePass,
];

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during rewrite passes
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("positional sort keys are not allowed with SELECT VALUE")]
    PositionalSortKeyWithSelectValue,
    #[error("positional sort keys are not allowed with SELECT *")]
    PositionalSortKeyWithSelectStar,
    #[error("positional sort key {0} out of range")]
    PositionalSortKeyOutOfRange(usize),
    #[error("positional sort key {0} references a select expression with no alias")]
    NoAliasForSortKeyAtPosition(usize),
    #[error("aggregation functions may not be used as GROUP BY keys")]
    AggregationFunctionInGroupByKeyList,
    #[error("cannot specify aggregation functions in GROUP BY AGGREGATE clause and elsewhere")]
    AggregationFunctionInGroupByAggListAndElsewhere,
    #[error("all SELECT expressions must be given aliases before the SelectRewritePass")]
    NoAliasForSelectExpression,
    #[error("the top-level SELECT in a subquery expression must be a standard SELECT")]
    SubqueryWithSelectValue,
    #[error("incorrect argument count for {name}: required {required}, found {found}")]
    IncorrectArgumentCount {
        name: &'static str,
        required: &'static str,
        found: usize,
    },
    #[error("invalid date part: {0}")]
    InvalidDatePart(&'static str),
    #[error("UNWIND datasource must have a PATH")]
    UnwindSourceWithoutPath,
    #[error("duplicate option in UNWIND: {0}")]
    DuplicateOptionInUnwind(&'static str),
    #[error("parameter index {0} is out of bounds")]
    ParameterIndexOutOfBounds(usize),
}

/// A fallible transformation that can be applied to a query
pub trait Pass {
    // apply_to_statement can be overridden to handle statements other than queries
    // when special treatment is needed.
    fn apply_to_statement(&self, stmt: ast::Statement) -> Result<ast::Statement> {
        match stmt {
            ast::Statement::Query(q) => {
                let rewritten = self.apply_to_query(q)?;
                Ok(ast::Statement::Query(rewritten))
            }
            ast::Statement::Insert(i) => match i.source {
                ast::InsertSource::Query(q) => {
                    let rewritten = self.apply_to_query(q)?;
                    Ok(ast::Statement::Insert(ast::Insert {
                        source: ast::InsertSource::Query(rewritten),
                        ..i
                    }))
                }
                _ => Ok(ast::Statement::Insert(i)),
            },
            // Sadly we can't apply these passes to just the RETURNING clauses
            // of UPDATE and DELETE
            _ => Ok(stmt),
        }
    }
    fn apply_to_query(&self, query: ast::Query) -> Result<ast::Query>;
}

pub fn rewrite_statement(stmt: ast::Statement) -> Result<ast::Statement> {
    let mut rewritten = stmt;
    for pass in PASSES_LIST {
        rewritten = pass.apply_to_statement(rewritten)?;
    }
    Ok(rewritten)
}

/// Rewrite the provided query by applying rewrites as specified in the MongoSql spec.
pub fn rewrite_query(query: ast::Query) -> Result<ast::Query> {
    let mut rewritten = query;
    for pass in PASSES_LIST {
        rewritten = pass.apply_to_query(rewritten)?;
    }
    Ok(rewritten)
}
