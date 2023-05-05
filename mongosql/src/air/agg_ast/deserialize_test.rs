macro_rules! test_deserialize_stage {
    ($func_name:ident, expected = $expected:expr, input = $input:expr) => {
        #[test]
        fn $func_name() {
            use super::TestStage;

            let input = $input;
            let s: TestStage = serde_yaml::from_str(&input).unwrap();

            assert_eq!($expected, s.stage)
        }
    };
}

macro_rules! test_deserialize_expr {
    ($func_name:ident, expected = $expected:expr, input = $input:expr) => {
        #[test]
        fn $func_name() {
            use super::TestExpr;

            let input = $input;
            let e: TestExpr = serde_yaml::from_str(&input).unwrap();

            assert_eq!($expected, e.expr)
        }
    };
}

mod stage_test {
    use crate::air::agg_ast::ast_definitions::Stage;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Deserialize)]
    struct TestStage {
        stage: Stage,
    }

    mod documents {
        use crate::{
            air::agg_ast::ast_definitions::{Expression, LiteralValue, Stage, StringOrRef},
            map,
        };

        test_deserialize_stage!(
            empty,
            expected = Stage::Documents(vec![]),
            input = r#"stage: {"$documents": []}"#
        );

        test_deserialize_stage!(
            singleton,
            expected = Stage::Documents(vec![
                map! {"a".to_string() => Expression::Literal(LiteralValue::Integer(1)) }
            ]),
            input = r#"stage: {"$documents": [{"a": 1}]}"#
        );

        test_deserialize_stage!(
            multiple_elements,
            expected = Stage::Documents(vec![
                map! {
                    "a".to_string() => Expression::Literal(LiteralValue::Integer(1)),
                    "b".to_string() => Expression::Literal(LiteralValue::Integer(2)),
                },
                map! {
                    "a".to_string() => Expression::StringOrRef(StringOrRef::String("yes".to_string())),
                    "b".to_string() => Expression::Literal(LiteralValue::Null),
                },
                map! {
                    "a".to_string() => Expression::Document(map! {
                        "b".to_string() => Expression::Document(map! {
                            "c".to_string() => Expression::Literal(LiteralValue::Boolean(true)),
                        }),
                    }),
                },
            ]),
            input = r#"stage: {"$documents": [
                                {"a": 1, "b": 2},
                                {"a": "yes", "b": null},
                                {"a": {"b": {"c": true}}}
            ]}"#
        );
    }

    mod project {
        use crate::{
            air::agg_ast::ast_definitions::{
                Expression, LiteralValue, ProjectItem, Stage, StringOrRef, UntaggedOperator,
            },
            map,
        };

        test_deserialize_stage!(
            empty,
            expected = Stage::Project(map! {}),
            input = r#"stage: {"$project": {}}"#
        );

        test_deserialize_stage!(
            singleton_exclusion,
            expected = Stage::Project(map! { "_id".to_string() => ProjectItem::Exclusion }),
            input = r#"stage: {"$project": {"_id": 0}}"#
        );

        test_deserialize_stage!(
            singleton_inclusion,
            expected = Stage::Project(map! { "_id".to_string() => ProjectItem::Inclusion }),
            input = r#"stage: {"$project": {"_id": 1}}"#
        );

        test_deserialize_stage!(
            singleton_assignment,
            expected = Stage::Project(
                map! { "_id".to_string() => ProjectItem::Assignment(Expression::Literal(LiteralValue::Integer(42))) }
            ),
            input = r#"stage: {"$project": {"_id": 42}}"#
        );

        test_deserialize_stage!(
            multiple_elements,
            expected = Stage::Project(map! {
                "_id".to_string() => ProjectItem::Exclusion,
                "foo".to_string() => ProjectItem::Assignment(Expression::StringOrRef(StringOrRef::Variable("ROOT".to_string()))),
                "bar".to_string() => ProjectItem::Assignment(Expression::StringOrRef(StringOrRef::FieldRef("bar".to_string()))),
                "a".to_string() => ProjectItem::Assignment(Expression::UntaggedOperator(UntaggedOperator {
                    op: "$add".to_string(),
                    args: vec![
                        Expression::Literal(LiteralValue::Integer(1)),
                        Expression::Literal(LiteralValue::Integer(2)),
                    ]
                })),
                "x".to_string() => ProjectItem::Assignment(Expression::UntaggedOperator(UntaggedOperator {
                    op: "$literal".to_string(),
                    args: vec![
                        Expression::Literal(LiteralValue::Integer(0)),
                    ]
                })),
                "y".to_string() => ProjectItem::Assignment(Expression::UntaggedOperator(UntaggedOperator {
                    op: "$literal".to_string(),
                    args: vec![
                        Expression::Literal(LiteralValue::Integer(1)),
                    ]
                })),
            }),
            input = r#"stage: {"$project": {
                                "_id": 0,
                                "foo": "$$ROOT",
                                "bar": "$bar",
                                "a": {"$add": [1, 2]},
                                "x": { "$literal": 0 },
                                "y": { "$literal": 1 },
            }}"#
        );
    }

    mod replace_with {
        use crate::air::agg_ast::ast_definitions::{
            Expression, Stage, StringOrRef, UntaggedOperator,
        };

        test_deserialize_stage!(
            simple,
            expected = Stage::ReplaceWith(Expression::StringOrRef(StringOrRef::FieldRef(
                "a".to_string()
            ))),
            input = r#"stage: {"$replaceWith": "$a"}"#
        );

        test_deserialize_stage!(
            complex,
            expected = Stage::ReplaceWith(Expression::UntaggedOperator(UntaggedOperator {
                op: "$mergeObjects".to_string(),
                args: vec![
                    Expression::StringOrRef(StringOrRef::Variable("ROOT".to_string())),
                    Expression::StringOrRef(StringOrRef::FieldRef("as".to_string())),
                ]
            })),
            input = r#"stage: {"$replaceWith": {"$mergeObjects": ["$$ROOT", "$as"]}}"#
        );
    }

    mod match_stage {
        use crate::air::agg_ast::ast_definitions::MatchExpr;
        use crate::{
            air::agg_ast::ast_definitions::{
                Expression, LiteralValue, MatchExpression, Stage, StringOrRef, UntaggedOperator,
            },
            map,
        };

        test_deserialize_stage!(
            non_expr,
            expected = Stage::Match(MatchExpression::NonExpr(Expression::Document(map! {
                "a".to_string() => Expression::UntaggedOperator(UntaggedOperator {
                    op: "$exists".to_string(),
                    args: vec![Expression::Literal(LiteralValue::Boolean(true))]
                })
            }))),
            input = r#"stage: {"$match": {"a": {"$exists": true}}}"#
        );

        test_deserialize_stage!(
            expr,
            expected = Stage::Match(MatchExpression::Expr(MatchExpr {
                expr: Box::new(Expression::UntaggedOperator(UntaggedOperator {
                    op: "$sqlEq".to_string(),
                    args: vec![
                        Expression::StringOrRef(StringOrRef::FieldRef("a".to_string())),
                        Expression::StringOrRef(StringOrRef::FieldRef("b".to_string())),
                    ]
                }))
            })),
            input = r#"stage: {"$match": {"$expr": {"$sqlEq": ["$a", "$b"]}}}"#
        );
    }

    mod limit_skip {
        use crate::air::agg_ast::ast_definitions::Stage;

        test_deserialize_stage!(
            limit,
            expected = Stage::Limit(10),
            input = r#"stage: {"$limit": 10}"#
        );

        test_deserialize_stage!(
            skip,
            expected = Stage::Skip(100),
            input = r#"stage: {"$skip": 100}"#
        );
    }

    mod sort {
        use crate::{air::agg_ast::ast_definitions::Stage, map};

        test_deserialize_stage!(
            empty,
            expected = Stage::Sort(map! {}),
            input = r#"stage: {"$sort": {}}"#
        );

        test_deserialize_stage!(
            singleton,
            expected = Stage::Sort(map! { "a".to_string() => 1 }),
            input = r#"stage: {"$sort": {"a": 1}}"#
        );

        test_deserialize_stage!(
            multiple_elements,
            expected = Stage::Sort(map! { "a".to_string() => 1, "b".to_string() => -1 }),
            input = r#"stage: {"$sort": {"a": 1, "b": -1}}"#
        );
    }

    mod unwind {
        use crate::air::agg_ast::ast_definitions::{
            Expression, Stage, StringOrRef, Unwind, UnwindExpr,
        };

        test_deserialize_stage!(
            unwind_field_ref,
            expected = Stage::Unwind(Unwind::FieldPath(Expression::StringOrRef(
                StringOrRef::FieldRef("eca58228-b657-498a-b76e-f48a9161a404".to_string())
            ))),
            input = r#"stage: { "$unwind": "$eca58228-b657-498a-b76e-f48a9161a404" }"#
        );

        test_deserialize_stage!(
            unwind_document_no_options,
            expected = Stage::Unwind(Unwind::Document(UnwindExpr {
                path: Box::new(Expression::StringOrRef(StringOrRef::FieldRef(
                    "array".to_string()
                ))),
                include_array_index: None,
                preserve_null_and_empty_arrays: None
            })),
            input = r#"stage: {"$unwind": {"path": "$array"}}"#
        );

        test_deserialize_stage!(
            unwind_document_all_options,
            expected = Stage::Unwind(Unwind::Document(UnwindExpr {
                path: Box::new(Expression::StringOrRef(StringOrRef::FieldRef(
                    "array".to_string()
                ))),
                include_array_index: Some("i".to_string()),
                preserve_null_and_empty_arrays: Some(true)
            })),
            input = r#"stage: {"$unwind": {"path": "$array", "includeArrayIndex": "i", "preserveNullAndEmptyArrays": true }}"#
        );
    }

    mod join {
        use crate::{
            air::agg_ast::ast_definitions::{
                Expression, Join, JoinType, LiteralValue, MatchExpr, MatchExpression, ProjectItem,
                Stage, StringOrRef, UntaggedOperator,
            },
            map,
        };

        test_deserialize_stage!(
            inner_join,
            expected = Stage::Join(Box::new(Join {
                database: None,
                collection: Some("bar".to_string()),
                let_body: None,
                join_type: JoinType::Inner,
                pipeline: vec![],
                condition: None
            })),
            input =
                r#"stage: {"$join": {"collection": "bar", "joinType": "inner", "pipeline": [] }}"#
        );

        test_deserialize_stage!(
            left_join_with_db,
            expected = Stage::Join(Box::new(Join {
                database: Some("db".to_string()),
                collection: Some("bar".to_string()),
                let_body: None,
                join_type: JoinType::Left,
                pipeline: vec![],
                condition: None
            })),
            input = r#"stage: { "$join":
                  {
                    "database": "db",
                    "collection": "bar",
                    "joinType": "left",
                    "pipeline": [],
                  },
              }"#
        );

        test_deserialize_stage!(
            join_with_no_collection_and_pipeline,
            expected = Stage::Join(Box::new(Join {
                database: None,
                collection: None,
                let_body: None,
                join_type: JoinType::Inner,
                pipeline: vec![Stage::Documents(vec![
                    map! {"a".to_string() => Expression::Literal(LiteralValue::Integer(1)) },
                    map! {"a".to_string() => Expression::Literal(LiteralValue::Integer(2)) },
                    map! {"a".to_string() => Expression::Literal(LiteralValue::Integer(3)) },
                ])],
                condition: None
            })),
            input = r#"stage: {
                "$join":
                  {
                    "joinType": "inner",
                    "pipeline":
                      [{ "$documents": [{ "a": 1 }, { "a": 2 }, { "a": 3 }] }],
                  },
              }"#
        );

        test_deserialize_stage!(
            join_with_let_vars_and_condition,
            expected = Stage::Join(Box::new(Join {
                database: None,
                collection: Some("bar".to_string()),
                let_body: Some(map! {
                    "x".to_string() => Expression::StringOrRef(StringOrRef::FieldRef("x".to_string()))
                }),
                join_type: JoinType::Inner,
                pipeline: vec![Stage::Project(map! {
                    "_id".to_string() => ProjectItem::Exclusion,
                    "x".to_string() => ProjectItem::Inclusion,
                })],
                condition: Some(Stage::Match(MatchExpression::Expr(MatchExpr {
                    expr: Box::new(Expression::UntaggedOperator(UntaggedOperator {
                        op: "$sqlEq".to_string(),
                        args: vec![
                            Expression::StringOrRef(StringOrRef::Variable("x".to_string())),
                            Expression::StringOrRef(StringOrRef::FieldRef("x".to_string())),
                        ]
                    }))
                })))
            })),
            input = r#"stage: {
                "$join":
                  {
                    "collection": "bar",
                    "joinType": "inner",
                    "let": { "x": "$x" },
                    "pipeline": [{ "$project": { "_id": 0, "x": 1 } }],
                    "condition":
                      { "$match": { "$expr": { "$sqlEq": ["$$x", "$x"] } } },
                  },
              }"#
        );

        test_deserialize_stage!(
            nested_join,
            expected = Stage::Join(Box::new(Join {
                database: None,
                collection: Some("bar".to_string()),
                let_body: None,
                join_type: JoinType::Inner,
                pipeline: vec![Stage::Join(Box::new(Join {
                    database: None,
                    collection: Some("baz".to_string()),
                    join_type: JoinType::Inner,
                    let_body: None,
                    pipeline: vec![Stage::Join(Box::new(Join {
                        database: None,
                        collection: Some("car".to_string()),
                        join_type: JoinType::Inner,
                        let_body: None,
                        pipeline: vec![],
                        condition: None
                    }))],
                    condition: None
                }))],
                condition: None
            })),
            input = r#"stage: {
                "$join":
                  {
                    "collection": "bar",
                    "joinType": "inner",
                    "pipeline":
                      [
                        {
                          "$join":
                            {
                              "collection": "baz",
                              "joinType": "inner",
                              "pipeline":
                                [
                                  {
                                    "$join":
                                      {
                                        "collection": "car",
                                        "joinType": "inner",
                                        "pipeline": [],
                                      },
                                  },
                                ],
                            },
                        },
                      ],
                  },
              }"#
        );
    }

    mod lookup_test {
        use crate::{
            air::agg_ast::ast_definitions::{
                Expression, LiteralValue, Lookup, LookupFrom, MatchExpr, MatchExpression,
                Namespace, ProjectItem, Stage, StringOrRef, UntaggedOperator,
            },
            map,
        };

        test_deserialize_stage!(
            lookup_with_no_optional_fields,
            expected = Stage::Lookup(Lookup {
                from: None,
                let_body: None,
                pipeline: vec![],
                as_var: "as_var".to_string()
            }),
            input = r#"stage: {"$lookup": {"pipeline": [], "as": "as_var"}}"#
        );
        test_deserialize_stage!(
            lookup_from_collection,
            expected = Stage::Lookup(Lookup {
                from: Some(LookupFrom::Collection("from_coll".to_string())),
                let_body: None,
                pipeline: vec![],
                as_var: "as_var".to_string()
            }),
            input = r#"stage: {"$lookup": {"from": "from_coll", "pipeline": [], "as": "as_var"}}"#
        );
        test_deserialize_stage!(
            lookup_from_namespace,
            expected = Stage::Lookup(Lookup {
                from: Some(LookupFrom::Namespace(Namespace {
                    db: "from_db".to_string(),
                    coll: "from_coll".to_string()
                })),
                let_body: None,
                pipeline: vec![],
                as_var: "as_var".to_string()
            }),
            input = r#"stage: {"$lookup": {"from": {"db": "from_db", "coll": "from_coll"}, "pipeline": [], "as": "as_var"}}"#
        );
        test_deserialize_stage!(
            lookup_with_single_let_var,
            expected = Stage::Lookup(Lookup {
                from: Some(LookupFrom::Namespace(Namespace {
                    db: "from_db".to_string(),
                    coll: "from_coll".to_string()
                })),
                let_body: Some(map! {
                    "x".to_string() => Expression::Literal(LiteralValue::Integer(9))
                }),
                pipeline: vec![],
                as_var: "as_var".to_string()
            }),
            input = r#"stage: {"$lookup": {
                "from": {"db": "from_db", "coll": "from_coll"},
                "let": {"x": 9},
                "pipeline": [], 
                "as": "as_var"
            }}"#
        );
        test_deserialize_stage!(
            lookup_with_multiple_let_vars,
            expected = Stage::Lookup(Lookup {
                from: Some(LookupFrom::Namespace(Namespace {
                    db: "from_db".to_string(),
                    coll: "from_coll".to_string()
                })),
                let_body: Some(map! {
                    "x".to_string() => Expression::Literal(LiteralValue::Integer(9)),
                    "y".to_string() => Expression::StringOrRef(StringOrRef::FieldRef("z".to_string())),
                }),
                pipeline: vec![],
                as_var: "as_var".to_string()
            }),
            input = r#"stage: {"$lookup": {
                "from": {"db": "from_db", "coll": "from_coll"},
                "let": {
                    "x": 9,
                    "y": "$z"
                },
                "pipeline": [], 
                "as": "as_var"
            }}"#
        );

        test_deserialize_stage!(
            lookup_with_pipeline,
            expected = Stage::Lookup(Lookup {
                from: Some(LookupFrom::Namespace(Namespace {
                    db: "db".to_string(),
                    coll: "bar".to_string()
                })),
                let_body: Some(map! {
                    "foo_b_0".to_string() => Expression::StringOrRef(StringOrRef::FieldRef("b".to_string())),
                }),
                pipeline: vec![
                    Stage::Match(MatchExpression::Expr(MatchExpr {
                        expr: Box::new(Expression::UntaggedOperator(UntaggedOperator {
                            op: "$eq".to_string(),
                            args: vec![
                                Expression::StringOrRef(StringOrRef::Variable(
                                    "foo_b_0".to_string()
                                )),
                                Expression::StringOrRef(StringOrRef::FieldRef("b".to_string()))
                            ]
                        }))
                    })),
                    Stage::Project(map! {
                        "_id".to_string() => ProjectItem::Exclusion,
                        "a".to_string() => ProjectItem::Inclusion,
                    })
                ],
                as_var: "__subquery_result_0".to_string()
            }),
            input = r#"stage: {
                "$lookup":
                  {
                    "from": { "db": "db", "coll": "bar" },
                    "let": { "foo_b_0": "$b" },
                    "pipeline":
                      [
                        { "$match": { "$expr": { "$eq": ["$$foo_b_0", "$b"] } } },
                        { "$project": { "_id": 0, "a": 1 } },
                      ],
                    "as": "__subquery_result_0"
              }}"#
        );
    }

    mod group_test {
        use crate::{
            air::agg_ast::ast_definitions::{
                Expression, Group, GroupAccumulator, GroupAccumulatorExpr, LiteralValue, Stage,
                StringOrRef,
            },
            map,
        };

        test_deserialize_stage!(
            group_null_id_no_acc,
            expected = Stage::Group(Group {
                keys: Expression::Literal(LiteralValue::Null),
                aggregations: map! {}
            }),
            input = r#"stage: {"$group": {
                "_id": null,
            }}"#
        );

        test_deserialize_stage!(
            group_with_single_acc,
            expected = Stage::Group(Group {
                keys: Expression::Literal(LiteralValue::Null),
                aggregations: map! {
                    "acc".to_string() => GroupAccumulator {
                        function: "$sqlSum".to_string(),
                        expr: GroupAccumulatorExpr::SqlAccumulator { distinct: true, var: Box::new(Expression::StringOrRef(StringOrRef::FieldRef("a".to_string()))) }
                    }
                }
            }),
            input = r#"stage: {
                "$group":
                  {
                    "_id": null,
                    "acc": { "$sqlSum": { "var": "$a", "distinct": true } },
                  }
              }"#
        );

        test_deserialize_stage!(
            group_with_keys_and_multiple_acc,
            expected = Stage::Group(Group {
                keys: Expression::Document(map! {
                    "a".to_string() => Expression::StringOrRef(StringOrRef::FieldRef("a".to_string()))
                },),
                aggregations: map! {
                    "acc_one".to_string() => GroupAccumulator {
                        function: "$sqlSum".to_string(),
                        expr: GroupAccumulatorExpr::SqlAccumulator { distinct: true, var: Box::new(Expression::StringOrRef(StringOrRef::FieldRef("a".to_string()))) },
                    },
                    "acc_two".to_string() => GroupAccumulator {
                        function: "$sqlAvg".to_string(),
                        expr: GroupAccumulatorExpr::SqlAccumulator { distinct: true, var: Box::new(Expression::StringOrRef(StringOrRef::FieldRef("b".to_string()))) },
                    },
                }
            }),
            input = r#"stage: {
                "$group":
                {
                    "_id": {"a": "$a"},
                    "acc_one": { "$sqlSum": { "var": "$a", "distinct": true } },
                    "acc_two": { "$sqlAvg": { "var": "$b", "distinct": true } },
                }
            }"#
        );

        test_deserialize_stage!(
            group_with_non_sql_acc,
            expected = Stage::Group(Group {
                keys: Expression::Literal(LiteralValue::Null),
                aggregations: map! {
                    "acc".to_string() => GroupAccumulator {
                        function: "$addToSet".to_string(),
                        expr: GroupAccumulatorExpr::NonSqlAccumulator(Expression::StringOrRef(StringOrRef::FieldRef("a".to_string()))),
                    }
                }
            }),
            input = r#"stage: { "$group": { "_id": null, "acc": { "$addToSet": "$a" } } }"#
        );
    }
}

