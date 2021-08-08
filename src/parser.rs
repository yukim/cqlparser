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

use std::convert::TryFrom;
use std::iter::Peekable;
use std::result::Result;

use super::ast::*;
use super::error::ParseError;
use super::lexer::*;
use super::TokenType;

pub type CqlResult = Result<CqlStatement, ParseError>;

/// Operator precedence
#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    Min,
    /// AND
    And,
    /// ==, != or `IS NOT`
    Equal,
    /// >, >=, <, or <=
    LessOrGreater,
    /// +-
    Addition,
    /// */%
    Product,
    /// !X, -X
    Prefix,
    /// function call
    Call,
}

impl From<&Token> for Precedence {
    fn from(token: &Token) -> Self {
        match &token.token_type {
            TokenType::Equal | TokenType::NotEqual | TokenType::Keyword(Keyword::Is) => {
                Precedence::Equal
            }
            TokenType::Gt | TokenType::Gte | TokenType::Lt | TokenType::Lte => {
                Precedence::LessOrGreater
            }
            TokenType::Plus | TokenType::Minus => Precedence::Addition,
            TokenType::Asterisk | TokenType::Slash | TokenType::Percent => Precedence::Product,
            TokenType::LParen => Precedence::Call,
            TokenType::Keyword(Keyword::And) => Precedence::And,
            _ => Precedence::Min,
        }
    }
}

