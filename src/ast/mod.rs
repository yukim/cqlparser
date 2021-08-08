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

//! # `cqlparser` Abstract Syntax Tree
//!
//! When CQL string is parsed, `Parser` will produce a result
//! containing multiple `CqlStatement`s.
//!
//! ## Root node
//!
//! `CqlStatements`
//!
//! ## Identifier
//!
//! In CQL, there are three types of identifiers: column identifier, non-column identifier and field identifier.
//! - Non-column identifier is generic identifier
//! - Column identifier identifies a CQL column definition
//!     - difference from Non-column identifier is that this can be cached using its name to save spaces ("interned")
//! - Field identifier identifies a field in UDT
//!
//! All of these are token types of:
//! - Identifier
//! - Quoted name
//! - Unreserved keywords
//!     - this include unreserved keyword and native data types
//!
//! ### Simple term
//!
//! a `simple term` is one of:
//! - value: `Expression::Value`
//! - function: `Expression::Function`
//! - type cast: `(type) simpleTerm` `Expression::Cast`
//!
//! ### Value
//!
//! Implemented as `Expression::Value`
//!
//! `value` is one of:
//! - constant
//! - collection literal
//! - user type literal
//! - tuple literal
//! - NULL
//! - binding variable (:xxx, ?)
//!
//! These kinds are implemented as `Literal`.
//!
//! ### Function
//!
//! Implemented as `Expression::Function`

use std::convert::TryFrom;

use crate::error::ParseError;
use crate::{Keyword, Token, TokenType};

mod ddl;
mod dml;
mod query;

pub use ddl::*;
pub use dml::*;
pub use query::*;

/// # Qualified name
///
/// Qualified name is the name of the keyspace-prefixed elements,
/// such as table name, index name, function names, etc.
///
/// `keyspace` part can be omittedm, by providing `None` to `keyspace`.
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct QualifiedName {
    pub keyspace: Option<String>,
    pub name: String,
}

