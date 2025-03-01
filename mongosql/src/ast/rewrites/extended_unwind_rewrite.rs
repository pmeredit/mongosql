use crate::ast::{
    self,
    definitions::{Datasource, ExtendedUnwindSource, UnwindSource},
    rewrites::{Pass, Result},
    visitor::Visitor,
};

pub struct ExtendedUnwindRewritePass;

impl Pass for ExtendedUnwindRewritePass {
    fn apply(&self, query: ast::Query) -> Result<ast::Query> {
        let mut visitor = ExtendedUnwindRewriteVisitor;
        Ok(query.walk(&mut visitor))
    }
}

/// The visitor that performs the rewrites for the `ExtendedUnwindRewriteRewritePass`.
#[derive(Default)]
struct ExtendedUnwindRewriteVisitor;

impl Visitor for ExtendedUnwindRewriteVisitor {
    fn visit_datasource(&mut self, data_source: ast::Datasource) -> ast::Datasource {
        match data_source {
            ast::Datasource::ExtendedUnwind(ref ds) => {
                // Perform the rewrite here
                dbg!(&ds);
                data_source
            }
            _ => data_source,
        }
    }
}