/// Apache Cassandra CQL Parser
///
/// ## Example
///
/// ```
/// use cqlparser::Parser;
/// let parser = Parser::new("SELECT * FROM test;");
/// assert!(parser.parse().is_ok());
/// ```
pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    /// Create new `Parser` of given CQL string
    pub fn new(cql: &'a str) -> Self {
        Parser {
            lexer: Lexer::new(cql).peekable(),
        }
    }

    /// Parse CQL statements
    ///
    /// If `Parser` only parses `&str` that contains a single CQL statement,
    /// `;` at the end of the statement can be omitted.
    pub fn parse(mut self) -> Result<Vec<CqlStatement>, ParseError> {
        let mut statements = Vec::new();
        while self.peek().is_some() {
            // Skip `;` between statements
            while self.expect(TokenType::SemiColon).is_ok() {}

            // at the end of the input
            if self.peek().is_none() {
                break;
            }

            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    // Peek next token, ignoring whitespaces and comments
    fn peek(&mut self) -> Option<&(&str, Token)> {
        loop {
            if let Some((_, next)) = self.lexer.peek() {
                match next.token_type {
                    // Skip whitespaces and comments
                    TokenType::Whitespace | TokenType::Comment(_) => {
                        self.lexer.next();
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        self.lexer.peek()
    }

    // Advance to the next token, ignoring whitespaces and comments
    fn advance(&mut self) -> Option<(&str, Token)> {
        while let Some(next) = self.lexer.next() {
            match next.1.token_type {
                // Skip whitespaces and comments
                TokenType::Whitespace | TokenType::Comment(_) => continue,
                _ => return Some(next),
            }
        }
        None
    }

    fn advance_if<P: FnOnce(&&(&str, Token)) -> bool>(
        &mut self,
        predicate: P,
    ) -> Option<(&str, Token)> {
        if self.peek().filter(predicate).is_some() {
            self.advance()
        } else {
            None
        }
    }

    // Advance to next token if it matches given token type
    // Otherwise, return `ParseError`.
    fn expect(&mut self, token_type: TokenType) -> Result<(&str, Token), ParseError> {
        let next_token = self.peek();
        // save next token as String for parse error message
        let next_token_string = next_token
            .map(|(s, _)| String::from(*s))
            .unwrap_or(String::new());

        let advanced = if next_token
            .filter(|(_, t)| t.token_type == token_type)
            .is_some()
        {
            self.advance()
        } else {
            None
        };
        advanced.ok_or(ParseError::with_message(format!(
            "Expected {:?}, but was {:?}",
            &token_type, next_token_string
        )))
    }

    /// Parse a single CQL statement
    fn parse_statement(&mut self) -> CqlResult {
        while let Some((_, next)) = self.peek() {
            match &next.token_type {
                TokenType::Keyword(kw) => match kw {
                    Keyword::Select => return self.parse_select_statement(),
                    Keyword::Insert => return self.parse_insert_statement(),
                    Keyword::Update => return self.parse_update_statement(),
                    Keyword::Create => return self.create_statement(),
                    _ => return Err(ParseError::new()),
                },
                _ => break,
            }
        }
        Err(ParseError::new())
    }

    // Parse expression
    //
    // - Literals
    //    - 1, 'String', true, P1DT2H, etc
    // - Unary operation
    //    - -1, -cast(col as int)
    // - Relationship
    //    - col_a > 10
    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        // parse prefix
        let mut left = self.parse_prefix()?;

        while let Some((_, next_token)) = self.peek() {
            let next_precedence = Precedence::from(next_token);
            if precedence < next_precedence {
                // if next precedence is higher, then try to parse infix
                left = self.parse_infix(left)?;
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression, ParseError> {
        // Literal constant
        let maybe_literal_constant = self
            .parse_constant()
            .map(Literal::Constant)
            .map(Expression::Value);
        if maybe_literal_constant.is_ok() {
            return maybe_literal_constant;
        }

        if let Some((_, next)) = self.peek() {
            match &next.token_type {
                TokenType::Keyword(keyword) => match keyword {
                    // NULL
                    Keyword::Null => {
                        self.advance();
                        Ok(Expression::Value(Literal::Null))
                    }
                    // TOKEN and COUNT keywords are allowed for function name
                    Keyword::Token | Keyword::Count => {
                        self.advance();
                        Ok(Expression::Value(Literal::Null))
                    }
                    Keyword::Cast => self.parse_cast(),
                    _ => self.parse_identifier(),
                },
                TokenType::Identifier => {
                    // Maybe function
                    let maybe_function_name = self.parse_function_name();
                    if maybe_function_name.is_ok() {
                        Ok(Expression::Identifier(maybe_function_name?.name))
                    } else {
                        self.parse_identifier()
                    }
                }
                TokenType::QuotedName => self.parse_identifier(),

                // There are several cases here:
                // - type cast: `(cql_type) simple_term`
                // - tuple literal: `(1, 2, 3)`
                TokenType::LParen => {
                    self.expect(TokenType::LParen)?;
                    // Empty tuple(`()`)?
                    if self.expect(TokenType::RParen).is_ok() {
                        return Ok(Expression::Value(Literal::Tuple(Vec::new())));
                    }
                    let maybe_cql_type = self.parse_data_type();
                    // can be type cast: `(type) expr`
                    let in_paren = if maybe_cql_type.is_ok() {
                        // if next token is ')', this is type cast
                        if self.expect(TokenType::RParen).is_ok() {
                            // We need to exit in paren here
                            Ok(Expression::TypeCast(
                                maybe_cql_type?,
                                Box::new(self.parse_expression(Precedence::Min)?),
                            ))
                        } else {
                            // this cql type is just an identifier
                            maybe_cql_type.and_then(|cql_type| match cql_type {
                                CqlType::Custom(s) => Ok(Expression::Identifier(s)),
                                CqlType::UserDefinedType(n) => Ok(Expression::Identifier(n.name)),
                                CqlType::Native(nt) => Ok(Expression::Identifier(String::from(nt))),
                                _ => Err(ParseError::with_message(format!(
                                    "{:?} cannot be an identifier",
                                    cql_type
                                ))),
                            })
                        }
                    } else {
                        self.parse_expression(Precedence::Min)
                    };
                    if self
                        .peek()
                        .filter(|(_, t)| t.token_type == TokenType::Comma)
                        .is_some()
                    {
                        // tuple
                        let mut values = Vec::new();
                        values.push(in_paren?);
                        while self.expect(TokenType::Comma).is_ok() {
                            values.push(self.parse_expression(Precedence::Min)?);
                        }
                        self.expect(TokenType::RParen)?;
                        return Ok(Expression::Value(Literal::Tuple(values)));
                    }
                    self.expect(TokenType::RParen)?;
                    in_paren
                }
                TokenType::Minus => {
                    self.advance();
                    Ok(Expression::UnaryOp(UnaryOp::new(
                        Operator::Minus,
                        Box::new(self.parse_expression(Precedence::Prefix)?),
                    )))
                }
                _ => Err(ParseError::new()),
            }
        } else {
            Err(ParseError::new())
        }
    }

    fn parse_infix(&mut self, left: Expression) -> Result<Expression, ParseError> {
        if let Some((_, next)) = self.peek() {
            match &next.token_type {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Asterisk
                | TokenType::Slash
                | TokenType::Percent
                | TokenType::Equal
                | TokenType::NotEqual
                | TokenType::Gt
                | TokenType::Gte
                | TokenType::Lt
                | TokenType::Lte
                | TokenType::Keyword(Keyword::And) => self.parse_binary_operator(left),
                TokenType::Keyword(Keyword::Is) => {
                    self.expect(TokenType::Keyword(Keyword::Is))?;
                    self.expect(TokenType::Keyword(Keyword::Not))?;
                    Ok(Expression::BinaryOp(BinaryOp::new(
                        Box::new(left),
                        Operator::IsNot,
                        Box::new(self.parse_expression(Precedence::Equal)?),
                    )))
                }
                // Collection sub selection
                TokenType::LBracket => self.parse_collection_subselection(left),
                TokenType::LParen => {
                    self.advance();
                    // Function argments
                    let mut args = Vec::new();
                    // can be empty
                    if self
                        .peek()
                        .filter(|(_, t)| t.token_type != TokenType::RParen)
                        .is_some()
                    {
                        loop {
                            let value = self.parse_expression(Precedence::Min)?;
                            args.push(value);
                            if self.expect(TokenType::Comma).is_err() {
                                break;
                            }
                        }
                    }
                    self.expect(TokenType::RParen)?;
                    Ok(Expression::Function {
                        name: Box::new(left),
                        args,
                    })
                }
                _ => Err(ParseError::new()),
            }
        } else {
            Err(ParseError::new())
        }
    }

    // Parse CQL's Cast function: `cast(expr AS native_type)`
    fn parse_cast(&mut self) -> Result<Expression, ParseError> {
        self.expect(TokenType::Keyword(Keyword::Cast))?;
        self.expect(TokenType::LParen)?;
        let expr = self.parse_expression(Precedence::Min)?;
        self.expect(TokenType::Keyword(Keyword::As))?;
        let target_type = self.parse_native_data_type()?;
        self.expect(TokenType::RParen)?;

        Ok(Expression::TypeCast(target_type, Box::new(expr)))
    }

    fn parse_identifier(&mut self) -> Result<Expression, ParseError> {
        let value = self.parse_ident().ok_or(ParseError::new())?;
        Ok(Expression::Identifier(value))
    }

    fn parse_string_literal(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::StringLiteral)?;
        // Remove surrounding `'` or `$$`
        let string_value = if value.starts_with('\'') {
            // regular string literal
            value[1..value.len() - 1].to_owned()
        } else if value.starts_with('$') {
            // PG style string literal
            value[2..value.len() - 2].to_owned()
        } else {
            unreachable!();
        };

        Ok(Constant::StringLiteral(string_value))
    }

    fn parse_integer(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::Integer)?;
        // TODO value greater than 32 bit (long, bigint)
        let int_value = value.parse::<u32>().map_err(|_| ParseError::new())?;
        Ok(Constant::Integer(int_value))
    }

    fn parse_float(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::Float)?;
        Ok(Constant::Float(value.to_owned()))
    }

    fn parse_boolean(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::Boolean)?;
        let bool_value = value.parse::<bool>().map_err(|_| ParseError::new())?;
        Ok(Constant::Boolean(bool_value))
    }

    fn parse_duration(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::Duration)?;
        Ok(Constant::Duration(value.to_owned()))
    }

    fn parse_uuid(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::UUID)?;
        Ok(Constant::UUID(value.to_owned()))
    }

    fn parse_hexnumber(&mut self) -> Result<Constant, ParseError> {
        let (value, _) = self.expect(TokenType::Hexnumber)?;
        let blob = if value.len() % 2 != 0 {
            Err(ParseError::with_message(format!(
                "hex string must have a even number of length: {}",
                value
            )))
        } else {
            // skip first two chars (`0x`)
            (2..value.len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&value[i..i + 2], 16)
                        .map_err(|e| ParseError::with_message(format!("Parse int error: {}", e)))
                })
                .collect()
        }?;
        Ok(Constant::Bytes(blob))
    }

    fn parse_map_literal(&mut self) -> Result<Literal, ParseError> {
        self.expect(TokenType::LBrace)?;
        let mut map = Vec::new();
        // can be empty
        if self
            .peek()
            .filter(|(_, t)| t.token_type != TokenType::RBrace)
            .is_some()
        {
            loop {
                let key = self.parse_expression(Precedence::Min)?;
                self.expect(TokenType::Colon)?;
                let value = self.parse_expression(Precedence::Min)?;
                map.push((key, value));
                if self.expect(TokenType::Comma).is_err() {
                    break;
                }
            }
        }
        self.expect(TokenType::RBrace)?;
        Ok(Literal::Map(map))
    }

    fn parse_binary_operator(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let (_, token) = self.advance().ok_or(ParseError::new())?;
        Ok(Expression::BinaryOp(BinaryOp::new(
            Box::new(left),
            Operator::try_from(&token)?,
            Box::new(self.parse_expression(Precedence::from(&token))?),
        )))
    }

    // collectionSubSelection [Selectable.Raw receiver] returns [Selectable.Raw s]
    // @init { boolean isSlice=false; }
    // : ( t1=term ( { isSlice=true; } RANGE (t2=term)? )?
    //   | RANGE { isSlice=true; } t2=term
    //   ) {
    //       $s = isSlice
    //          ? new Selectable.WithSliceSelection.Raw(receiver, t1, t2)
    //          : new Selectable.WithElementSelection.Raw(receiver, t1);
    //   }
    //  ;
    fn parse_collection_subselection(
        &mut self,
        left: Expression,
    ) -> Result<Expression, ParseError> {
        self.expect(TokenType::LBracket)?;
        // parse term
        self.expect(TokenType::RBracket)?;
        Ok(Expression::CollectionSubSelection {
            receiver: Box::new(left),
            element: Box::new(self.parse_expression(Precedence::Min)?),
            upto: None,
        })
    }

    // Parse CQL data type
    fn parse_data_type(&mut self) -> Result<CqlType, ParseError> {
        // native data type?
        let maybe_native_type = self.parse_native_data_type();
        if maybe_native_type.is_ok() {
            return maybe_native_type;
        }
        // collection type?
        let maybe_collection_type = self.parse_collection_type();
        if maybe_collection_type.is_ok() {
            return maybe_collection_type;
        }
        // frozen type?
        let maybe_frozen = self.expect(TokenType::Keyword(Keyword::Frozen));
        if maybe_frozen.is_ok() {
            self.expect(TokenType::Lt)?;
            let inner_type = self.parse_data_type()?;
            self.expect(TokenType::Gt)?;
            return Ok(CqlType::Frozen(Box::new(inner_type)));
        }
        // User type name?
        let maybe_user_type_name = self.parse_user_type_name();
        if maybe_user_type_name.is_ok() {
            return Ok(CqlType::UserDefinedType(maybe_user_type_name?));
        }

        Err(ParseError::new())
    }

    // Parse CQL's native data type
    fn parse_native_data_type(&mut self) -> Result<CqlType, ParseError> {
        if let Some((_, next_token)) = self.peek() {
            let native_data_type = match &next_token.token_type {
                TokenType::Keyword(k) => match k {
                    Keyword::Ascii => Some(NativeDataType::Ascii),
                    Keyword::Bigint => Some(NativeDataType::BigInt),
                    Keyword::Blob => Some(NativeDataType::Blob),
                    Keyword::Boolean => Some(NativeDataType::Boolean),
                    Keyword::Counter => Some(NativeDataType::Counter),
                    Keyword::Decimal => Some(NativeDataType::Decimal),
                    Keyword::Double => Some(NativeDataType::Double),
                    Keyword::Duration => Some(NativeDataType::Duration),
                    Keyword::Float => Some(NativeDataType::Float),
                    Keyword::Inet => Some(NativeDataType::Inet),
                    Keyword::Int => Some(NativeDataType::Int),
                    Keyword::SmallInt => Some(NativeDataType::SmallInt),
                    Keyword::Text => Some(NativeDataType::Text),
                    Keyword::Timestamp => Some(NativeDataType::Timestamp),
                    Keyword::TinyInt => Some(NativeDataType::TinyInt),
                    Keyword::UUID => Some(NativeDataType::UUID),
                    Keyword::Varchar => Some(NativeDataType::Varchar),
                    Keyword::VarInt => Some(NativeDataType::VarInt),
                    Keyword::TimeUUID => Some(NativeDataType::TimeUUID),
                    Keyword::Date => Some(NativeDataType::Date),
                    Keyword::Time => Some(NativeDataType::Time),
                    _ => None,
                },
                _ => None,
            };
            native_data_type
                .map(|dt| {
                    self.advance();
                    CqlType::Native(dt)
                })
                .ok_or(ParseError::new())
        } else {
            Err(ParseError::new())
        }
    }

    fn parse_collection_type(&mut self) -> Result<CqlType, ParseError> {
        if self.expect(TokenType::Keyword(Keyword::Map)).is_ok() {
            self.expect(TokenType::Lt)?;
            let key_type = self.parse_data_type()?;
            self.expect(TokenType::Comma)?;
            let value_type = self.parse_data_type()?;
            self.expect(TokenType::Gt)?;
            Ok(CqlType::Collection(CollectionType::Map {
                key_type: Box::new(key_type),
                value_type: Box::new(value_type),
            }))
        } else if self.expect(TokenType::Keyword(Keyword::List)).is_ok() {
            self.expect(TokenType::Lt)?;
            let inner_type = self.parse_data_type()?;
            self.expect(TokenType::Gt)?;
            Ok(CqlType::Collection(CollectionType::List(Box::new(
                inner_type,
            ))))
        } else if self.expect(TokenType::Keyword(Keyword::Set)).is_ok() {
            self.expect(TokenType::Lt)?;
            let inner_type = self.parse_data_type()?;
            self.expect(TokenType::Gt)?;
            Ok(CqlType::Collection(CollectionType::Set(Box::new(
                inner_type,
            ))))
        } else if self.expect(TokenType::Keyword(Keyword::Tuple)).is_ok() {
            self.expect(TokenType::Lt)?;
            let mut inner_types = Vec::new();
            inner_types.push(self.parse_data_type()?);
            while self.expect(TokenType::Comma).is_ok() {
                inner_types.push(self.parse_data_type()?);
            }
            self.expect(TokenType::Gt)?;
            Ok(CqlType::Tuple(inner_types))
        } else {
            Err(ParseError::new())
        }
    }

    /// SELECT statement
    fn parse_select_statement(&mut self) -> CqlResult {
        self.expect(TokenType::Keyword(Keyword::Select))?;
        // TODO JSON
        // json is a valid column name. By consequence, we need to resolve the ambiguity for "json - json"
        // need to look ahead couples of tokens to determine...
        // probabliy need mark()-rewind() solution?

        // TODO DISTINCT
        let projection = self.parse_projection()?;

        self.expect(TokenType::Keyword(Keyword::From))?;
        let table_name = self.parse_qualified_name()?;

        // WHERE clause
        let selection = if self.expect(TokenType::Keyword(Keyword::Where)).is_ok() {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // GROUP BY clause
        if self.expect(TokenType::Keyword(Keyword::Group)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::By))?;
            // TODO
        }
        // ORDER BY clause
        if self.expect(TokenType::Keyword(Keyword::Order)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::By))?;
            // TODO
        }
        // PER PARTITION LIMIT clause
        let per_partition_limit = if self.expect(TokenType::Keyword(Keyword::Per)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::Partition))?;
            self.expect(TokenType::Keyword(Keyword::Limit))?;
            // TODO binding
            Some(Literal::Constant(self.parse_integer()?))
        } else {
            None
        };
        // LIMIT
        let limit = if self.expect(TokenType::Keyword(Keyword::Limit)).is_ok() {
            // TODO binding
            Some(Literal::Constant(self.parse_integer()?))
        } else {
            None
        };
        // ALLOW FILTERING
        let allow_filtering = if self.expect(TokenType::Keyword(Keyword::Allow)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::Filtering))?;
            true
        } else {
            false
        };

        Ok(CqlStatement::Select(SelectStatement {
            table_name,
            projection,
            selection,
            is_json: false,
            is_distinct: false,
            per_partition_limit,
            limit,
            allow_filtering,
        }))
    }

    fn parse_projection(&mut self) -> Result<Projection, ParseError> {
        // '*' - select all columns
        if self.expect(TokenType::Asterisk).is_ok() {
            return Ok(Projection::Wildcard);
        }

        let mut selectors = vec![];
        loop {
            let selector = self.parse_selector()?;
            // TODO maybe limit the size of selectors for safety (to not panic)
            selectors.push(selector);
            if self.expect(TokenType::Comma).is_err() {
                break;
            }
        }
        Ok(Projection::Selectors(selectors))
    }

    fn parse_selector(&mut self) -> Result<Selector, ParseError> {
        let selector = self.parse_expression(Precedence::Min)?;
        // check if selector has alias
        let alias = if self.expect(TokenType::Keyword(Keyword::As)).is_ok() {
            self.parse_ident()
        } else {
            None
        };
        Ok(Selector::new(selector, alias))
    }

    // TODO Negative NaN and Negative Infinity need to be TokenTypes as well
    fn parse_constant(&mut self) -> Result<Constant, ParseError> {
        if let Some((_, next)) = self.peek() {
            match &next.token_type {
                TokenType::Keyword(keyword) => match keyword {
                    // Literal constants
                    Keyword::NaN => Ok(Constant::NaN),
                    Keyword::Infinity => Ok(Constant::Infinity),
                    _ => Err(ParseError::new()),
                },
                // Literal constants
                TokenType::StringLiteral => self.parse_string_literal(),
                TokenType::Integer => self.parse_integer(),
                TokenType::Float => self.parse_float(),
                TokenType::Boolean => self.parse_boolean(),
                TokenType::Duration => self.parse_duration(),
                TokenType::UUID => self.parse_uuid(),
                TokenType::Hexnumber => self.parse_hexnumber(),
                _ => Err(ParseError::new()),
            }
        } else {
            Err(ParseError::new())
        }
    }

    // Unlike Apache Cassnadra's CQL parser, where clause is parsed as expression
    // relation_or_expression := relation
    //                        |  custom_index_expression
    //
    // # Custom index expression (CASSANDRA-10217)
    //
    // WHERE expr(lucene, '{lucene query here}')
    fn parse_where_clause(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression(Precedence::Min)
    }

    /// INSERT
    fn parse_insert_statement(&mut self) -> CqlResult {
        self.expect(TokenType::Keyword(Keyword::Insert))?;
        self.expect(TokenType::Keyword(Keyword::Into))?;
        let table = self.parse_qualified_name()?;

        // JSON insert
        let values = if self.expect(TokenType::Keyword(Keyword::Json)).is_ok() {
            let (json_literal, _) = self.expect(TokenType::StringLiteral)?;
            let json_string = json_literal.to_owned();
            // (DEFAULT (NULL | UNSET))?
            let has_default = self.expect(TokenType::Keyword(Keyword::Default)).is_ok();
            let behavior = if has_default {
                self.advance_if(|(_, t)| match t.token_type {
                    TokenType::Keyword(Keyword::Unset) | TokenType::Keyword(Keyword::Null) => true,
                    _ => false,
                })
                .map(|(_, t)| match t.token_type {
                    TokenType::Keyword(Keyword::Unset) => JsonBehavior::Unset,
                    TokenType::Keyword(Keyword::Null) => JsonBehavior::Null,
                    _ => unreachable!(),
                })
                .ok_or(ParseError::with_message(format!(
                    "UNSET or NULL was expected"
                )))?
            } else {
                JsonBehavior::Unset
            };
            InsertMethod::json(json_string, behavior)
        } else {
            // column list
            self.expect(TokenType::LParen)?;
            let mut columns = Vec::new();
            columns.push(self.parse_identifier()?);
            while self.expect(TokenType::Comma).is_ok() {
                columns.push(self.parse_identifier()?);
            }
            self.expect(TokenType::RParen)?;
            self.expect(TokenType::Keyword(Keyword::Values))?;
            // value list
            self.expect(TokenType::LParen)?;
            let mut values = Vec::new();
            values.push(self.parse_expression(Precedence::Min)?);
            while self.expect(TokenType::Comma).is_ok() {
                values.push(self.parse_expression(Precedence::Min)?);
            }
            self.expect(TokenType::RParen)?;
            InsertMethod::normal(columns, values)
        };
        // IF NOT EXISTS
        let if_not_exists = self.parse_if_not_exists()?;
        // USING clause
        let (timestamp, time_to_live) = self.parse_using_clause()?;

        Ok(CqlStatement::Insert(InsertStatement {
            table,
            values,
            if_not_exists,
            timestamp,
            time_to_live,
        }))
    }

    // UPDATE statement
    fn parse_update_statement(&mut self) -> CqlResult {
        self.expect(TokenType::Keyword(Keyword::Update))?;
        let table = self.parse_qualified_name()?;
        let (timestamp, time_to_live) = self.parse_using_clause()?;
        self.expect(TokenType::Keyword(Keyword::Set))?;
        let mut assignments = Vec::new();
        loop {
            assignments.push(self.parse_expression(Precedence::Min)?);
            if self.expect(TokenType::Comma).is_err() {
                break;
            }
        }
        self.expect(TokenType::Keyword(Keyword::Where))?;
        let selection = self.parse_where_clause()?;
        let mut if_exists = false;
        // IF
        if self.expect(TokenType::Keyword(Keyword::If)).is_ok() {
            // EXISTS?
            if self.expect(TokenType::Keyword(Keyword::Exists)).is_ok() {
                if_exists = true;
            } else {
                // TODO IF condition
            }
        }
        Ok(CqlStatement::Update(UpdateStatement {
            table,
            if_exists,
            assignments,
            selection,
            timestamp,
            time_to_live,
        }))
    }

    /// IF NOT EXISTS
    fn parse_if_not_exists(&mut self) -> Result<bool, ParseError> {
        if self.expect(TokenType::Keyword(Keyword::If)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::Not))?;
            self.expect(TokenType::Keyword(Keyword::Exists))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Returns (timestamp, time_to_live) pair if USING clause is present
    fn parse_using_clause(&mut self) -> Result<(Option<Literal>, Option<Literal>), ParseError> {
        let has_using_clause = self.expect(TokenType::Keyword(Keyword::Using)).is_ok();
        if has_using_clause {
            let mut timestamp = None;
            let mut ttl = None;
            loop {
                if self.expect(TokenType::Keyword(Keyword::Timestamp)).is_ok() {
                    match self.parse_integer() {
                        Ok(v @ Constant::Integer(_)) => timestamp.replace(Literal::Constant(v)),
                        _ => {
                            return Err(ParseError::with_message(
                                "Integer value is expected in timestamp".to_owned(),
                            ))
                        }
                    };
                    // TODO binding value
                } else if self.expect(TokenType::Keyword(Keyword::Ttl)).is_ok() {
                    match self.parse_integer() {
                        Ok(v @ Constant::Integer(_)) => ttl.replace(Literal::Constant(v)),
                        _ => {
                            return Err(ParseError::with_message(
                                "Integer value is expected in ttl".to_owned(),
                            ))
                        }
                    };
                    // TODO binding value
                } else {
                    return Err(ParseError::with_message(format!(
                        "Only TIMESTAMP or TTL is expected in USING clause"
                    )));
                }

                if self.expect(TokenType::Keyword(Keyword::And)).is_err() {
                    break;
                }
            }
            Ok((timestamp, ttl))
        } else {
            Ok((None, None))
        }
    }

    // Entry point for all the CREATE statements
    fn create_statement(&mut self) -> CqlResult {
        self.expect(TokenType::Keyword(Keyword::Create))?;

        let (_, next_keyword_token) = self
            .advance_if(|(_, t)| match t.token_type {
                TokenType::Keyword(Keyword::Keyspace)
                | TokenType::Keyword(Keyword::Table)
                | TokenType::Keyword(Keyword::Custom)
                | TokenType::Keyword(Keyword::Index)
                | TokenType::Keyword(Keyword::Materialized)
                | TokenType::Keyword(Keyword::Type) => true,
                _ => false,
            })
            .ok_or(ParseError::with_message(
                "Unexpected token after CREATE".to_owned(),
            ))?;
        match next_keyword_token.token_type {
            TokenType::Keyword(Keyword::Keyspace) => self.parse_create_keyspace_statement(),
            TokenType::Keyword(Keyword::Table) => self.parse_create_table_statement(),
            TokenType::Keyword(Keyword::Index) => self.parse_create_index_statement(false),
            TokenType::Keyword(Keyword::Custom) => {
                self.expect(TokenType::Keyword(Keyword::Index))?;
                self.parse_create_index_statement(true)
            }
            TokenType::Keyword(Keyword::Materialized) => {
                self.expect(TokenType::Keyword(Keyword::View))?;
                self.parse_create_materialized_view_statement()
            }
            TokenType::Keyword(Keyword::Type) => self.parse_create_type_statement(),
            _ => Err(ParseError::new()),
        }
    }

    /// CREATE KEYSPACE
    fn parse_create_keyspace_statement(&mut self) -> CqlResult {
        let if_not_exists = self.parse_if_not_exists()?;
        let keyspace_name = self.parse_ident().ok_or(ParseError::new())?;

        // parse properties
        self.expect(TokenType::Keyword(Keyword::With))?;
        let attributes = self.parse_properties()?;

        Ok(CqlStatement::CreateKeyspace(CreateKeyspaceStatement {
            keyspace_name,
            attributes,
            if_not_exists,
        }))
    }

    /// CREATE TABLE
    fn parse_create_table_statement(&mut self) -> CqlResult {
        let if_not_exists = self.parse_if_not_exists()?;
        let table_name = self.parse_qualified_name()?;
        self.expect(TokenType::LParen)?;
        let mut column_definitions = Vec::new();
        let mut partition_keys = Vec::new();
        let mut clustering_columns = Vec::new();
        let mut static_columns = Vec::new();
        loop {
            if let Some((s, token)) = self.peek() {
                match token.token_type {
                    // PRIMARY KEY (...) definition
                    TokenType::Keyword(Keyword::Primary) => {
                        let (pk, clustering) = self.parse_primary_key_clause()?;
                        partition_keys.push(pk);
                        clustering_columns.extend(clustering);
                    }
                    TokenType::Identifier | TokenType::QuotedName | TokenType::Keyword(_) => {
                        let (column, data_type, is_static, is_pk) =
                            self.parse_column_definition()?;
                        column_definitions.push((column.clone(), data_type));
                        if is_static {
                            static_columns.push(column.clone());
                        }
                        if is_pk {
                            partition_keys.push(vec![column]);
                        }
                    }
                    _ => {
                        return Err(ParseError::with_message(format!(
                            "unexpected token: {}",
                            *s
                        )));
                    }
                }
            }
            if self.expect(TokenType::Comma).is_err() {
                break;
            }
        }
        self.expect(TokenType::RParen)?;
        // Table properties
        let mut table_properties = Vec::new();
        let mut compact_storage = false;
        let mut clustering_order = Vec::new();
        if self.expect(TokenType::Keyword(Keyword::With)).is_ok() {
            loop {
                // Compact Storage
                compact_storage = if !compact_storage
                    && self.expect(TokenType::Keyword(Keyword::Compact)).is_ok()
                {
                    self.expect(TokenType::Keyword(Keyword::Storage))?;
                    true
                } else {
                    false
                };
                // Clustering Order By
                clustering_order.extend(self.parse_clustering_order_by()?);
                // Table property
                if let Ok(prop) = self.parse_property() {
                    table_properties.push(prop);
                }
                if self.expect(TokenType::Keyword(Keyword::And)).is_err() {
                    break;
                }
            }
        }

        Ok(CqlStatement::CreateTable(CreateTableStatement {
            if_not_exists,
            name: table_name,
            column_definitions,
            partition_keys,
            clustering_columns,
            static_columns,
            compact_storage,
            clustering_order,
            table_properties,
        }))
    }

    /// returns (partition keys, clustering columns) pair
    fn parse_primary_key_clause(&mut self) -> Result<(Vec<String>, Vec<String>), ParseError> {
        self.expect(TokenType::Keyword(Keyword::Primary))?;
        self.expect(TokenType::Keyword(Keyword::Key))?;
        self.expect(TokenType::LParen)?;

        let mut partition_keys = Vec::new();
        if self.expect(TokenType::LParen).is_ok() {
            // multiple partition keys
            partition_keys.push(self.parse_ident().ok_or(ParseError::with_message(format!(
                "Identifier is expected in partition key definition"
            )))?);
            while self.expect(TokenType::Comma).is_ok() {
                partition_keys.push(self.parse_ident().ok_or(ParseError::with_message(
                    format!("Identifier is expected in partition key definition"),
                ))?);
            }
            self.expect(TokenType::RParen)?;
        } else {
            partition_keys.push(self.parse_ident().ok_or(ParseError::with_message(format!(
                "Identifier is expected in partition key definition"
            )))?);
        }
        // Clustering columns
        let mut clustering_columns = Vec::new();
        while self.expect(TokenType::Comma).is_ok() {
            clustering_columns.push(self.parse_ident().ok_or(ParseError::with_message(
                format!("Identifier is expected in clustring column definition"),
            ))?);
        }
        self.expect(TokenType::RParen)?;

        Ok((partition_keys, clustering_columns))
    }

    // returns (column name, data type, static?, primary key?) pair
    fn parse_column_definition(&mut self) -> Result<(String, CqlType, bool, bool), ParseError> {
        let ident = self
            .parse_ident()
            .ok_or(ParseError::with_message(format!("identifier expected")))?;
        let cql_type = self.parse_data_type()?;

        // is STATIC column definition?
        let is_static = self.expect(TokenType::Keyword(Keyword::Static)).is_ok();
        // is PRIMARY KEY?
        let is_primary_key = if self.expect(TokenType::Keyword(Keyword::Primary)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::Key))?;
            true
        } else {
            false
        };

        Ok((ident, cql_type, is_static, is_primary_key))
    }

    fn parse_clustering_order_by(&mut self) -> Result<Vec<(String, bool)>, ParseError> {
        let mut clustering_orders = Vec::new();
        if self.expect(TokenType::Keyword(Keyword::Clustering)).is_ok() {
            self.expect(TokenType::Keyword(Keyword::Order))?;
            self.expect(TokenType::Keyword(Keyword::By))?;
            self.expect(TokenType::LParen)?;
            loop {
                let ident = self
                    .parse_ident()
                    .ok_or(ParseError::with_message(format!("Identifier expected")))?;
                let ascending = if self.expect(TokenType::Keyword(Keyword::Asc)).is_ok() {
                    true
                } else {
                    self.expect(TokenType::Keyword(Keyword::Desc))?;
                    false
                };
                clustering_orders.push((ident, ascending));
                if self.expect(TokenType::Comma).is_err() {
                    break;
                }
            }
            self.expect(TokenType::RParen)?;
        }
        Ok(clustering_orders)
    }

    fn parse_properties(&mut self) -> Result<Vec<Property>, ParseError> {
        let mut properties = Vec::new();
        properties.push(self.parse_property()?);
        while self.expect(TokenType::Keyword(Keyword::And)).is_ok() {
            properties.push(self.parse_property()?);
        }
        Ok(properties)
    }

    fn parse_property(&mut self) -> Result<Property, ParseError> {
        let key = self.parse_ident().ok_or(ParseError::new())?;
        self.expect(TokenType::Equal)?;
        // Value for the property is either:
        // - constant
        // - unreserved keywords (though I'm not sure why unreserved keywords are allowed)
        // - map literal
        let value = self
            .parse_constant()
            .map(Literal::Constant)
            .or_else(|_| {
                if let Some((s, _)) = self.advance_if(|(_, t)| match &t.token_type {
                    TokenType::Keyword(k) => k.is_unreserved_keyword(),
                    _ => false,
                }) {
                    Ok(Literal::Constant(Constant::StringLiteral(
                        s.to_ascii_lowercase(),
                    )))
                } else {
                    Err(ParseError::new())
                }
            })
            .or_else(|_| self.parse_map_literal())?;
        Ok(Property::new(key, value))
    }

    fn parse_qualified_name(&mut self) -> Result<QualifiedName, ParseError> {
        self.parse_ident()
            .map(|name| {
                let second = if self.expect(TokenType::Dot).is_ok() {
                    self.parse_ident()
                } else {
                    None
                };
                (name, second)
            })
            .and_then(|(first_name, second_name)| {
                if second_name.is_some() {
                    Some(QualifiedName::new(Some(first_name), second_name.unwrap()))
                } else {
                    Some(QualifiedName::new(None, first_name))
                }
            })
            .ok_or(ParseError::with_message(
                "Invalid qualified name".to_owned(),
            ))
    }

    // Similar to `parse_qualified_name`, however,
    // `TOKEN` and `COUNT` keywords are allowed for function name.
    fn parse_function_name(&mut self) -> Result<QualifiedName, ParseError> {
        self.parse_ident()
            .map(|name| {
                let second = if self.expect(TokenType::Dot).is_ok() {
                    // TODO TOKEN and COUNT are allowed
                    self.parse_ident()
                } else {
                    None
                };
                (name, second)
            })
            .and_then(|(first_name, second_name)| {
                if second_name.is_some() {
                    Some(QualifiedName::new(Some(first_name), second_name.unwrap()))
                } else {
                    Some(QualifiedName::new(None, first_name))
                }
            })
            .ok_or(ParseError::with_message(
                "Invalid qualified name".to_owned(),
            ))
    }

    // Similar to `parse_qualified_name`, however,
    // only basic unreserved keyword + `KEY` keyword can be used.
    fn parse_user_type_name(&mut self) -> Result<QualifiedName, ParseError> {
        // TODO first part (keyspace name can be just ident)
        self.parse_non_type_ident()
            .map(|name| {
                let second = if self.expect(TokenType::Dot).is_ok() {
                    self.parse_non_type_ident()
                } else {
                    None
                };
                (name, second)
            })
            .and_then(|(first_name, second_name)| {
                if second_name.is_some() {
                    Some(QualifiedName::new(Some(first_name), second_name.unwrap()))
                } else {
                    Some(QualifiedName::new(None, first_name))
                }
            })
            .ok_or(ParseError::with_message(
                "Invalid qualified name".to_owned(),
            ))
    }

    fn parse_non_type_ident(&mut self) -> Option<String> {
        self.parse_ident_and_keywords(|k| k.is_basic_unreserved_keyword() || *k == Keyword::Key)
    }

    fn parse_ident_and_keywords<F>(&mut self, keyword_filter: F) -> Option<String>
    where
        F: Fn(&Keyword) -> bool,
    {
        if let Some((s, token)) = self.advance_if(|(_, t)| match &t.token_type {
            TokenType::Identifier | TokenType::QuotedName => true,
            TokenType::Keyword(k) => keyword_filter(k),
            _ => false,
        }) {
            match token.token_type {
                // If IDENT, return lowercase version of the name
                TokenType::Identifier => Some(String::from(s).to_ascii_lowercase()),
                TokenType::QuotedName => {
                    // remove surounding `"`
                    let inner = s[1..s.len() - 1].to_owned();
                    // replace `""` with single `"`
                    Some(inner.replace("\"\"", "\""))
                }
                TokenType::Keyword(_) => Some(String::from(s).to_ascii_lowercase()),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    // createIndexStatement returns [CreateIndexStatement.Raw stmt]
    // @init {
    //     IndexAttributes props = new IndexAttributes();
    //     boolean ifNotExists = false;
    //     QualifiedName name = new QualifiedName();
    //     List<IndexTarget.Raw> targets = new ArrayList<>();
    // }
    // : K_CREATE (K_CUSTOM { props.isCustom = true; })? K_INDEX (K_IF K_NOT K_EXISTS { ifNotExists = true; } )?
    //     (idxName[name])? K_ON cf=columnFamilyName '(' (indexIdent[targets] (',' indexIdent[targets])*)? ')'
    //     (K_USING cls=STRING_LITERAL { props.customClass = $cls.text; })?
    //     (K_WITH properties[props])?
    //   { $stmt = new CreateIndexStatement.Raw(cf, name, targets, props, ifNotExists); }
    // ;
    fn parse_create_index_statement(
        &mut self,
        is_custom: bool,
    ) -> Result<CqlStatement, ParseError> {
        let if_not_exists = self.parse_if_not_exists()?;
        // index name is optional
        let index_name = self.parse_ident();
        self.expect(TokenType::Keyword(Keyword::On))?;
        let table_name = self.parse_qualified_name()?;
        self.expect(TokenType::LParen)?;
        let mut index_targets = Vec::new();
        loop {
            index_targets.push(self.parse_index_target()?);
            if self.expect(TokenType::Comma).is_err() {
                break;
            }
        }
        self.expect(TokenType::RParen)?;
        Ok(CqlStatement::CreateIndex(CreateIndexStatement {
            index_name,
            table_name,
            if_not_exists,
            is_custom,
            index_targets,
        }))
    }

    /// Index target
    /// One of the following:
    /// - ident
    /// - VALUES(ident)
    /// - KEYS(ident)
    /// - ENTRIES(ident)
    /// - FULL(ident)
    fn parse_index_target(&mut self) -> Result<(String, IndexType), ParseError> {
        if let Some((_, t)) = self.peek() {
            match t.token_type {
                TokenType::Keyword(Keyword::Values) => {
                    if self.expect(TokenType::LParen).is_ok() {
                        // VALUES(ident) pattern
                        let ident = self
                            .parse_ident()
                            .ok_or(ParseError::with_message(format!("identifier expected")))?;
                        self.expect(TokenType::RParen)?;
                        Ok((ident, IndexType::Values))
                    } else {
                        // VALUES as simple index target
                        Ok((String::from("values"), IndexType::Simple))
                    }
                }
                TokenType::Keyword(Keyword::Keys) => {
                    if self.expect(TokenType::LParen).is_ok() {
                        // VALUES(ident) pattern
                        let ident = self
                            .parse_ident()
                            .ok_or(ParseError::with_message(format!("identifier expected")))?;
                        self.expect(TokenType::RParen)?;
                        Ok((ident, IndexType::Keys))
                    } else {
                        // VALUES as simple index target
                        Ok((String::from("keys"), IndexType::Simple))
                    }
                }
                TokenType::Keyword(Keyword::Entries) => {
                    if self.expect(TokenType::LParen).is_ok() {
                        // VALUES(ident) pattern
                        let ident = self
                            .parse_ident()
                            .ok_or(ParseError::with_message(format!("identifier expected")))?;
                        self.expect(TokenType::RParen)?;
                        Ok((ident, IndexType::KeysAndValues))
                    } else {
                        // VALUES as simple index target
                        Ok((String::from("entries"), IndexType::Simple))
                    }
                }
                TokenType::Keyword(Keyword::Full) => {
                    if self.expect(TokenType::LParen).is_ok() {
                        // VALUES(ident) pattern
                        let ident = self
                            .parse_ident()
                            .ok_or(ParseError::with_message(format!("identifier expected")))?;
                        self.expect(TokenType::RParen)?;
                        Ok((ident, IndexType::Full))
                    } else {
                        // VALUES as simple index target
                        Ok((String::from("full"), IndexType::Simple))
                    }
                }
                TokenType::Identifier | TokenType::QuotedName | TokenType::Keyword(_) => {
                    let ident = self
                        .parse_ident()
                        .ok_or(ParseError::with_message(format!("identifier expected")))?;
                    Ok((ident, IndexType::Simple))
                }
                _ => Err(ParseError::new()),
            }
        } else {
            Err(ParseError::new())
        }
    }

    // CREATE MATERIALIZED VIEW statement
    fn parse_create_materialized_view_statement(&mut self) -> Result<CqlStatement, ParseError> {
        let if_not_exists = self.parse_if_not_exists()?;
        let name = self.parse_qualified_name()?;
        self.expect(TokenType::Keyword(Keyword::As))?;
        self.expect(TokenType::Keyword(Keyword::Select))?;
        let projection = self.parse_projection()?;
        self.expect(TokenType::Keyword(Keyword::From))?;
        let base_table = self.parse_qualified_name()?;
        // WHERE clause
        let selection = if self.expect(TokenType::Keyword(Keyword::Where)).is_ok() {
            Some(self.parse_where_clause()?)
        } else {
            None
        };
        // PRIMARY KEY (...) definition
        let (partition_keys, clustering_columns) = self.parse_primary_key_clause()?;
        // Table properties
        let mut view_properties = Vec::new();
        let mut compact_storage = false;
        let mut clustering_order = Vec::new();
        if self.expect(TokenType::Keyword(Keyword::With)).is_ok() {
            loop {
                // Compact Storage
                compact_storage = if !compact_storage
                    && self.expect(TokenType::Keyword(Keyword::Compact)).is_ok()
                {
                    self.expect(TokenType::Keyword(Keyword::Storage))?;
                    true
                } else {
                    false
                };
                // Clustering Order By
                clustering_order.extend(self.parse_clustering_order_by()?);
                // Table property
                if let Ok(prop) = self.parse_property() {
                    view_properties.push(prop);
                }
                if self.expect(TokenType::Keyword(Keyword::And)).is_err() {
                    break;
                }
            }
        }
        Ok(CqlStatement::CreateMaterializedView(
            CreateMaterializedViewStatement {
                name,
                base_table,
                if_not_exists,
                projection,
                selection,
                partition_keys,
                clustering_columns,
                compact_storage,
                clustering_order,
                view_properties,
            },
        ))
    }

    // CREATE TYPE statement
    fn parse_create_type_statement(&mut self) -> Result<CqlStatement, ParseError> {
        let if_not_exists = self.parse_if_not_exists()?;
        let name = self.parse_user_type_name()?;
        self.expect(TokenType::LParen)?;
        let mut field_definitions = Vec::new();
        loop {
            if let Some((s, token)) = self.peek() {
                match token.token_type {
                    TokenType::Identifier | TokenType::QuotedName | TokenType::Keyword(_) => {
                        let field = self
                            .parse_ident()
                            .ok_or(ParseError::with_message(format!("identifier expected")))?;
                        let cql_type = self.parse_data_type()?;
                        field_definitions.push((field, cql_type));
                    }
                    _ => {
                        return Err(ParseError::with_message(format!(
                            "unexpected token: {}",
                            *s
                        )));
                    }
                }
            }
            if self.expect(TokenType::Comma).is_err() {
                break;
            }
        }
        self.expect(TokenType::RParen)?;
        Ok(CqlStatement::CreateType(CreateTypeStatement {
            name,
            if_not_exists,
            field_definitions,
        }))
    }

    /// Parse identifier
    ///
    /// An identifier is one of the following:
    /// - IDENT token
    /// - QUOTED_NAME token
    /// - Unreserved keyword
    ///
    /// When IDENT or Unreserved keyword, string is converted into lowercase.
    /// When QUOTED_NAME, surrounding double quote (`"`) is removed, and escaped
    /// double quote (`""`) is converted into single double quote.
    fn parse_ident(&mut self) -> Option<String> {
        self.parse_ident_and_keywords(|k| k.is_unreserved_keyword())
    }
}

#[test]
fn test_relation() {
    let mut _p = Parser::new("col1 = 'a'");

    // assert!(p.relation().is_ok());
}

#[test]
fn test_parse_qualified_name() {
    let test_cases = [
        ("test1", Ok(QualifiedName::new(None, String::from("test1")))),
        (
            "ks.test1",
            Ok(QualifiedName::new(
                Some("ks".to_string()),
                String::from("test1"),
            )),
        ),
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_qualified_name(), test.1);
    }
}

#[test]
fn test_parse_property() {
    let test_cases = [
        (
            "prop = 'value'",
            Ok(Property::new(
                "prop".to_owned(),
                Literal::Constant(Constant::StringLiteral("value".to_owned())),
            )),
        ),
        (
            "replication = {'class': 'NetworkTopologyStrategy', 'dc1': 3}",
            Ok(Property::new(
                "replication".to_owned(),
                Literal::Map(vec![
                    (
                        Expression::Value(Literal::Constant(Constant::StringLiteral(
                            "class".to_owned(),
                        ))),
                        Expression::Value(Literal::Constant(Constant::StringLiteral(
                            "NetworkTopologyStrategy".to_owned(),
                        ))),
                    ),
                    (
                        Expression::Value(Literal::Constant(Constant::StringLiteral(
                            "dc1".to_owned(),
                        ))),
                        Expression::Value(Literal::Constant(Constant::Integer(3))),
                    ),
                ]),
            )),
        ),
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_property(), test.1);
    }
}

