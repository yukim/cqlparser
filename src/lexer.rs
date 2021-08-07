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

use std::iter::Iterator;
use std::iter::Peekable;
use std::str::Chars;

use crate::literal::*;

/// CQL Tokens
#[derive(Debug, Eq, PartialEq)]
pub struct Token {
    /// Type of this token, as defined in `TokenType`.
    pub token_type: TokenType,
    /// Position in bytes in original CQL from the beginning.
    pub offset: usize,
    /// Length of token in bytes.
    pub length: usize,
}

impl Token {
    /// Create new Token with given type, offset and length.
    ///
    /// `offset` is a position in bytes in original CQL from the beginning.
    /// `length` is a length of token in bytes.
    pub fn new(token_type: TokenType, offset: usize, length: usize) -> Self {
        Token {
            token_type,
            offset,
            length,
        }
    }

    /// return true if this token's type is given `token_type`
    pub fn is_type(&self, token_type: TokenType) -> bool {
        self.token_type == token_type
    }
}

/*
impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, offset:{}, length:{})", self.token_type, self.offset, self.length)
    }
}
*/

/// Token types
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenType {
    /// CQL Keywords
    /// (https://cassandra.apache.org/doc/latest/cql/appendices.html#appendix-a-cql-keywords)
    Keyword(Keyword),

    /// A string constant is an arbitrary sequence of characters characters enclosed by single-quote(').
    /// One can include a single-quote in a string by repeating it, e.g. 'It''s raining today'.
    /// Those are not to be confused with quoted identifiers that use double-quotes.
    ///
    /// CQL also supports PostgreSQL style string literal.
    /// Content inside '$$'...'$$' is considered one string literal.
    StringLiteral,

    /// Identifiers to identify tables, columns and other objects.
    /// [a-zA-Z][a-zA-Z0-9_]*
    Identifier,

    /// Quoted identifier
    ///
    /// Inside a quoted identifier, the double-quote character can be repeated to escape it,
    /// so "foo "" bar" is a valid identifier.
    QuotedName,

    /// Integer
    /// [0-9]+
    ///
    /// Negative integer is evaluated with '-' unary operator.
    Integer,
    /// A float constant is defined by [0-9]+('.'[0-9]*)?([eE][+-]?[0-9]+)?.
    /// On top of that, NaN and Infinity are also float constants.
    Float,
    /// A boolean constant is either true or false up to case-insensitivity (i.e. True is a valid boolean constant).
    Boolean,
    /// Duration in ISO 8601 format
    Duration,
    /// A blob constant is an hexadecimal number defined by 0[xX](hex)+ where hex is an hexadecimal character, e.g. [0-9a-fA-F].
    Hexnumber,
    /// A UUID constant is defined by hex{8}-hex{4}-hex{4}-hex{4}-hex{12} where
    /// hex is an hexadecimal character, e.g. [0-9a-fA-F] and {4} is the number of such characters.
    UUID,
    /// Whitespace
    /// (' ' | '\t' | '\n' | '\r')+
    Whitespace,
    /// A comment in CQL is a line beginning by either double dashes (--) or double slash (//).
    /// Multi-line comments are also supported through enclosure within /* and */ (but nesting is not supported).
    ///
    /// When internal `bool` is `true`, this indicates multi-line comments.
    Comment(bool),
    /// '='
    Equal,
    /// '!='
    NotEqual,
    /// '>'
    Gt,
    /// '>='
    Gte,
    /// '<'
    Lt,
    /// '<='
    Lte,
    /// '+'
    Plus,
    /// '-'
    Minus,
    /// '*'
    Asterisk,
    /// '/'
    Slash,
    /// '%'
    Percent,
    /// '.'
    Dot,
    /// '..'
    Range,
    /// ';'
    SemiColon,
    /// ':'
    Colon,
    /// ','
    Comma,
    /// Left parenthesis `(`
    LParen,
    /// Right parenthesis `)`
    RParen,
    /// Left bracket `[`
    LBracket,
    /// Right bracket `]`
    RBracket,
    /// Ampersand '&'
    Ampersand,
    /// Question mark '?'
    Qmark,
    /// Left brace `{`
    LBrace,
    /// Right brace `}`
    RBrace,
    /// EOF
    EOF,
    /// Error token
    Error,
}

