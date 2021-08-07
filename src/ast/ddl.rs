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

use super::{CqlType, Expression, Projection, Property, QualifiedName};

/// CREATE KEYSPACE statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateKeyspaceStatement {
    pub keyspace_name: String,
    pub attributes: Vec<Property>,
    pub if_not_exists: bool,
}

/// CREATE TABLE statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateTableStatement {
    pub name: QualifiedName,
    pub if_not_exists: bool,
    pub column_definitions: Vec<(String, CqlType)>,
    pub static_columns: Vec<String>,
    /// Partition keys here is defined as Vec<Vec<String>>,
    /// since the statement can define partition keys in two
    /// different places: `column_name type PRIMARY KEY` and
    /// `PRIMARY KEY (...)`.
    ///
    /// Only one partition keys should be defined,
    /// so `partition_keys.len() > 1` is illegal.
    pub partition_keys: Vec<Vec<String>>,
    pub clustering_columns: Vec<String>,
    pub compact_storage: bool,
    pub clustering_order: Vec<(String, bool)>,
    pub table_properties: Vec<Property>,
}

/// CREATE (CUSTOM)? INDEX statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateIndexStatement {
    pub index_name: Option<String>,
    pub table_name: QualifiedName,
    pub if_not_exists: bool,
    pub is_custom: bool,
    pub index_targets: Vec<(String, IndexType)>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum IndexType {
    Simple,
    Values,
    Keys,
    KeysAndValues,
    Full,
}

/// CREATE TYPE statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateTypeStatement {
    pub name: QualifiedName,
    pub if_not_exists: bool,
    pub field_definitions: Vec<(String, CqlType)>,
}

/// CREATE MATERIALIZED VIEW statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateMaterializedViewStatement {
    pub name: QualifiedName,
    pub base_table: QualifiedName,
    pub if_not_exists: bool,
    pub projection: Projection,
    /// WHERE clause
    pub selection: Option<Expression>,
    pub partition_keys: Vec<String>,
    pub clustering_columns: Vec<String>,
    pub compact_storage: bool,
    pub clustering_order: Vec<(String, bool)>,
    pub view_properties: Vec<Property>,
}