#[test]
fn test_parse_map_literal() {
    let test_cases = [
        ("{}", Ok(Literal::Map(Vec::new()))),
        (
            "{'key': 1}",
            Ok(Literal::Map(vec![(
                Expression::Value(Literal::Constant(Constant::StringLiteral(String::from(
                    "key",
                )))),
                Expression::Value(Literal::Constant(Constant::Integer(1))),
            )])),
        ),
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_map_literal(), test.1);
    }
}

#[test]
fn test_parse_tuple() {
    let test_cases = [
        ("()", Ok(Expression::Value(Literal::Tuple(Vec::new())))),
        (
            "(1, 2, 3)",
            Ok(Expression::Value(Literal::Tuple(vec![
                Expression::Value(Literal::Constant(Constant::Integer(1))),
                Expression::Value(Literal::Constant(Constant::Integer(2))),
                Expression::Value(Literal::Constant(Constant::Integer(3))),
            ]))),
        ),
        // tuple of idents
        (
            "(a, B, \"C\")",
            Ok(Expression::Value(Literal::Tuple(vec![
                Expression::Identifier(String::from("a")),
                Expression::Identifier(String::from("b")),
                Expression::Identifier(String::from("C")),
            ]))),
        ),
        (
            "(1, (2, 3), 4)",
            Ok(Expression::Value(Literal::Tuple(vec![
                Expression::Value(Literal::Constant(Constant::Integer(1))),
                Expression::Value(Literal::Tuple(vec![
                    Expression::Value(Literal::Constant(Constant::Integer(2))),
                    Expression::Value(Literal::Constant(Constant::Integer(3))),
                ])),
                Expression::Value(Literal::Constant(Constant::Integer(4))),
            ]))),
        ),
        (
            "((1, 2), (3, 4), (5, 6))",
            Ok(Expression::Value(Literal::Tuple(vec![
                Expression::Value(Literal::Tuple(vec![
                    Expression::Value(Literal::Constant(Constant::Integer(1))),
                    Expression::Value(Literal::Constant(Constant::Integer(2))),
                ])),
                Expression::Value(Literal::Tuple(vec![
                    Expression::Value(Literal::Constant(Constant::Integer(3))),
                    Expression::Value(Literal::Constant(Constant::Integer(4))),
                ])),
                Expression::Value(Literal::Tuple(vec![
                    Expression::Value(Literal::Constant(Constant::Integer(5))),
                    Expression::Value(Literal::Constant(Constant::Integer(6))),
                ])),
            ]))),
        ),
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_expression(Precedence::Min), test.1);
    }
}

