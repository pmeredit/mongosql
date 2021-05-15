macro_rules! test_codegen_plan {
    (
		$func_name:ident,
		Ok({
			database: $expected_db:expr,
			collection: $expected_collection:expr,
			pipeline: $expected_pipeline:expr,
		}),
		$input: expr,
	) => {
        #[test]
        fn $func_name() {
            use crate::codegen::{generate_mql, MqlTranslation};

            let input = $input;
            let expected_db = $expected_db;
            let expected_collection = $expected_collection;
            let expected_pipeline = $expected_pipeline;

            let MqlTranslation {
                database: db,
                collection: col,
                mapping_registry: _,
                pipeline: pipeline,
            } = generate_mql(input).expect("codegen failed");

            assert_eq!(expected_db, db);
            assert_eq!(expected_collection, col);
            assert_eq!(expected_pipeline, pipeline);
        }
    };

    ($func_name:ident, Err($expected_err:expr), $input:expr,) => {
        #[test]
        fn $func_name() {
            use crate::codegen::generate_mql;

            let input = $input;
            let expected = Err($expected_err);

            assert_eq!(expected, generate_mql(input));
        }
    };
}

macro_rules! test_codegen_expr {
    ($func_name:ident, $mapping_registry:expr, $expected:expr, $input:expr,) => {
        #[test]
        fn $func_name() {
            use crate::codegen::mql::MqlCodeGenerator;
            let mapping_registry = $mapping_registry;
            let expected = $expected;
            let input = $input;

            let gen = MqlCodeGenerator { mapping_registry };
            assert_eq!(expected, gen.codegen_expression(input));
        }
    };

    ($func_name:ident, $expected:expr, $input:expr,) => {
        test_codegen_expr!(
            $func_name,
            crate::codegen::mql::MappingRegistry::default(),
            $expected,
            $input,
        );
    };
}

mod collection {
    use crate::ir::*;

    test_codegen_plan!(
        simple,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
            ],
        }),
        Stage::Collection(Collection {
            db: "mydb".to_string(),
            collection: "col".to_string(),
        }),
    );
}

mod array_stage {
    use crate::ir::*;

    test_codegen_plan!(
        empty,
        Ok({
            database: None,
            collection: None,
            pipeline: vec![
                bson::doc!{"$array": {"arr": []}},
            ],
        }),
        Stage::Array(Array {
            exprs: vec![],
            alias: "arr".to_string(),
        }),
    );
    test_codegen_plan!(
        non_empty,
        Ok({
            database: None,
            collection: None,
            pipeline: vec![
                bson::doc!{"$array": {"arr": [{"$literal": false}]}},
            ],
        }),
        Stage::Array(Array {
            exprs: vec![Expression::Literal(Literal::Boolean(false))],
            alias: "arr".to_string(),
        }),
    );
}

mod filter {
    use crate::ir::*;

    test_codegen_plan!(
        simple,
        Ok({
            database: None,
            collection: None,
            pipeline: vec![
                bson::doc!{"$array": {"arr": []}},
                bson::doc!{"$match": {"$expr": {"$literal": true}}},
            ],
        }),
        Stage::Filter(Filter {
            condition: Expression::Literal(Literal::Boolean(true)),
            source: Stage::Array(Array {
                exprs: vec![],
                alias: "arr".to_string(),
            }).into(),
        }),
    );
}

mod sort {
    use crate::{
        codegen::Error,
        ir::{Expression::Reference, SortSpecification::*, *},
        map,
    };

