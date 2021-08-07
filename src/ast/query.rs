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

use super::{Expression, Literal, QualifiedName};

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectStatement {
    /// FROM table name
    pub table_name: QualifiedName,
    pub projection: Projection,
    /// WHERE clause
    pub selection: Option<Expression>,
    /// true when the SELECT statement begins with `SELECT JSON columns...`
    pub is_json: bool,
    /// true when the SELECT statement contains `DISTINCT`
    pub is_distinct: bool,
    /// Per partition limit
    pub per_partition_limit: Option<Literal>,
    /// limit
    pub limit: Option<Literal>,
    /// true when the SELECT statement contains `ALLOW FILTERING`
    pub allow_filtering: bool,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum Projection {
    /// Wildcard(`*`) projection
    Wildcard,
    /// List of selectors
    Selectors(Vec<Selector>),
}

/// Selector is an expression in SELECT clause to be selected for the result set.
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct Selector {
    selectable: Expression,
    /// alias name if any
    alias: Option<String>,
}

impl Selector {
    /// Creates new selector with given selectable and optional alias name
    pub fn new(selectable: Expression, alias: Option<String>) -> Self {
        Selector { selectable, alias }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereClause {
    relations: Vec<()>,
    custom_index_expressions: Vec<String>,
}