#[test]
fn test_parse_cast() {
    let test_cases = [(
        "cast(col as int)",
        Ok(Expression::TypeCast(
            CqlType::Native(NativeDataType::Int),
            Box::new(Expression::Identifier(String::from("col"))),
        )),
    )];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_cast(), test.1);
    }
}

#[test]
fn test_parse_expression() {
    let test_cases = [
        // unary operations
        (
            "-col",
            Ok(Expression::UnaryOp(UnaryOp::new(
                Operator::Minus,
                Box::new(Expression::Identifier("col".to_owned())),
            ))),
        ),
        (
            "-1000",
            Ok(Expression::UnaryOp(UnaryOp::new(
                Operator::Minus,
                Box::new(Expression::Value(Literal::Constant(Constant::Integer(
                    1000,
                )))),
            ))),
        ),
        // binary operations
        (
            "col + 1",
            Ok(Expression::BinaryOp(BinaryOp::new(
                Box::new(Expression::Identifier("col".to_owned())),
                Operator::Plus,
                Box::new(Expression::Value(Literal::Constant(Constant::Integer(1)))),
            ))),
        ),
        (
            "col = 'test'",
            Ok(Expression::BinaryOp(BinaryOp::new(
                Box::new(Expression::Identifier("col".to_owned())),
                Operator::Equal,
                Box::new(Expression::Value(Literal::Constant(
                    Constant::StringLiteral(String::from("test")),
                ))),
            ))),
        ),
        (
            "a = 1 AND b = 2",
            Ok(Expression::BinaryOp(BinaryOp::new(
                Box::new(Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Operator::Equal,
                    Box::new(Expression::Value(Literal::Constant(Constant::Integer(1)))),
                ))),
                Operator::And,
                Box::new(Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::Identifier("b".to_owned())),
                    Operator::Equal,
                    Box::new(Expression::Value(Literal::Constant(Constant::Integer(2)))),
                ))),
            ))),
        ),
        // complex
        (
            "((cast(storage_port as int) + 1000) * 4) - cast(native_transport_port as int)",
            Ok(Expression::BinaryOp(BinaryOp::new(
                Box::new(Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::BinaryOp(BinaryOp::new(
                        Box::new(Expression::TypeCast(
                            CqlType::Native(NativeDataType::Int),
                            Box::new(Expression::Identifier("storage_port".to_owned())),
                        )),
                        Operator::Plus,
                        Box::new(Expression::Value(Literal::Constant(Constant::Integer(
                            1000,
                        )))),
                    ))),
                    Operator::Multiply,
                    Box::new(Expression::Value(Literal::Constant(Constant::Integer(4)))),
                ))),
                Operator::Minus,
                Box::new(Expression::TypeCast(
                    CqlType::Native(NativeDataType::Int),
                    Box::new(Expression::Identifier("native_transport_port".to_owned())),
                )),
            ))),
        ),
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_expression(Precedence::Min), test.1);
    }
}