    test_codegen_plan!(
        empty,
        Ok({
            database: None,
            collection: None,
            pipeline: vec![
                bson::doc!{"$array": {"arr": []}},
                bson::doc!{"$sort": {}},
            ],
        }),
        Stage::Sort(Sort {
            specs: vec![],
            source: Stage::Array(Array {
                exprs: vec![],
                alias: "arr".to_string(),
            }).into(),
        }),
    );
    test_codegen_plan!(
        single_spec_asc,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
                bson::doc!{"$sort": {"col": 1}},
            ],
        }),
        Stage::Sort(Sort {
            specs: vec![Asc(Reference(("col", 0u16).into()).into())],
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            }).into(),
        }),
    );
    test_codegen_plan!(
        single_spec_dsc,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
                bson::doc!{"$sort": {"col": -1}},
            ],
        }),
        Stage::Sort(Sort {
            specs: vec![Dsc(Reference(("col", 0u16).into()).into())],
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            }).into(),
        }),
    );
    test_codegen_plan!(
        multi_spec,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
                bson::doc!{"$sort": {"col": 1, "col.a": -1}},
            ],
        }),
        Stage::Sort(Sort {
            specs: vec![
                Asc(Reference(("col", 0u16).into()).into()),
                Dsc(
                    Expression::FieldAccess(FieldAccess{
                        field: "a".to_string(),
                        expr: Reference(("col", 0u16).into()).into(),
                    }).into(),
                ),
            ],
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            }).into(),
        }),
    );
    test_codegen_plan!(
        compound_ident,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
                bson::doc!{"$sort": {"col.f": 1}},
            ],
        }),
        Stage::Sort(Sort {
            specs: vec![Asc(
                Expression::FieldAccess(FieldAccess{
                    field: "f".to_string(),
                    expr: Reference(("col", 0u16).into()).into(),
                }).into(),
            )],
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            }).into(),
        }),
    );
    test_codegen_plan!(
        other_expr,
        Err(Error::InvalidSortKey),
        Stage::Sort(Sort {
            specs: vec![Asc(Expression::Literal(Literal::Integer(1)).into())],
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            })
            .into(),
        }),
    );
    test_codegen_plan!(
        non_ident_field_reference,
        Err(Error::InvalidSortKey),
        Stage::Sort(Sort {
            specs: vec![Asc(Expression::FieldAccess(FieldAccess {
                expr: Expression::Document(
                    map! {"a".into() => Expression::Literal(Literal::Integer(1))}
                )
                .into(),
                field: "sub".to_string(),
            })
            .into(),)],
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            })
            .into(),
        }),
    );
}

mod limit_offset {
    use crate::ir::*;

    test_codegen_plan!(
        limit_simple,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
                bson::doc!{"$limit": 1u64},
            ],
        }),
        Stage::Limit(Limit {
            limit: 1,
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            }).into(),
        }),
    );

    test_codegen_plan!(
        offset_simple,
        Ok({
            database: Some("mydb".to_string()),
            collection: Some("col".to_string()),
            pipeline: vec![
                bson::doc!{"$project": {"_id": 0, "col": "$$ROOT"}},
                bson::doc!{"$skip": 1u64},
            ],
        }),
        Stage::Offset(Offset {
            offset: 1,
            source: Stage::Collection(Collection {
                db: "mydb".to_string(),
                collection: "col".to_string(),
            }).into(),
        }),
    );
}

mod literal {
    use crate::ir::{Expression::*, Literal::*};
    use bson::{bson, Bson};

    test_codegen_expr!(null, Ok(bson!({ "$literal": Bson::Null })), Literal(Null),);
    test_codegen_expr!(bool, Ok(bson!({"$literal": true})), Literal(Boolean(true)),);
    test_codegen_expr!(
        string,
        Ok(bson!({"$literal": "abc"})),
        Literal(String("abc".into())),
    );
    test_codegen_expr!(int, Ok(bson!({"$literal": 5_i32})), Literal(Integer(5)),);
    test_codegen_expr!(long, Ok(bson!({"$literal": 6_i64})), Literal(Long(6)),);
    test_codegen_expr!(double, Ok(bson!({"$literal": 7.0})), Literal(Double(7.0)),);
}

mod reference {
    use crate::{
        codegen::{mql::MappingRegistry, Error},
        ir::Expression::*,
    };
    use bson::Bson;

    test_codegen_expr!(
        not_found,
        MappingRegistry::default(),
        Err(Error::ReferenceNotFound(("f", 0u16).into())),
        Reference(("f", 0u16).into()),
    );

    test_codegen_expr!(
        found,
        {
            let mut mr = MappingRegistry::default();
            mr.insert(("f", 0u16), "f");
            mr
        },
        Ok(Bson::String("$f".into())),
        Reference(("f", 0u16).into()),
    );
}

mod array {
    use crate::ir::{Expression::*, Literal};
    use bson::bson;

    test_codegen_expr!(empty, Ok(bson!([])), Array(vec![]),);
    test_codegen_expr!(
        non_empty,
        Ok(bson!([{"$literal": "abc"}])),
        Array(vec![Literal(Literal::String("abc".into()))]),
    );
    test_codegen_expr!(
        nested,
        Ok(bson!([{ "$literal": null }, [{ "$literal": null }]])),
        Array(vec![
            Literal(Literal::Null),
            Array(vec![Literal(Literal::Null)])
        ]),
    );
}