impl QualifiedName {
    pub fn new(keyspace: Option<String>, name: String) -> Self {
        QualifiedName { keyspace, name }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct UnaryOp<A, R> {
    operator: R,
    operand: A,
}

impl<A, R> UnaryOp<A, R> {
    pub fn new(operator: R, operand: A) -> Self {
        UnaryOp { operator, operand }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct BinaryOp<A, R> {
    left: A,
    operator: R,
    right: A,
}

impl<A, R> BinaryOp<A, R> {
    pub fn new(left: A, operator: R, right: A) -> Self {
        BinaryOp {
            left,
            operator,
            right,
        }
    }
}

/// Literal
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum Literal {
    /// Constant literals
    Constant(Constant),

    /// NULL literal
    Null,

    /// List collection literal
    List(Vec<Expression>),

    /// ## Set literal
    /// Example: {1, 2, 3}
    Set,

    /// ## Map literal
    /// Example: {key1: 1, key2: 2}
    Map(Vec<(Expression, Expression)>),

    /// ## Tuple literal
    Tuple(Vec<Expression>),

    /// ## User Defined Type
    UserType,

    /// ## Binding variable
    ///
    /// Binding variables in CQL are in two form:
    /// - ? (positional)
    /// - :name (with name)
    Binding(Option<String>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum Constant {
    StringLiteral(String),
    Integer(u32),
    Float(String),
    Boolean(bool),
    Duration(String),
    /// ## UUID literal
    ///
    /// Note: This library does not convert UUID string to 128-bit UUID,
    /// and it may not be a valid UUID.
    /// It is a user's responsibility to parse UUID string.
    UUID(String),
    /// ## Binary data
    Bytes(Vec<u8>),
    /// ## Not a number
    NaN,
    /// ## Infinity
    Infinity,
}

/// Operators
#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum Operator {
    /// '+': arithmetic operator for addition
    Plus,
    /// '-': arithmetic operator for subtraction
    Minus,
    /// '*': arithmetic operator for multiplication
    Multiply,
    /// '/': arithmetic operator for division
    Divide,
    /// '%': arithmetic operator for modulus
    Modulus,
    /// '.': field selection operator
    Dot,
    /// '[': collection selection operator
    LBracket,

    /// '=': relationship operator for equality
    Equal,
    /// '!=': relationship operator for inequality
    NotEqual,
    /// '<': relationship operator for comparison
    LessThan,
    /// '<=': relationship operator for comparison
    LessThanOrEqual,
    /// '>': relationship operator for comparison
    GreaterThan,
    /// '>=': relationship operator for comparison
    GreaterThanOrEqual,
    /// 'IN': relationship operator for comparison
    In,
    /// 'CONTAINS': relationship operator
    Contains,
    /// 'CONTAINS KEY': relationship operator
    ContainsKey,
    /// IS NOT (NULL)
    IsNot,
    /// LIKE
    Like,

    /// AND
    And,
}

impl TryFrom<&Token> for Operator {
    type Error = ParseError;

    fn try_from(tt: &Token) -> Result<Self, Self::Error> {
        match &tt.token_type {
            TokenType::Plus => Ok(Operator::Plus),
            TokenType::Minus => Ok(Operator::Minus),
            TokenType::Asterisk => Ok(Operator::Multiply),
            TokenType::Slash => Ok(Operator::Divide),
            TokenType::Percent => Ok(Operator::Modulus),
            TokenType::Equal => Ok(Operator::Equal),
            TokenType::NotEqual => Ok(Operator::NotEqual),
            TokenType::Gt => Ok(Operator::GreaterThan),
            TokenType::Gte => Ok(Operator::GreaterThanOrEqual),
            TokenType::Lt => Ok(Operator::LessThan),
            TokenType::Lte => Ok(Operator::LessThanOrEqual),
            TokenType::Keyword(Keyword::And) => Ok(Operator::And),
            _ => Err(ParseError::with_message(format!(
                "Cannot convert {:?} for operator!",
                tt
            ))),
        }
    }
}

/// # Expression
///
/// `Expression`s are used in the following:
/// - Projection
/// - Selection
///
/// ## Examples of selection in CQL
///
/// - `col`, `"ColName"`: identifier
/// - `writetime(col): function call
/// - `cast(col AS int)`: cast function
/// - `-1`: unary operation
/// - `1 + 1`: binary operation
/// - `udt.prop1`: UDT access
/// - `map['key']: collection access
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum Expression {
    /// # Identifier
    ///
    /// In CQL, if the string is not quoted with `"`, case is not preserved.
    Identifier(String),
    /// Unary operation
    UnaryOp(UnaryOp<Box<Expression>, Operator>),
    /// Binary operation
    BinaryOp(BinaryOp<Box<Expression>, Operator>),

    /// Literal values (UUID, numbers, string, etc)
    ///
    /// In CQL3 Parser, this is defined as one of simple terms, `value`.
    Value(Literal),
    /// Function call
    ///
    /// In CQL3 Parser, this is defined as one of simple terms, `function`.
    Function {
        /// Function name
        ///
        /// Function name consists of optional keyspace name followed by `.`, and one of the followings:
        /// - Identifier
        /// - Quoted string literal
        /// - Unreserved keywords or native data type name
        /// - `TOKEN` keyword or `COUNT` keyword
        name: Box<Expression>,
        args: Vec<Expression>,
    },
    /// `cast` function is treated differently,
    /// since the argument is in the form of `xxx AS type`.
    /// Another form of type cast is `(type) simple_term`.
    ///
    /// In CQL3 Parser, this is defined as one of simple terms.
    TypeCast(CqlType, Box<Expression>),

    /// Collection sub selection
    ///
    /// Example: map_column['key'], set_column[1..4]
    CollectionSubSelection {
        receiver: Box<Expression>,
        element: Box<Expression>,
        upto: Option<Box<Expression>>,
    },
}

impl Expression {
    /// Expression is a "Simple Term" if it is one of:
    /// - Value
    /// - Function call
    /// - Type cast
    pub fn is_simple_term(&self) -> bool {
        match self {
            Self::Value(_) | Self::Function { .. } | Self::TypeCast(_, _) => true,
            _ => false,
        }
    }
}

/// # Property
///
/// Property value is one of the following:
/// - Constant
/// - Unreserved keyword
/// - Map literal
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct Property {
    key: String,
    value: Literal,
}

impl Property {
    pub fn new(key: String, value: Literal) -> Self {
        Property { key, value }
    }
}

/// # CQL data types
///
/// In Cassnadra, there are several types of data types:
/// - Native data type (text, int, etc)
/// - Collection type (map, set, list)
/// - Tuple type
/// - User defined type
/// - Custom data type
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum CqlType {
    /// CQL native data types such as `text`, `int`, etc.
    Native(NativeDataType),
    /// CQL collection types `map`, `list`, `set`.
    Collection(CollectionType),
    /// CQL Tuple type
    Tuple(Vec<CqlType>),
    UserDefinedType(QualifiedName),
    Frozen(Box<CqlType>),
    /// Custom data type.
    ///
    /// In CQL, custom type can be specified using string literal.
    Custom(String),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum NativeDataType {
    Ascii,
    BigInt,
    Blob,
    Boolean,
    Counter,
    Decimal,
    Double,
    Duration,
    Float,
    Inet,
    Int,
    SmallInt,
    Text,
    Timestamp,
    TinyInt,
    UUID,
    Varchar,
    VarInt,
    TimeUUID,
    Date,
    Time,
}

impl From<NativeDataType> for String {
    fn from(nt: NativeDataType) -> Self {
        (match nt {
            NativeDataType::Ascii => "ascii",
            NativeDataType::BigInt => "bigint",
            NativeDataType::Blob => "blob",
            NativeDataType::Boolean => "boolean",
            NativeDataType::Counter => "counter",
            NativeDataType::Decimal => "decimal",
            NativeDataType::Double => "double",
            NativeDataType::Duration => "duration",
            NativeDataType::Float => "float",
            NativeDataType::Inet => "inet",
            NativeDataType::Int => "int",
            NativeDataType::SmallInt => "smallint",
            NativeDataType::Text => "text",
            NativeDataType::Timestamp => "timestamp",
            NativeDataType::TinyInt => "tinyint",
            NativeDataType::UUID => "uuid",
            NativeDataType::Varchar => "varchar",
            NativeDataType::VarInt => "varint",
            NativeDataType::TimeUUID => "timeuuid",
            NativeDataType::Date => "date",
            NativeDataType::Time => "time",
        })
        .to_owned()
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum CollectionType {
    Map {
        key_type: Box<CqlType>,
        value_type: Box<CqlType>,
    },
    List(Box<CqlType>),
    Set(Box<CqlType>),
}

/// Statement
#[derive(Debug, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub enum CqlStatement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete,
    Batch,
    Truncate,
    Use,
    CreateAggregate,
    CreateFunction,
    CreateIndex(CreateIndexStatement),
    CreateKeyspace(CreateKeyspaceStatement),
    CreateTable(CreateTableStatement),
    CreateTrigger,
    CreateType(CreateTypeStatement),
    CreateMaterializedView(CreateMaterializedViewStatement),
    AlterKeyspace,
    AlterTable,
    AlterType,
    AlterView,
    DropAggregate,
    DropFunction,
    DropIndex,
    DropKeyspace,
    DropTable,
    DropTrigger,
    DropType,
    DropView,
    AlterRole,
    CreateRole,
    DropRole,
    GrantRole,
    RevokeRole,
    ListPermissions,
    ListRoles,
    ListUsers,
    GrantPermissions,
    RevokePermissions,
}
