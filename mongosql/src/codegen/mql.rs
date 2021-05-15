use crate::{
    codegen::{Error, Result},
    ir::{self, binding_tuple::Key},
};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct MappingRegistry(HashMap<Key, String>);

impl MappingRegistry {
    pub fn new() -> Self {
        MappingRegistry(HashMap::new())
    }

    pub fn insert<K: Into<Key>, V: Into<String>>(&mut self, k: K, v: V) -> Option<String> {
        self.0.insert(k.into(), v.into())
    }

    pub fn merge(&mut self, other: MappingRegistry) {
        self.0.extend(other.0.into_iter());
    }
}

impl Default for MappingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! mappings {
	($($key:expr => $ref:expr),* $(,)?) => {
		MappingRegistry(std::iter::Iterator::collect(std::array::IntoIter::new([
			$({
				let key: Key = $key;
				let name: String = $ref.to_string();
				(key, name)
			},)*
		])))
	}
}

#[derive(PartialEq, Debug)]
pub struct MqlTranslation {
    pub database: Option<String>,
    pub collection: Option<String>,
    pub mapping_registry: MappingRegistry,
    pub pipeline: Vec<bson::Document>,
}

impl MqlTranslation {
    fn with_additional_stage(mut self, stage: bson::Document) -> Self {
        self.pipeline.push(stage);
        self
    }
}

#[derive(Clone)]
pub struct MqlCodeGenerator {
    pub mapping_registry: MappingRegistry,
}

impl MqlCodeGenerator {
    /// Recursively generates a translation for this stage and its
    /// sources. When this function is called, `self.mapping_registry`
    /// should include mappings for any datasources from outer scopes.
    /// Mappings for the current scope will be obtained by calling
    /// `codegen_stage` on source stages.
    pub fn codegen_stage(&self, stage: ir::Stage) -> Result<MqlTranslation> {
        use bson::{doc, Bson};
        use ir::Stage::*;
        match stage {
            Filter(f) => Ok(self.codegen_stage(*f.source)?.with_additional_stage(
                doc! {"$match": {"$expr": self.codegen_expression(f.condition)?}},
            )),
            Project(_) => unimplemented!(),
            Group(_) => unimplemented!(),
            Limit(l) => Ok(self
                .codegen_stage(*l.source)?
                .with_additional_stage(doc! {"$limit": l.limit})),
            Offset(o) => Ok(self
                .codegen_stage(*o.source)?
                .with_additional_stage(doc! {"$skip": o.offset})),
            Sort(s) => self.codegen_sort(s),
            Collection(c) => Ok(MqlTranslation {
                database: Some(c.db),
                collection: Some(c.collection.clone()),
                mapping_registry: mappings! { (&c.collection, 0u16).into() => &c.collection },
                pipeline: vec![doc! {"$project": {"_id": 0, &c.collection: "$$ROOT"}}],
            }),
            Array(arr) => {
                let mapping_registry = mappings! {(&arr.alias, 0u16).into() => &arr.alias};
                let docs = arr
                    .exprs
                    .into_iter()
                    .map(|e| self.codegen_expression(e))
                    .collect::<Result<Vec<Bson>>>()?;
                Ok(MqlTranslation {
                    database: None,
                    collection: None,
                    mapping_registry,
                    pipeline: vec![doc! {"$array": {arr.alias: Bson::Array(docs)}}],
                })
            }
            Join(_) => unimplemented!(),
            Set(_) => unimplemented!(),
        }
    }

    fn with_merged_mappings(mut self, mappings: MappingRegistry) -> Self {
        self.mapping_registry.merge(mappings);
        self
    }

    fn codegen_sort(&self, sort: ir::Sort) -> Result<MqlTranslation> {
        use bson::{doc, Bson};
        use ir::{Expression::*, SortSpecification::*};

        let source_translation = self.codegen_stage(*sort.source)?;
        let expr_code_generator = self
            .clone()
            .with_merged_mappings(source_translation.mapping_registry.clone());

        let sort_specs = sort
            .specs
            .into_iter()
            .map(|spec| {
                let (expr, direction) = match spec {
                    Asc(expr) => (*expr, Bson::Int32(1)),
                    Dsc(expr) => (*expr, Bson::Int32(-1)),
                };

                // anything that's not a reference or a static field
                // access cannot be used as a sort key
                match expr {
                    Reference(_) | FieldAccess(_) => Ok(()),
                    _ => Err(Error::InvalidSortKey),
                }?;

                // we still need to ensure that the result is a
                // string, since not all FieldAccess expressions will
                // translate to single MQL references
                let expr = expr_code_generator.codegen_expression(expr)?;
                let key = match expr {
                    Bson::String(s) => Ok(s[1..].to_string()),
                    _ => Err(Error::InvalidSortKey),
                }?;

                Ok((key, direction))
            })
            .collect::<Result<bson::Document>>()?;

        Ok(source_translation.with_additional_stage(doc! {"$sort": sort_specs}))
    }

    /// Recursively generates a translation for this expression. When
    /// this function is called, `self.mapping_registry` should
    /// include mappings for all datasources in scope.
    #[allow(dead_code)]
    pub fn codegen_expression(&self, expr: ir::Expression) -> Result<bson::Bson> {
        use bson::Bson;
        use ir::{Expression::*, Literal::*};
        match expr {
            Literal(lit) => Ok(bson::bson!({
                "$literal": match lit {
                    Null => Bson::Null,
                    Boolean(b) => Bson::Boolean(b),
                    String(s) => Bson::String(s),
                    Integer(i) => Bson::Int32(i),
                    Long(l) => Bson::Int64(l),
                    Double(d) => Bson::Double(d),
                },
            })),
            Reference(key) => self
                .mapping_registry
                .0
                .get(&key)
                .ok_or(Error::ReferenceNotFound(key))
                .map(|s| Bson::String(format!("${}", s))),
            Array(exprs) => Ok(Bson::Array(
                exprs
                    .into_iter()
                    .map(|e| self.codegen_expression(e))
                    .collect::<Result<Vec<Bson>>>()?,
            )),
            Document(map) => Ok(Bson::Document({
                if map.is_empty() {
                    bson::doc! {"$literal": {}}
                } else {
                    map.into_iter()
                        .map(|(k, v)| {
                            if k.starts_with('$') {
                                Err(Error::DotsOrDollarsInFieldName)
                            } else {
                                Ok((k, self.codegen_expression(v)?))
                            }
                        })
                        .collect::<Result<bson::Document>>()?
                }
            })),
            FieldAccess(fa) => {
                if fa.field.contains(&['.', '$'] as &[_]) {
                    return Err(Error::DotsOrDollarsInFieldName);
                };
                Ok(match self.codegen_expression(*fa.expr)? {
                    Bson::String(e) => Bson::String(format!("{}.{}", e, fa.field)),
                    e => bson::bson!({"$let": {
                        "vars": {"docExpr": e},
                        "in": format!("$$docExpr.{}", fa.field),
                    }}),
                })
            }
            Function(_) => unimplemented!(),
            SubqueryExpression(_) => unimplemented!(),
            SubqueryComparison(_) => unimplemented!(),
            Exists(_) => unimplemented!(),
        }
    }
}