/// CQL keywords
///
/// ## Unreserved keywords
///
/// unreserved_function_keyword returns [String str]
/// : u=basic_unreserved_keyword { $str = u; }
/// | t=native_type              { $str = t.toString(); }
/// | k=(K_TTL | K_COUNT | K_WRITETIME | K_KEY | K_CAST | K_JSON | K_DISTINCT) { $str = $k.text; }
/// ;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Keyword {
    Select,
    From,
    As,
    Where,
    And,
    Key,
    Keys,
    Entries,
    Full,
    Insert,
    Update,
    With,
    Limit,
    Per,
    Partition,
    Using,
    Use,
    Distinct,
    Count,
    Set,
    Begin,
    Unlogged,
    Batch,
    Apply,
    Truncate,
    Delete,
    In,
    Create,
    Schema,
    Keyspace,
    Keyspaces,
    Table,
    Tables,
    Materialized,
    View,
    Index,
    Custom,
    On,
    To,
    Drop,
    Primary,
    Into,
    Values,
    Timestamp,
    Ttl,
    Cast,
    Alter,
    Rename,
    Add,
    Type,
    Types,
    Compact,
    Storage,
    Order,
    By,
    Asc,
    Desc,
    Allow,
    Filtering,
    If,
    Is,
    Contains,
    Group,
    Cluster,
    Internals,
    Only,
    Grant,
    All,
    Permission,
    Permissions,
    Of,
    Revoke,
    Modify,
    Authorize,
    Describe,
    Execute,
    NoRecursive,
    MBean,
    MBeans,
    User,
    Users,
    Role,
    Roles,
    Superuser,
    NoSuperuser,
    Password,
    Login,
    NoLogin,
    Options,
    Access,
    Datacenters,
    Clustering,
    Ascii,
    Bigint,
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
    TinyInt,
    Text,
    UUID,
    Varchar,
    VarInt,
    TimeUUID,
    Token,
    WriteTime,
    Date,
    Time,
    Null,
    Not,
    Exists,
    Map,
    List,
    NaN,
    Infinity,
    Tuple,
    Trigger,
    Static,
    Frozen,
    Function,
    Functions,
    Aggregate,
    Aggregates,
    SFunc,
    SType,
    FinalFunc,
    InitCond,
    Returns,
    Called,
    Input,
    Language,
    Or,
    Replace,
    Json,
    Default,
    Unset,
    Like,
}

