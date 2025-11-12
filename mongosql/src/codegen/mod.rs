pub use crate::mapping_registry::MqlMappingRegistry;
use crate::{air, OperationType};
use thiserror::Error;

#[cfg(test)]
mod test;

mod expressions;
mod functions;
mod match_query;
mod stages;
mod utils;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("cannot generate Mql for {0:?} operator")]
    UnsupportedOperator(air::SqlOperator),
    #[error("cannot $convert to document")]
    ConvertToDocument,
    #[error("cannot $convert to array")]
    ConvertToArray,
}

#[derive(PartialEq, Debug)]
pub struct MqlTranslation {
    pub database: Option<String>,
    pub collection: Option<String>,
    pub operation_type: OperationType,
    pub pipeline: Vec<bson::Document>,
}

#[derive(Clone, Debug)]
pub struct MqlCodeGenerator {
    pub no_literal_wrap: bool,
}

pub fn generate_mql(plan: air::Stage) -> Result<MqlTranslation> {
    let cg = MqlCodeGenerator {
        no_literal_wrap: false,
    };

    cg.codegen_stage(plan)
}
