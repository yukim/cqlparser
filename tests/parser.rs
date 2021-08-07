// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cqlparser::ast::*;
use cqlparser::Parser;

#[test]
fn test_create() {
    let test_cases = [
        (
            "CREATE KEYSPACE ks WITH prop = 2",
            Ok(vec![CqlStatement::CreateKeyspace(
                CreateKeyspaceStatement {
                    keyspace_name: String::from("ks"),
                    if_not_exists: false,
                    attributes: vec![Property::new(
                        String::from("prop"),
                        Literal::Constant(Constant::Integer(2)),
                    )],
                },
            )]),
        ),
        (
            "CREATE TABLE ks.test (key int, values set<text>, PRIMARY KEY ((key))) WITH prop = 2",
            Ok(vec![CqlStatement::CreateTable(CreateTableStatement {
                name: QualifiedName::new(Some(String::from("ks")), String::from("test")),
                if_not_exists: false,
                column_definitions: vec![
                    (String::from("key"), CqlType::Native(NativeDataType::Int)),
                    (
                        String::from("values"),
                        CqlType::Collection(CollectionType::Set(Box::new(CqlType::Native(
                            NativeDataType::Text,
                        )))),
                    ),
                ],
                static_columns: vec![],
                partition_keys: vec![vec![String::from("key")]],
                clustering_columns: vec![],
                compact_storage: false,
                clustering_order: vec![],
                table_properties: vec![Property::new(
                    String::from("prop"),
                    Literal::Constant(Constant::Integer(2)),
                )],
            })]),
        ),
        (
            "CREATE MATERIALIZED VIEW cycling.cyclist_by_age 
            AS SELECT age, name, country 
            FROM cycling.cyclist_mv 
            WHERE age IS NOT NULL AND cid IS NOT NULL 
            PRIMARY KEY (age, cid)
            WITH caching = { 'keys' : 'ALL', 'rows_per_partition' : '100' }
            AND comment = 'Based on table cyclist' ;",
            Ok(vec![CqlStatement::CreateMaterializedView(
                CreateMaterializedViewStatement {
                    name: QualifiedName::new(
                        Some(String::from("cycling")),
                        String::from("cyclist_by_age"),
                    ),
                    base_table: QualifiedName::new(
                        Some(String::from("cycling")),
                        String::from("cyclist_mv"),
                    ),
                    if_not_exists: false,
                    projection: Projection::Selectors(vec![
                        Selector::new(Expression::Identifier(String::from("age")), None),
                        Selector::new(Expression::Identifier(String::from("name")), None),
                        Selector::new(Expression::Identifier(String::from("country")), None),
                    ]),
                    selection: Some(Expression::BinaryOp(BinaryOp::new(
                        Box::new(Expression::BinaryOp(BinaryOp::new(
                            Box::new(Expression::Identifier(String::from("age"))),
                            Operator::IsNot,
                            Box::new(Expression::Value(Literal::Null)),
                        ))),
                        Operator::And,
                        Box::new(Expression::BinaryOp(BinaryOp::new(
                            Box::new(Expression::Identifier(String::from("cid"))),
                            Operator::IsNot,
                            Box::new(Expression::Value(Literal::Null)),
                        ))),
                    ))),
                    partition_keys: vec![String::from("age")],
                    clustering_columns: vec![String::from("cid")],
                    compact_storage: false,
                    clustering_order: Vec::new(),
                    view_properties: vec![
                        Property::new(
                            String::from("caching"),
                            Literal::Map(vec![
                                (
                                    Expression::Value(Literal::Constant(Constant::StringLiteral(
                                        String::from("keys"),
                                    ))),
                                    Expression::Value(Literal::Constant(Constant::StringLiteral(
                                        String::from("ALL"),
                                    ))),
                                ),
                                (
                                    Expression::Value(Literal::Constant(Constant::StringLiteral(
                                        String::from("rows_per_partition"),
                                    ))),
                                    Expression::Value(Literal::Constant(Constant::StringLiteral(
                                        String::from("100"),
                                    ))),
                                ),
                            ]),
                        ),
                        Property::new(
                            String::from("comment"),
                            Literal::Constant(Constant::StringLiteral(String::from(
                                "Based on table cyclist",
                            ))),
                        ),
                    ],
                },
            )]),
        ),
    ];
    for test in &test_cases {
        let p = Parser::new(test.0);
        assert_eq!(p.parse(), test.1);
    }
}

#[test]
fn test_select_statements() {
    let test_cases = [
        (
            "SELECT * FROM ks.tbl",
            Ok(vec![CqlStatement::Select(SelectStatement {
                table_name: QualifiedName::new(Some(String::from("ks")), String::from("tbl")),
                projection: Projection::Wildcard,
                selection: None,
                is_json: false,
                is_distinct: false,
                per_partition_limit: None,
                limit: None,
                allow_filtering: false,
            })]),
        ),
        (
            "SELECT * FROM ks.tbl WHERE key = 1",
            Ok(vec![CqlStatement::Select(SelectStatement {
                table_name: QualifiedName::new(Some(String::from("ks")), String::from("tbl")),
                projection: Projection::Wildcard,
                selection: Some(Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::Identifier(String::from("key"))),
                    Operator::Equal,
                    Box::new(Expression::Value(Literal::Constant(Constant::Integer(1)))),
                ))),
                is_json: false,
                is_distinct: false,
                per_partition_limit: None,
                limit: None,
                allow_filtering: false,
            })]),
        ),
        (
            "SELECT col1, col2 as \"col_A\" FROM tbl LIMIT 10 ALLOW FILTERING",
            Ok(vec![CqlStatement::Select(SelectStatement {
                table_name: QualifiedName::new(None, String::from("tbl")),
                projection: Projection::Selectors(vec![
                    Selector::new(Expression::Identifier(String::from("col1")), None),
                    Selector::new(
                        Expression::Identifier(String::from("col2")),
                        Some(String::from("col_A")),
                    ),
                ]),
                selection: None,
                is_json: false,
                is_distinct: false,
                per_partition_limit: None,
                limit: Some(Literal::Constant(Constant::Integer(10))),
                allow_filtering: true,
            })]),
        ),
    ];
    for test in &test_cases {
        let p = Parser::new(test.0);
        assert_eq!(p.parse(), test.1);
    }
}

#[test]
fn test_update_statements() {
    let test_cases = [(
        "UPDATE tbl SET col1 = 'text', col2 = 1 WHERE k = 1",
        Ok(vec![CqlStatement::Update(UpdateStatement {
            table: QualifiedName::new(None, String::from("tbl")),
            assignments: vec![
                Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::Identifier(String::from("col1"))),
                    Operator::Equal,
                    Box::new(Expression::Value(Literal::Constant(
                        Constant::StringLiteral(String::from("text")),
                    ))),
                )),
                Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::Identifier(String::from("col2"))),
                    Operator::Equal,
                    Box::new(Expression::Value(Literal::Constant(Constant::Integer(1)))),
                )),
            ],
            selection: Expression::BinaryOp(BinaryOp::new(
                Box::new(Expression::Identifier(String::from("k"))),
                Operator::Equal,
                Box::new(Expression::Value(Literal::Constant(Constant::Integer(1)))),
            )),
            if_exists: false,
            timestamp: None,
            time_to_live: None,
        })]),
    )];
    for test in &test_cases {
        let p = Parser::new(test.0);
        assert_eq!(p.parse(), test.1);
    }
}