impl Keyword {
    /// Returns `Some(Keyword)` if given `s` is a keyword
    /// Otherwise, returns `None`.
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_ref() {
            "SELECT" => Some(Keyword::Select),
            "FROM" => Some(Keyword::From),
            "AS" => Some(Keyword::As),
            "WHERE" => Some(Keyword::Where),
            "AND" => Some(Keyword::And),
            "KEY" => Some(Keyword::Key),
            "KEYS" => Some(Keyword::Keys),
            "ENTRIES" => Some(Keyword::Entries),
            "FULL" => Some(Keyword::Full),
            "INSERT" => Some(Keyword::Insert),
            "UPDATE" => Some(Keyword::Update),
            "WITH" => Some(Keyword::With),
            "LIMIT" => Some(Keyword::Limit),
            "PER" => Some(Keyword::Per),
            "PARTITION" => Some(Keyword::Partition),
            "USING" => Some(Keyword::Using),
            "USE" => Some(Keyword::Use),
            "DISTINCT" => Some(Keyword::Distinct),
            "COUNT" => Some(Keyword::Count),
            "SET" => Some(Keyword::Set),
            "BEGIN" => Some(Keyword::Begin),
            "UNLOGGED" => Some(Keyword::Unlogged),
            "BATCH" => Some(Keyword::Batch),
            "APPLY" => Some(Keyword::Apply),
            "TRUNCATE" => Some(Keyword::Truncate),
            "DELETE" => Some(Keyword::Delete),
            "IN" => Some(Keyword::In),
            "CREATE" => Some(Keyword::Create),
            "SCHEMA" => Some(Keyword::Schema),
            "KEYSPACE" => Some(Keyword::Keyspace),
            "KEYSPACES" => Some(Keyword::Keyspaces),
            "COLUMNFAMILY" | "TABLE" => Some(Keyword::Table),
            "COLUMNFAMILIES" | "TABLES" => Some(Keyword::Tables),
            "MATERIALIZED" => Some(Keyword::Materialized),
            "VIEW" => Some(Keyword::View),
            "INDEX" => Some(Keyword::Index),
            "CUSTOM" => Some(Keyword::Custom),
            "ON" => Some(Keyword::On),
            "TO" => Some(Keyword::To),
            "DROP" => Some(Keyword::Drop),
            "PRIMARY" => Some(Keyword::Primary),
            "INTO" => Some(Keyword::Into),
            "VALUES" => Some(Keyword::Values),
            "TIMESTAMP" => Some(Keyword::Timestamp),
            "TTL" => Some(Keyword::Ttl),
            "CAST" => Some(Keyword::Cast),
            "ALTER" => Some(Keyword::Alter),
            "RENAME" => Some(Keyword::Rename),
            "ADD" => Some(Keyword::Add),
            "TYPE" => Some(Keyword::Type),
            "TYPES" => Some(Keyword::Types),
            "COMPACT" => Some(Keyword::Compact),
            "STORAGE" => Some(Keyword::Storage),
            "ORDER" => Some(Keyword::Order),
            "BY" => Some(Keyword::By),
            "ASC" => Some(Keyword::Asc),
            "DESC" => Some(Keyword::Desc),
            "ALLOW" => Some(Keyword::Allow),
            "FILTERING" => Some(Keyword::Filtering),
            "IF" => Some(Keyword::If),
            "IS" => Some(Keyword::Is),
            "CONTAINS" => Some(Keyword::Contains),
            "GROUP" => Some(Keyword::Group),
            "CLUSTER" => Some(Keyword::Cluster),
            "INTERNALS" => Some(Keyword::Internals),
            "ONLY" => Some(Keyword::Only),

            "GRANT" => Some(Keyword::Grant),
            "ALL" => Some(Keyword::All),
            "PERMISSION" => Some(Keyword::Permission),
            "PERMISSIONS" => Some(Keyword::Permissions),
            "OF" => Some(Keyword::Of),
            "REVOKE" => Some(Keyword::Revoke),
            "MODIFY" => Some(Keyword::Modify),
            "AUTHORIZE" => Some(Keyword::Authorize),
            "DESCRIBE" => Some(Keyword::Describe),
            "EXECUTE" => Some(Keyword::Execute),
            "NORECURSIVE" => Some(Keyword::NoRecursive),
            "MBEAN" => Some(Keyword::MBean),
            "MBEANS" => Some(Keyword::MBeans),

            "USER" => Some(Keyword::User),
            "USERS" => Some(Keyword::Users),
            "ROLE" => Some(Keyword::Role),
            "ROLES" => Some(Keyword::Roles),
            "SUPERUSER" => Some(Keyword::Superuser),
            "NOSUPERUSER" => Some(Keyword::NoSuperuser),
            "PASSWORD" => Some(Keyword::Password),
            "LOGIN" => Some(Keyword::Login),
            "NOLOGIN" => Some(Keyword::NoLogin),
            "OPTIONS" => Some(Keyword::Options),
            "ACCESS" => Some(Keyword::Access),
            "DATACENTERS" => Some(Keyword::Datacenters),

            "CLUSTERING" => Some(Keyword::Clustering),
            "ASCII" => Some(Keyword::Ascii),
            "BIGINT" => Some(Keyword::Bigint),
            "BLOB" => Some(Keyword::Blob),
            "BOOLEAN" => Some(Keyword::Boolean),
            "COUNTER" => Some(Keyword::Counter),
            "DECIMAL" => Some(Keyword::Decimal),
            "DOUBLE" => Some(Keyword::Double),
            "DURATION" => Some(Keyword::Duration),
            "FLOAT" => Some(Keyword::Float),
            "INET" => Some(Keyword::Inet),
            "INT" => Some(Keyword::Int),
            "SMALLINT" => Some(Keyword::SmallInt),
            "TINYINT" => Some(Keyword::TinyInt),
            "TEXT" => Some(Keyword::Text),
            "UUID" => Some(Keyword::UUID),
            "VARCHAR" => Some(Keyword::Varchar),
            "VARINT" => Some(Keyword::VarInt),
            "TIMEUUID" => Some(Keyword::TimeUUID),
            "TOKEN" => Some(Keyword::Token),
            "WRITETIME" => Some(Keyword::WriteTime),
            "DATE" => Some(Keyword::Date),
            "TIME" => Some(Keyword::Time),

            "NULL" => Some(Keyword::Null),
            "NOT" => Some(Keyword::Not),
            "EXISTS" => Some(Keyword::Exists),

            "MAP" => Some(Keyword::Map),
            "LIST" => Some(Keyword::List),
            "TUPLE" => Some(Keyword::Tuple),

            // these are kind of float
            "NAN" => Some(Keyword::NaN),
            "INFINITY" => Some(Keyword::Infinity),

            "TRIGGER" => Some(Keyword::Trigger),
            "STATIC" => Some(Keyword::Static),
            "FROZEN" => Some(Keyword::Frozen),

            "FUNCTION" => Some(Keyword::Function),
            "FUNCTIONS" => Some(Keyword::Functions),
            "AGGREGATE" => Some(Keyword::Aggregate),
            "AGGREGATES" => Some(Keyword::Aggregates),
            "SFUNC" => Some(Keyword::SFunc),
            "STYPE" => Some(Keyword::SType),
            "FINALFUNC" => Some(Keyword::FinalFunc),
            "INITCOND" => Some(Keyword::InitCond),
            "RETURNS" => Some(Keyword::Returns),
            "CALLED" => Some(Keyword::Called),
            "INPUT" => Some(Keyword::Input),
            "LANGUAGE" => Some(Keyword::Language),
            "OR" => Some(Keyword::Or),
            "REPLACE" => Some(Keyword::Replace),

            "JSON" => Some(Keyword::Json),
            "DEFAULT" => Some(Keyword::Default),
            "UNSET" => Some(Keyword::Unset),
            "LIKE" => Some(Keyword::Like),
            _ => None,
        }
    }

    /// Returns true if this is reserved keyword.
    ///
    /// Reserved keywords are defined in
    /// [Apache Cassandra's reserved keyword file][1].
    ///
    /// ---
    /// [1]: https://github.com/apache/cassandra/blob/cassandra-4.0.0/src/resources/org/apache/cassandra/cql3/reserved_keywords.txt
    pub fn is_reserved(&self) -> bool {
        match self {
            Keyword::Add
            | Keyword::Allow
            | Keyword::Alter
            | Keyword::And
            | Keyword::Apply
            | Keyword::Asc
            | Keyword::Authorize
            | Keyword::Batch
            | Keyword::Begin
            | Keyword::By
            | Keyword::Create
            | Keyword::Delete
            | Keyword::Desc
            | Keyword::Describe
            | Keyword::Drop
            | Keyword::Entries
            | Keyword::Execute
            | Keyword::From
            | Keyword::Full
            | Keyword::Grant
            | Keyword::If
            | Keyword::In
            | Keyword::Index
            | Keyword::Infinity
            | Keyword::Insert
            | Keyword::Into
            | Keyword::Is
            | Keyword::Keyspace
            | Keyword::Limit
            | Keyword::Materialized
            | Keyword::Modify
            | Keyword::NaN
            | Keyword::NoRecursive
            | Keyword::Not
            | Keyword::Null
            | Keyword::Of
            | Keyword::On
            | Keyword::Or
            | Keyword::Order
            | Keyword::Primary
            | Keyword::Rename
            | Keyword::Revoke
            | Keyword::Select
            | Keyword::Set
            | Keyword::Table
            | Keyword::To
            | Keyword::Token
            | Keyword::Truncate
            | Keyword::Unlogged
            | Keyword::Update
            | Keyword::Use
            | Keyword::Using
            | Keyword::View
            | Keyword::Where
            | Keyword::With
            // | Keyword::Replace
            // | Keyword::Default
            // | Keyword::Unset
            // | Keyword::MBean
            // | Keyword::MBeans
            => true,
            _ => false,
        }
    }

    /// Returns true if this keyword can be used as identifier,
    /// as defined in [`unreserved_keyword` in Parser.g][1]
    ///
    /// ---
    /// [1]: https://github.com/apache/cassandra/blob/cassandra-4.0.0/
    pub fn is_unreserved_keyword(&self) -> bool {
        self.is_basic_unreserved_keyword()
            | self.is_native_type()
            | match self {
                Keyword::Ttl
                | Keyword::Count
                | Keyword::WriteTime
                | Keyword::Key
                | Keyword::Cast
                | Keyword::Json
                | Keyword::Distinct => true,
                _ => false,
            }
    }

    /// Returns true if this keyword can be used as function name,
    /// as defined in [`allowedFunctionName` in Parser.g][1]
    ///
    /// ---
    /// [1]: https://github.com/apache/cassandra/blob/cassandra-4.0.0/
    pub fn is_unreserved_for_function_name(&self) -> bool {
        self.is_basic_unreserved_keyword()
            | self.is_native_type()
            | match self {
                Keyword::Token | Keyword::Count => true,
                _ => false,
            }
    }

    /// Returns true if this keyword describes CQL3 native data type.
    pub fn is_native_type(&self) -> bool {
        match self {
            Keyword::Ascii
            | Keyword::Bigint
            | Keyword::Blob
            | Keyword::Boolean
            | Keyword::Counter
            | Keyword::Decimal
            | Keyword::Double
            | Keyword::Duration
            | Keyword::Float
            | Keyword::Inet
            | Keyword::Int
            | Keyword::SmallInt
            | Keyword::Text
            | Keyword::Timestamp
            | Keyword::TinyInt
            | Keyword::UUID
            | Keyword::Varchar
            | Keyword::VarInt
            | Keyword::TimeUUID
            | Keyword::Date
            | Keyword::Time => true,
            _ => false,
        }
    }

    pub fn is_basic_unreserved_keyword(&self) -> bool {
        match self {
            Keyword::Keys
            | Keyword::As
            | Keyword::Cluster
            | Keyword::Clustering
            | Keyword::Compact
            | Keyword::Storage
            | Keyword::Tables
            | Keyword::Type
            | Keyword::Types
            | Keyword::Values
            | Keyword::Map
            | Keyword::List
            | Keyword::Filtering
            | Keyword::Permission
            | Keyword::Permissions
            | Keyword::Keyspaces
            | Keyword::All
            | Keyword::User
            | Keyword::Users
            | Keyword::Role
            | Keyword::Roles
            | Keyword::Superuser
            | Keyword::NoSuperuser
            | Keyword::Login
            | Keyword::NoLogin
            | Keyword::Options
            | Keyword::Password
            | Keyword::Exists
            | Keyword::Custom
            | Keyword::Trigger
            | Keyword::Contains
            | Keyword::Internals
            | Keyword::Only
            | Keyword::Static
            | Keyword::Frozen
            | Keyword::Tuple
            | Keyword::Function
            | Keyword::Functions
            | Keyword::Aggregate
            | Keyword::Aggregates
            | Keyword::SFunc
            | Keyword::SType
            | Keyword::FinalFunc
            | Keyword::InitCond
            | Keyword::Returns
            | Keyword::Language
            | Keyword::Called
            | Keyword::Input
            | Keyword::Like
            | Keyword::Per
            | Keyword::Partition
            | Keyword::Group
            | Keyword::Datacenters
            | Keyword::Access
            | Keyword::Default
            | Keyword::MBean
            | Keyword::MBeans
            | Keyword::Replace
            | Keyword::Unset => true,
            _ => false,
        }
    }
}