mod expression_test {
    use crate::air::agg_ast::ast_definitions::Expression;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Deserialize)]
    struct TestExpr {
        expr: Expression,
    }

    mod literal {
        use crate::air::agg_ast::ast_definitions::{Expression, LiteralValue};

        test_deserialize_expr!(
            null,
            expected = Expression::Literal(LiteralValue::Null),
            input = r#"expr: null"#
        );

        test_deserialize_expr!(
            boolean_true,
            expected = Expression::Literal(LiteralValue::Boolean(true)),
            input = r#"expr: true"#
        );

        test_deserialize_expr!(
            boolean_false,
            expected = Expression::Literal(LiteralValue::Boolean(false)),
            input = r#"expr: false"#
        );

        test_deserialize_expr!(
            int,
            expected = Expression::Literal(LiteralValue::Integer(1)),
            input = r#"expr: 1"#
        );

        test_deserialize_expr!(
            long,
            expected = Expression::Literal(LiteralValue::Long(2147483648)),
            input = r#"expr: 2147483648"#
        );

        test_deserialize_expr!(
            double,
            expected = Expression::Literal(LiteralValue::Double(1.5)),
            input = r#"expr: 1.5"#
        );
    }

    mod string_or_ref {
        use crate::air::agg_ast::ast_definitions::{Expression, StringOrRef};

        test_deserialize_expr!(
            string,
            expected = Expression::StringOrRef(StringOrRef::String("yes".to_string())),
            input = r#"expr: "yes""#
        );

        test_deserialize_expr!(
            simple_field_ref,
            expected = Expression::StringOrRef(StringOrRef::FieldRef("a".to_string())),
            input = r#"expr: "$a""#
        );

        test_deserialize_expr!(
            nested_field_ref,
            expected = Expression::StringOrRef(StringOrRef::FieldRef("a.b.c".to_string())),
            input = r#"expr: "$a.b.c""#
        );

        test_deserialize_expr!(
            variable,
            expected = Expression::StringOrRef(StringOrRef::Variable("v".to_string())),
            input = r#"expr: "$$v""#
        );
    }

    mod array {
        use crate::air::agg_ast::ast_definitions::{Expression, LiteralValue, StringOrRef};

        test_deserialize_expr!(
            empty,
            expected = Expression::Array(vec![]),
            input = r#"expr: []"#
        );

        test_deserialize_expr!(
            singleton,
            expected = Expression::Array(vec![Expression::Literal(LiteralValue::Integer(1))]),
            input = r#"expr: [1]"#
        );

        test_deserialize_expr!(
            multiple_elements,
            expected = Expression::Array(vec![
                Expression::Literal(LiteralValue::Integer(1)),
                Expression::StringOrRef(StringOrRef::String("yes".to_string())),
                Expression::Array(vec![
                    Expression::Literal(LiteralValue::Boolean(true)),
                    Expression::Literal(LiteralValue::Double(4.1)),
                ]),
            ]),
            input = r#"expr: [1, "yes", [true, 4.1]]"#
        );
    }

    mod document {
        use crate::{
            air::agg_ast::ast_definitions::{Expression, LiteralValue, StringOrRef},
            map,
        };

        test_deserialize_expr!(
            empty,
            expected = Expression::Document(map! {}),
            input = r#"expr: {}"#
        );

        test_deserialize_expr!(
            singleton,
            expected = Expression::Document(
                map! {"a".to_string() => Expression::Literal(LiteralValue::Integer(1))}
            ),
            input = r#"expr: {"a": 1}"#
        );

        test_deserialize_expr!(
            multiple_elements,
            expected = Expression::Document(map! {
                "a".to_string() => Expression::Literal(LiteralValue::Integer(1)),
                "b".to_string() => Expression::StringOrRef(StringOrRef::String("two".to_string())),
                "c".to_string() => Expression::Document(map! {
                    "x".to_string() => Expression::Literal(LiteralValue::Boolean(false))
                }),
            }),
            input = r#"expr: {"a": 1, "b": "two", "c": {"x": false}}"#
        );

        test_deserialize_expr!(
            similar_to_op_but_no_dollarx,
            expected = Expression::Document(map! {
                "notOp".to_string() => Expression::Array(vec![
                    Expression::Literal(LiteralValue::Integer(1)),
                    Expression::Literal(LiteralValue::Integer(2)),
                    Expression::Literal(LiteralValue::Integer(3)),
                ])
            }),
            input = r#"expr: {"notOp": [1, 2, 3]}"#
        );
    }

    mod tagged_operators {
        use crate::{
            air::agg_ast::ast_definitions::{
                Convert, Expression, GetField, Let, Like, LiteralValue, ProjectItem, Reduce,
                SetField, SqlConvert, SqlDivide, Stage, StringOrRef, Subquery, SubqueryComparison,
                SubqueryExists, Switch, SwitchCase, TaggedOperator, UnsetField,
            },
            map,
        };

        test_deserialize_expr!(
            get_field,
            expected = Expression::TaggedOperator(TaggedOperator::GetField(GetField {
                field: "x".to_string(),
                input: Box::new(Expression::Document(map! {
                    "x".to_string() => Expression::Literal(LiteralValue::Integer(1))
                }))
            })),
            input = r#"expr: {"$getField": {"field": "x", "input": {"x": 1}}}"#
        );

        test_deserialize_expr!(
            set_field,
            expected = Expression::TaggedOperator(TaggedOperator::SetField(SetField {
                field: "x".to_string(),
                input: Box::new(Expression::Document(map! {
                    "x".to_string() => Expression::Literal(LiteralValue::Integer(1))
                })),
                value: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "new".to_string()
                )))
            })),
            input = r#"expr: {"$setField": {"field": "x", "input": {"x": 1}, "value": "new"}}"#
        );

        test_deserialize_expr!(
            unset_field,
            expected = Expression::TaggedOperator(TaggedOperator::UnsetField(UnsetField {
                field: "x".to_string(),
                input: Box::new(Expression::Document(map! {
                    "x".to_string() => Expression::Literal(LiteralValue::Integer(1))
                }))
            })),
            input = r#"expr: {"$unsetField": {"field": "x", "input": {"x": 1}}}"#
        );

        test_deserialize_expr!(
            switch,
            expected = Expression::TaggedOperator(TaggedOperator::Switch(Switch {
                branches: vec![
                    SwitchCase {
                        case: Box::new(Expression::StringOrRef(StringOrRef::FieldRef(
                            "a".to_string()
                        ))),
                        then: Box::new(Expression::Literal(LiteralValue::Integer(10))),
                    },
                    SwitchCase {
                        case: Box::new(Expression::StringOrRef(StringOrRef::FieldRef(
                            "b".to_string()
                        ))),
                        then: Box::new(Expression::Literal(LiteralValue::Integer(20))),
                    },
                ],
                default: Box::new(Expression::Literal(LiteralValue::Null))
            })),
            input = r#"expr: {"$switch": {
                                "branches": [
                                    {"case": "$a", "then": 10},
                                    {"case": "$b", "then": 20},
                                ],
                                "default": null
            }}"#
        );

        test_deserialize_expr!(
            let_expr,
            expected = Expression::TaggedOperator(TaggedOperator::Let(Let {
                vars: map! {
                    "a".to_string() => Expression::Literal(LiteralValue::Integer(1)),
                    "b".to_string() => Expression::Literal(LiteralValue::Integer(2)),
                },
                inside: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "body".to_string()
                )))
            })),
            input = r#"expr: {"$let": {
                                "vars": {"a": 1, "b": 2},
                                "in": "body"
            }}"#
        );

        test_deserialize_expr!(
            sql_convert,
            expected = Expression::TaggedOperator(TaggedOperator::SqlConvert(SqlConvert {
                input: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "1".to_string()
                ))),
                to: "int".to_string(),
                on_null: Box::new(Expression::Literal(LiteralValue::Null)),
                on_error: Box::new(Expression::Literal(LiteralValue::Null)),
            })),
            input = r#"expr: {"$sqlConvert": {
                                "input": "1",
                                "to": "int",
                                "onNull": null,
                                "onError": null
            }}"#
        );

        test_deserialize_expr!(
            convert,
            expected = Expression::TaggedOperator(TaggedOperator::Convert(Convert {
                input: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "1".to_string()
                ))),
                to: "int".to_string(),
                on_null: Box::new(Expression::Literal(LiteralValue::Null)),
                on_error: Box::new(Expression::Literal(LiteralValue::Null)),
            })),
            input = r#"expr: {"$convert": {
                                "input": "1",
                                "to": "int",
                                "onNull": null,
                                "onError": null
            }}"#
        );

        test_deserialize_expr!(
            like_with_escape,
            expected = Expression::TaggedOperator(TaggedOperator::Like(Like {
                input: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "x*yz".to_string()
                ))),
                pattern: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "x!*.*".to_string()
                ))),
                escape: Some("!".to_string())
            })),
            input = r#"expr: {"$like": {
                                "input": "x*yz",
                                "pattern": "x!*.*",
                                "escape": "!"
            }}"#
        );

        test_deserialize_expr!(
            like_without_escape,
            expected = Expression::TaggedOperator(TaggedOperator::Like(Like {
                input: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "x*yz".to_string()
                ))),
                pattern: Box::new(Expression::StringOrRef(StringOrRef::String(
                    "x!*.*".to_string()
                ))),
                escape: None
            })),
            input = r#"expr: {"$like": {
                                "input": "x*yz",
                                "pattern": "x!*.*"
            }}"#
        );

        test_deserialize_expr!(
            sql_divide,
            expected = Expression::TaggedOperator(TaggedOperator::SqlDivide(SqlDivide {
                dividend: Box::new(Expression::StringOrRef(StringOrRef::FieldRef(
                    "a".to_string()
                ))),
                divisor: Box::new(Expression::Literal(LiteralValue::Integer(2))),
                on_error: Box::new(Expression::Literal(LiteralValue::Null)),
            })),
            input = r#"expr: {"$sqlDivide": {
                                "dividend": "$a",
                                "divisor": 2,
                                "onError": null
            }}"#
        );

        test_deserialize_expr!(
            reduce,
            expected = Expression::TaggedOperator(TaggedOperator::Reduce(Reduce {
                input: Box::new(Expression::StringOrRef(StringOrRef::FieldRef(
                    "a".to_string()
                ))),
                initial_value: Box::new(Expression::Literal(LiteralValue::Integer(2))),
                inside: Box::new(Expression::Literal(LiteralValue::Null)),
            })),
            input = r#"expr: {"$reduce": {
                                "input": "$a",
                                "initialValue": 2,
                                "in": null
            }}"#
        );

        test_deserialize_expr!(
            sql_subquery,
            expected = Expression::TaggedOperator(TaggedOperator::Subquery(Subquery {
                db: Some("foo".to_string()),
                collection: Some("bar".to_string()),
                let_bindings: None,
                output_path: Some(vec!["x".to_string()]),
                pipeline: vec![Stage::Project(
                    map! {"x".to_string() => ProjectItem::Inclusion}
                )]
            })),
            input = r#"expr: {"$subquery": {
                            "db": "foo",
                            "collection": "bar",
                            "outputPath": ["x"],
                            "pipeline": [
                              {
                                "$project": {
                                  "x": 1
                                }
                              }
                            ]
                          }}"#
        );

        test_deserialize_expr!(
            sql_subquery_comparison,
            expected = Expression::TaggedOperator(TaggedOperator::SubqueryComparison(
                SubqueryComparison {
                    op: "eq".to_string(),
                    modifier: "all".to_string(),
                    arg: Box::new(Expression::Literal(LiteralValue::Integer(42))),
                    subquery: Subquery {
                        db: Some("foo".to_string()),
                        collection: Some("bar".to_string()),
                        let_bindings: None,
                        output_path: Some(vec!["x".to_string()]),
                        pipeline: vec![
                            Stage::Documents(vec![]),
                            Stage::Project(map! {"x".to_string() => ProjectItem::Inclusion})
                        ]
                    }
                    .into()
                }
            )),
            input = r#"expr: {"$subqueryComparison": {
                            "op": "eq",
                            "modifier": "all",
                            "arg": 42,
                            "subquery": {
                                "db": "foo",
                                "collection": "bar",
                                "outputPath": ["x"],
                                "pipeline": [
                                    {"$documents": []},
                                    {
                                        "$project": {
                                            "x": 1
                                        }
                                    }
                                ]
                          }}}"#
        );

        test_deserialize_expr!(
            sql_subquery_exists,
            expected = Expression::TaggedOperator(TaggedOperator::SubqueryExists(SubqueryExists {
                db: Some("foo".to_string()),
                collection: Some("bar".to_string()),
                let_bindings: None,
                pipeline: vec![Stage::Project(
                    map! {"x".to_string() => ProjectItem::Inclusion}
                )]
            })),
            input = r#"expr: {"$subqueryExists": {
                            "db": "foo",
                            "collection": "bar",
                            "pipeline": [
                              {
                                "$project": {
                                  "x": 1
                                }
                              }
                            ]
                          }}"#
        );
    }

    mod untagged_operators {
        use crate::air::agg_ast::ast_definitions::{
            Expression, LiteralValue, StringOrRef, UntaggedOperator,
        };

        test_deserialize_expr!(
            one_argument_non_array,
            expected = Expression::UntaggedOperator(UntaggedOperator {
                op: "$sqlSqrt".to_string(),
                args: vec![Expression::StringOrRef(StringOrRef::FieldRef(
                    "x".to_string()
                ))]
            }),
            input = r#"expr: {"$sqlSqrt": "$x"}"#
        );

        test_deserialize_expr!(
            one_argument,
            expected = Expression::UntaggedOperator(UntaggedOperator {
                op: "$sqlSqrt".to_string(),
                args: vec![Expression::StringOrRef(StringOrRef::FieldRef(
                    "x".to_string()
                ))]
            }),
            input = r#"expr: {"$sqlSqrt": ["$x"]}"#
        );

        test_deserialize_expr!(
            multiple_arguments,
            expected = Expression::UntaggedOperator(UntaggedOperator {
                op: "$add".to_string(),
                args: vec![
                    Expression::StringOrRef(StringOrRef::FieldRef("x".to_string())),
                    Expression::StringOrRef(StringOrRef::FieldRef("y".to_string())),
                    Expression::StringOrRef(StringOrRef::FieldRef("z".to_string())),
                ]
            }),
            input = r#"expr: {"$add": ["$x", "$y", "$z"]}"#
        );

        test_deserialize_expr!(
            literal,
            expected = Expression::UntaggedOperator(UntaggedOperator {
                op: "$literal".to_string(),
                args: vec![Expression::Literal(LiteralValue::Integer(1))]
            }),
            input = r#"expr: {"$literal": 1}"#
        );
    }
}
