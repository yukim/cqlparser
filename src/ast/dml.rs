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

/// # INSERT statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct InsertStatement {
    pub table: QualifiedName,
    pub values: InsertMethod,
    pub if_not_exists: bool,
    /// timestamp value
    /// Can be `Literal::Integer` or `Literal::Binding`
    pub timestamp: Option<Literal>,
    pub time_to_live: Option<Literal>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum InsertMethod {
    Normal {
        columns: Vec<Expression>,
        values: Vec<Expression>,
    },
    Json {
        value: String,
        default_behavior: JsonBehavior,
    },
}

impl InsertMethod {
    pub fn normal(columns: Vec<Expression>, values: Vec<Expression>) -> Self {
        InsertMethod::Normal { columns, values }
    }

    pub fn json(value: String, default_behavior: JsonBehavior) -> Self {
        InsertMethod::Json {
            value,
            default_behavior,
        }
    }
}

/// # Default Json behavior in `INSERT INTO tbl JSON` statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum JsonBehavior {
    Unset,
    Null,
}

/// UPDATE statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateStatement {
    pub table: QualifiedName,
    pub if_exists: bool,
    pub assignments: Vec<Expression>,
    pub selection: Expression,
    /// timestamp value
    /// Can be `Literal::Integer` or `Literal::Binding`
    pub timestamp: Option<Literal>,
    pub time_to_live: Option<Literal>,
}