/// CQL Lexer
///
/// Tokenize CQL
/// Implements iterator to produce `Token`s
#[derive(Debug)]
pub struct Lexer<'a> {
    original: &'a str,
    cql: Peekable<Chars<'a>>,
    token_start: usize,
    token_end: usize,
}

impl<'a> Lexer<'a> {
    /// Create new lexer for given CQL string.
    pub fn new(cql: &'a str) -> Self {
        Lexer {
            original: cql,
            cql: cql.chars().peekable(),
            token_start: 0,
            token_end: 0,
        }
    }

    fn consume_and_create_token(&mut self, token_type: TokenType) -> (&'a str, Token) {
        self.advance();
        self.create_token(token_type)
    }

    fn create_token(&self, token_type: TokenType) -> (&'a str, Token) {
        (
            self.original
                .get(self.token_start..self.token_end)
                .unwrap_or_default(),
            Token::new(
                token_type,
                self.token_start,
                self.token_end - self.token_start,
            ),
        )
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.cql.next();
        if let Some(ch) = c {
            self.token_end += ch.len_utf8();
        }
        c
    }

    // String literal
    // - Quoted string literal
    // 'abc'
    // inside quoted string literal, quote(') can be escaped by putting two together ('')
    fn string_literal(&mut self) -> (&'a str, Token) {
        self.advance();
        let mut in_string = true;
        while let Some(c) = self.advance() {
            if c == '\'' {
                if let Some(&n) = self.cql.peek() {
                    if n != '\'' {
                        // not escaped single quote
                        in_string = false;
                        break;
                    } else {
                        self.advance();
                    }
                } else {
                    in_string = false;
                    break;
                }
            }
        }
        let token_type = if in_string {
            TokenType::Error
        } else {
            TokenType::StringLiteral
        };
        self.create_token(token_type)
    }

