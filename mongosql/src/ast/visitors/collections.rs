use crate::ast::*;

#[derive(Default)]
struct CollectionVisitor {
    collections: Vec<CollectionSource>,
}

impl visitor::Visitor for CollectionVisitor {
    fn visit_collection_source(&mut self, node: CollectionSource) -> CollectionSource {
        self.collections.push(node.clone());
        node
    }

    fn visit_delete(&mut self, node: Delete) -> Delete {
        if let Datasource::Collection(ref target) = node.target {
            self.collections.push(target.clone());
        }
        node
    }

    fn visit_update(&mut self, node: Update) -> Update {
        if let Datasource::Collection(ref target) = node.target {
            self.collections.push(target.clone());
        }
        node
    }

    fn visit_insert(&mut self, node: Insert) -> Insert {
        if let Datasource::Collection(ref target) = node.target {
            self.collections.push(target.clone());
        }
        if let InsertSource::Query(query) = node.source {
            let query = query.walk(self);
            return Insert {
                target: node.target,
                columns: node.columns,
                source: InsertSource::Query(query),
            };
        }
        node
    }
}

pub fn get_collection_sources(stmt: Statement) -> Vec<CollectionSource> {
    let mut visitor = CollectionVisitor::default();
    stmt.walk(&mut visitor);
    visitor.collections
}