#[test]
fn test_parse_projection() {
    let test_cases = [
        ("*", Ok(Projection::Wildcard)),
        (
            "1 + 2 * (3 - 4) as calc",
            Ok(Projection::Selectors(vec![Selector::new(
                Expression::BinaryOp(BinaryOp::new(
                    Box::new(Expression::Value(Literal::Constant(Constant::Integer(1)))),
                    Operator::Plus,
                    Box::new(Expression::BinaryOp(BinaryOp::new(
                        Box::new(Expression::Value(Literal::Constant(Constant::Integer(2)))),
                        Operator::Multiply,
                        Box::new(Expression::BinaryOp(BinaryOp::new(
                            Box::new(Expression::Value(Literal::Constant(Constant::Integer(3)))),
                            Operator::Minus,
                            Box::new(Expression::Value(Literal::Constant(Constant::Integer(4)))),
                        ))),
                    ))),
                )),
                Some("calc".to_owned()),
            )])),
        ),
        (
            "col1, col2 as col_a",
            Ok(Projection::Selectors(vec![
                Selector::new(Expression::Identifier("col1".to_owned()), None),
                Selector::new(
                    Expression::Identifier("col2".to_owned()),
                    Some("col_a".to_owned()),
                ),
            ])),
        ),
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_projection(), test.1);
    }
}

#[test]
fn test_parse_ident() {
    let test_cases = [
        ("IDENT", Some(String::from("ident"))), // identifier
        ("\"\"\"\"\"Key\"\"\"", Some(String::from("\"\"Key\""))), // quoted name
        ("Inet", Some(String::from("inet"))),   // unreserved keyword
    ];
    for test in &test_cases {
        let mut p = Parser::new(test.0);
        assert_eq!(p.parse_ident(), test.1);
    }
}