    // Pg style string literal
    fn pg_string_literal(&mut self) -> (&'a str, Token) {
        self.advance(); // skip second '$'
        let mut in_string = true;
        while let Some(c) = self.advance() {
            if c == '$' {
                if let Some(&n) = self.cql.peek() {
                    if n == '$' {
                        self.advance();
                        in_string = false;
                        break;
                    }
                }
            }
        }
        let token_type = if in_string {
            TokenType::Error
        } else {
            TokenType::StringLiteral
        };
        self.create_token(token_type)
    }

    // Catch all for token that begins with ascii alphabet character.
    //
    // Can be:
    // - UUID
    // - Duration (ISO8601 format)
    // - One of reserved keywords
    // - Identifier
    fn parse_alphabet(&mut self) -> (&'a str, Token) {
        let mut uuid = UUIDParser::new();
        let mut duration = Iso8601Parser::new();
        let mut duration_alt = Iso8601AlternativeParser::new();

        let mut accept = [true; 4];
        let mut length = [0u32; 4];
        while let Some(&c) = self.cql.peek() {
            for i in 0..accept.len() {
                if accept[i] {
                    accept[i] = match i {
                        0 => uuid.accept(&c),
                        1 => duration.accept(&c),
                        2 => duration_alt.accept(&c),
                        3 => match c {
                            '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' => true,
                            _ => false,
                        },
                        _ => unreachable!(),
                    };
                    if accept[i] {
                        length[i] += 1;
                    }
                }
            }
            if accept.iter().any(|b| *b) {
                self.advance();
            } else {
                break;
            }
        }
        // max length of chars
        let max = length.iter().max().unwrap();
        // find indexes of accepted parsers
        for (idx, len) in length.iter().enumerate() {
            if len == max {
                if idx == 0 && uuid.is_valid() {
                    return self.create_token(TokenType::UUID);
                } else if (idx == 1 && duration.is_valid()) || (idx == 2 && duration_alt.is_valid())
                {
                    return self.create_token(TokenType::Duration);
                } else if idx == 3 {
                    let token_type = match self
                        .original
                        .get(self.token_start..self.token_end)
                        .map(str::to_ascii_uppercase)
                    {
                        Some(s) => match s.as_str() {
                            "TRUE" | "FALSE" => TokenType::Boolean,
                            _ => Keyword::from_string(&s)
                                // .filter(Keyword::is_reserved)
                                .map(TokenType::Keyword)
                                .unwrap_or(TokenType::Identifier),
                        },
                        _ => TokenType::Error,
                    };
                    return self.create_token(token_type);
                }
            }
        }
        self.create_token(TokenType::Error)
    }

