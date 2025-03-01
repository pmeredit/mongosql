use crate::ast::{
    self,
    definitions::{
        Datasource, Expression, ExtendedUnwindOption, ExtendedUnwindSource, UnwindOption,
        UnwindPathPart, UnwindPathPartOption, UnwindSource,
    },
    rewrites::{Error, Pass, Result},
    visitor::Visitor,
};

pub struct ExtendedUnwindRewritePass;

impl Pass for ExtendedUnwindRewritePass {
    fn apply(&self, query: ast::Query) -> Result<ast::Query> {
        let mut visitor = ExtendedUnwindRewriteVisitor::default();
        let res = query.walk(&mut visitor);
        if let Some(error) = visitor.error {
            return Err(error);
        }
        Ok(res)
    }
}

/// The visitor that performs the rewrites for the `ExtendedUnwindRewriteRewritePass`.
#[derive(Default)]
struct ExtendedUnwindRewriteVisitor {
    error: Option<Error>,
}

impl Visitor for ExtendedUnwindRewriteVisitor {
    fn visit_datasource(&mut self, data_source: ast::Datasource) -> ast::Datasource {
        match data_source {
            ast::Datasource::ExtendedUnwind(ExtendedUnwindSource {
                datasource: source,
                options,
            }) => {
                let (mut paths, mut global_index, mut global_outer) = (None, None, false);
                for option in options {
                    match option {
                        ExtendedUnwindOption::Paths(path) => {
                            paths = Some(path);
                        }
                        ExtendedUnwindOption::Index(i) => {
                            global_index = Some(i);
                        }
                        ExtendedUnwindOption::Outer(o) => {
                            global_outer = o;
                        }
                    }
                }
                if paths.is_none() {
                    self.error = Some(Error::UnwindSourceWithoutPath);
                    return ast::Datasource::ExtendedUnwind(ExtendedUnwindSource {
                        datasource: source,
                        options: Vec::new(),
                    });
                }
                create_unwind_datasource(*source, paths.unwrap(), global_index, global_outer)
            }
            _ => data_source,
        }
    }
}

fn create_unwind_datasource(
    source: Datasource,
    mut paths: Vec<Vec<UnwindPathPart>>,
    global_index: Option<String>,
    global_outer: bool,
) -> Datasource {
    if paths.is_empty() {
        return source;
    }
    let mut current_path = paths.pop().unwrap();
    if current_path.is_empty() {
        return create_unwind_datasource(source, paths, global_index, global_outer);
    }
    let mut current_part = current_path.pop().unwrap();
    let mut options = vec![UnwindOption::Path(Expression::Identifier(
        current_part.field,
    ))];
    let mut found_index = false;
    let mut found_outer = false;
    // TODO: Handle nested arrays
    for option in current_part.options.pop().unwrap().into_iter() {
        match option {
            UnwindPathPartOption::Index(i) => {
                options.push(UnwindOption::Index(i));
                found_index = true;
            }
            UnwindPathPartOption::Outer(o) => {
                options.push(UnwindOption::Outer(o));
                found_outer = true;
            }
        }
    }
    if !found_index && global_index.is_some() {
        // TODO: handle index renaming
        options.push(UnwindOption::Index(global_index.clone().unwrap()));
    }
    if !found_outer && global_outer {
        options.push(UnwindOption::Outer(global_outer));
    }

    create_unwind_datasource(
        Datasource::Unwind(UnwindSource {
            datasource: Box::new(source),
            options,
        }),
        paths,
        global_index,
        global_outer,
    )
}