mod document {
    use crate::{
        codegen::Error,
        ir::{Expression::*, Literal},
        map,
    };
    use bson::bson;

    test_codegen_expr!(empty, Ok(bson!({"$literal": {}})), Document(map! {}),);
    test_codegen_expr!(
        non_empty,
        Ok(bson!({"foo": {"$literal": 1}})),
        Document(map! {"foo".to_string() => Literal(Literal::Integer(1)),}),
    );
    test_codegen_expr!(
        nested,
        Ok(bson!({"foo": {"$literal": 1}, "bar": {"baz": {"$literal": 2}}})),
        Document(map! {
            "foo".to_string() => Literal(Literal::Integer(1)),
            "bar".to_string() => Document(map!{
                "baz".to_string() => Literal(Literal::Integer(2))
            }),
        }),
    );
    test_codegen_expr!(
        dollar_prefixed_key_disallowed,
        Err(Error::DotsOrDollarsInFieldName),
        Document(map! {"$foo".to_string() => Literal(Literal::Integer(1)),}),
    );
    test_codegen_expr!(
        key_containing_dot_allowed,
        Ok(bson!({"foo.bar": {"$literal": 1}})),
        Document(map! {"foo.bar".to_string() => Literal(Literal::Integer(1)),}),
    );
}

mod field_access {
    use crate::{
        codegen::{mql::MappingRegistry, Error},
        ir::*,
        map,
    };
    use bson::Bson;

    test_codegen_expr!(
        reference,
        {
            let mut mr = MappingRegistry::default();
            mr.insert(("f", 0u16), "f");
            mr
        },
        Ok(Bson::String("$f.sub".to_string())),
        Expression::FieldAccess(FieldAccess {
            expr: Expression::Reference(("f", 0u16).into()).into(),
            field: "sub".to_string(),
        }),
    );
    test_codegen_expr!(
        field_access,
        {
            let mut mr = MappingRegistry::default();
            mr.insert(("f", 0u16), "f");
            mr
        },
        Ok(Bson::String("$f.sub.sub".to_string())),
        Expression::FieldAccess(FieldAccess {
            field: "sub".to_string(),
            expr: Expression::FieldAccess(FieldAccess {
                expr: Expression::Reference(("f", 0u16).into()).into(),
                field: "sub".to_string(),
            })
            .into(),
        }),
    );
    test_codegen_expr!(
        expr,
        Ok(bson::bson!({"$let": {
            "vars": {"docExpr": {"a": {"$literal": 1}}},
            "in": "$$docExpr.sub",
        }})),
        Expression::FieldAccess(FieldAccess {
            expr: Expression::Document(
                map! {"a".into() => Expression::Literal(Literal::Integer(1))}
            )
            .into(),
            field: "sub".to_string(),
        }),
    );
    test_codegen_expr!(
        string,
        Ok(bson::bson!({"$let": {
            "vars": {"docExpr": {"$literal": "$f"}},
            "in": "$$docExpr.sub",
        }})),
        Expression::FieldAccess(FieldAccess {
            expr: Expression::Literal(Literal::String("$f".into())).into(),
            field: "sub".to_string(),
        }),
    );
    test_codegen_expr!(
        dollar_prefixed_field,
        {
            let mut mr = MappingRegistry::default();
            mr.insert(("f", 0u16), "f");
            mr
        },
        Err(Error::DotsOrDollarsInFieldName),
        Expression::FieldAccess(FieldAccess {
            expr: Expression::Reference(("f", 0u16).into()).into(),
            field: "$sub".to_string(),
        }),
    );
    test_codegen_expr!(
        field_contains_dollar,
        {
            let mut mr = MappingRegistry::default();
            mr.insert(("f", 0u16), "f");
            mr
        },
        Err(Error::DotsOrDollarsInFieldName),
        Expression::FieldAccess(FieldAccess {
            expr: Expression::Reference(("f", 0u16).into()).into(),
            field: "s$ub".to_string(),
        }),
    );
    test_codegen_expr!(
        field_contains_dot,
        {
            let mut mr = MappingRegistry::default();
            mr.insert(("f", 0u16), "f");
            mr
        },
        Err(Error::DotsOrDollarsInFieldName),
        Expression::FieldAccess(FieldAccess {
            expr: Expression::Reference(("f", 0u16).into()).into(),
            field: "s.ub".to_string(),
        }),
    );
}