    // Quoted Identifier
    // Double quote (`"`) inside quoted identifier can be escaped by putting it twice (`""`).
    fn quoted_identifier(&mut self) -> (&'a str, Token) {
        self.advance();
        let mut in_quote = true;
        while let Some(c) = self.advance() {
            if c == '"' {
                // if the next char is '"' again, it is escaped double quote
                match self.cql.peek() {
                    Some('"') => {
                        self.advance();
                    }
                    _ => {
                        in_quote = false;
                        break;
                    }
                }
            }
        }
        let token_type = if in_quote {
            TokenType::Error
        } else {
            TokenType::QuotedName
        };
        self.create_token(token_type)
    }

    // Catch all for token that begins with ascii digit character.
    //
    // The token can be either
    // - Hexnumber
    // - Duration
    // - UUID
    // - Float
    // - Integer
    fn parse_digit(&mut self) -> (&'a str, Token) {
        let mut duration = DurationUnitParser::new();
        let mut uuid = UUIDParser::new();
        let mut hexnumber = HexnumberParser::new();
        let mut numeric = NumberParser::new();

        let mut accept = [true; 4];
        let mut length = [0u64; 4];
        while let Some(&c) = self.cql.peek() {
            for i in 0..accept.len() {
                if accept[i] {
                    accept[i] = match i {
                        0 => duration.accept(&c),
                        1 => uuid.accept(&c),
                        2 => hexnumber.accept(&c),
                        3 => numeric.accept(&c),
                        _ => unreachable!(),
                    };
                    if accept[i] {
                        length[i] += 1;
                    }
                }
            }
            if accept.iter().any(|b| *b) {
                self.advance();
            } else {
                break;
            }
        }
        // max length of chars
        let max = length.iter().max().unwrap();
        // find indexes of accepted parsers
        for (idx, len) in length.iter().enumerate() {
            if len == max {
                if idx == 0 && duration.is_valid() {
                    return self.create_token(TokenType::Duration);
                } else if idx == 1 && uuid.is_valid() {
                    return self.create_token(TokenType::UUID);
                } else if idx == 2 && hexnumber.is_valid() {
                    return self.create_token(TokenType::Hexnumber);
                } else if idx == 3 && numeric.is_valid() {
                    return if numeric.is_float() {
                        self.create_token(TokenType::Float)
                    } else {
                        self.create_token(TokenType::Integer)
                    };
                }
            }
        }
        self.create_token(TokenType::Error)
    }

