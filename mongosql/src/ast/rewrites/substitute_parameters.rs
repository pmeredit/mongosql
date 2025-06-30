use crate::ast::{
    self,
    rewrites::{Pass, Result, Error},
    visitor::Visitor,
};

pub struct SubstituteParametersRewritePass<'a> {
    pub arguments: &'a[ast::Expression],
}

impl<'a> SubstituteParametersRewritePass<'a>  {
    pub fn new(arguments: &'a[ast::Expression]) -> Self {
        Self { arguments }
    }
}

impl<'a> Pass for SubstituteParametersRewritePass<'a> {
    fn apply(&self, ast: ast::Query) -> Result<ast::Query> {
        let mut visitor = SubstituteParametersVisitor::new(self.arguments);
        let ret = visitor.visit_query(ast);
        if let Some(error) = visitor.error {
            Err(error)
        } else {
            Ok(ret)
        }
    }
}

pub struct SubstituteParametersVisitor<'a> {
    pub arguments: &'a[ast::Expression],
    pub error: Option<Error>,
}

impl<'a> SubstituteParametersVisitor<'a> {
    pub fn new(arguments: &'a[ast::Expression]) -> Self {
        Self { arguments, error: None }
    }
}

impl<'a> Visitor for SubstituteParametersVisitor<'a> {
    fn visit_expression(&mut self, node:ast::definitions::Expression) -> ast::definitions::Expression {
        if let ast::Expression::Parameter(index) = node {
            if let Some(arg) = self.arguments.get(index) {
                arg.clone()
            } else {
                self.error = Some(Error::ParameterIndexOutOfBounds(index));
                node.walk(self)
            }
        } else {
            node.walk(self)
        }
    }
}
