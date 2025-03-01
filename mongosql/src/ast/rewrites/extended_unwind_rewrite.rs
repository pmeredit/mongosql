use crate::ast::{
    self,
    definitions::{
        Datasource, Expression, ExtendedUnwindOption, ExtendedUnwindSource, UnwindOption,
        UnwindPathPart, UnwindPathPartOption, UnwindSource,
    },
    rewrites::{Error, Pass, Result},
    visitor::Visitor,
    SubpathExpr,
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

fn path_vec_to_path(mut path: Vec<String>) -> Expression {
    if path.len() == 1 {
        return Expression::Identifier(path.remove(0));
    }
    let mut ret = Expression::Identifier(path.remove(0));
    for p in path.into_iter() {
        ret = Expression::Subpath(SubpathExpr {
            expr: Box::new(ret),
            subpath: p.to_string(),
        });
    }
    ret
}

fn get_options(
    options: Vec<UnwindPathPartOption>,
    path: Vec<String>,
    global_index: Option<String>,
    global_outer: bool,
) -> Vec<UnwindOption> {
    let mut ret = vec![UnwindOption::Path(path_vec_to_path(path))];
    let mut found_index = false;
    let mut found_outer = false;
    for option in options.into_iter() {
        match option {
            UnwindPathPartOption::Index(i) => {
                ret.push(UnwindOption::Index(i));
                found_index = true;
            }
            UnwindPathPartOption::Outer(o) => {
                ret.push(UnwindOption::Outer(o));
                found_outer = true;
            }
        }
    }

    if !found_index && global_index.is_some() {
        // TODO: handle index renaming
        ret.push(UnwindOption::Index(global_index.clone().unwrap()));
    }
    if !found_outer && global_outer {
        ret.push(UnwindOption::Outer(global_outer));
    }
    ret
}

fn create_unwind_datasource(
    source: Datasource,
    paths: Vec<Vec<UnwindPathPart>>,
    global_index: Option<String>,
    global_outer: bool,
) -> Datasource {
    let mut ret = source;
    for path in paths {
        ret = create_unwind_datasource_for_path(ret, path, global_index.clone(), global_outer);
    }
    ret
}

fn create_unwind_datasource_for_path(
    source: Datasource,
    path: Vec<UnwindPathPart>,
    global_index: Option<String>,
    global_outer: bool,
) -> Datasource {
    let mut ret = source;
    let mut subpath = Vec::new();
    for path_part in path.into_iter() {
        subpath.push(path_part.field);
        // if the options are empty, we are not unwinding at this point in the path
        if path_part.options.is_empty() {
            continue;
        }
        for options in path_part.options {
            let options = get_options(options, subpath.clone(), global_index.clone(), global_outer);
            ret = Datasource::Unwind(UnwindSource {
                datasource: Box::new(ret),
                options,
            });
        }
    }
    ret
}