    // Whitespace
    // (' ' | '\t' | '\n' | '\r')+
    fn whitespace(&mut self) -> (&'a str, Token) {
        while let Some(&c) = self.cql.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                }
                _ => break,
            }
        }
        self.create_token(TokenType::Whitespace)
    }

    // Single line comment
    fn singleline_comment(&mut self) -> (&'a str, Token) {
        while let Some(c) = self.advance() {
            match c {
                '\n' => break,
                '\r' => {
                    // CRLF case
                    if let Some('\n') = self.cql.peek() {
                        self.advance();
                    }
                    break;
                }
                _ => continue,
            }
        }
        self.create_token(TokenType::Comment(false))
    }

    // Multiline comment
    fn multiline_comment(&mut self) -> (&'a str, Token) {
        while let Some(c) = self.advance() {
            match c {
                // end of multiline comment
                '*' => {
                    if let Some('/') = self.cql.peek() {
                        // remove previously added '*'
                        self.advance();
                        return self.create_token(TokenType::Comment(true));
                    }
                }
                _ => continue,
            }
        }
        self.create_token(TokenType::Error)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (&'a str, Token);

    fn next(&mut self) -> Option<(&'a str, Token)> {
        self.token_start = self.token_end;
        if let Some(c) = self.cql.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => Some(self.whitespace()),
                '+' => Some(self.consume_and_create_token(TokenType::Plus)),
                '*' => Some(self.consume_and_create_token(TokenType::Asterisk)),
                '=' => Some(self.consume_and_create_token(TokenType::Equal)),
                ';' => Some(self.consume_and_create_token(TokenType::SemiColon)),
                ':' => Some(self.consume_and_create_token(TokenType::Colon)),
                ',' => Some(self.consume_and_create_token(TokenType::Comma)),
                '(' => Some(self.consume_and_create_token(TokenType::LParen)),
                ')' => Some(self.consume_and_create_token(TokenType::RParen)),
                '[' => Some(self.consume_and_create_token(TokenType::LBracket)),
                ']' => Some(self.consume_and_create_token(TokenType::RBracket)),
                '{' => Some(self.consume_and_create_token(TokenType::LBrace)),
                '}' => Some(self.consume_and_create_token(TokenType::RBrace)),
                '.' => {
                    self.advance();
                    match self.cql.peek() {
                        Some('.') => {
                            self.advance();
                            Some(self.create_token(TokenType::Range))
                        }
                        _ => Some(self.create_token(TokenType::Dot)),
                    }
                }
                '>' => {
                    self.advance();
                    match self.cql.peek() {
                        Some('=') => {
                            self.advance();
                            Some(self.create_token(TokenType::Gte))
                        }
                        _ => Some(self.create_token(TokenType::Gt)),
                    }
                }
                '<' => {
                    self.advance();
                    match self.cql.peek() {
                        Some('=') => {
                            self.advance();
                            Some(self.create_token(TokenType::Lte))
                        }
                        _ => Some(self.create_token(TokenType::Lt)),
                    }
                }
                '\'' => Some(self.string_literal()),
                '$' => {
                    self.advance();
                    match self.cql.peek() {
                        Some('$') => Some(self.pg_string_literal()),
                        _ => Some(self.create_token(TokenType::StringLiteral)), //TODO maybe emit single char ('$')
                    }
                }
                '"' => Some(self.quoted_identifier()),
                '/' => {
                    self.advance();
                    match self.cql.peek() {
                        Some('/') => {
                            self.advance();
                            Some(self.singleline_comment())
                        }
                        Some('*') => {
                            self.advance();
                            Some(self.multiline_comment())
                        }
                        _ => Some(self.create_token(TokenType::Slash)),
                    }
                }
                '-' => {
                    self.advance();
                    match self.cql.peek() {
                        Some('-') => {
                            self.advance();
                            Some(self.singleline_comment())
                        }
                        _ => Some(self.create_token(TokenType::Minus)),
                    }
                }
                c if c.is_ascii_digit() => Some(self.parse_digit()),
                c if c.is_ascii_alphabetic() => Some(self.parse_alphabet()),
                _ => Some(self.consume_and_create_token(TokenType::Error)),
            }
        } else {
            None
        }
    }
}
