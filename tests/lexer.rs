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

use cqlparser::{Lexer, Token, TokenType};

macro_rules! test_tokenize {
    ($input:literal, $expected_token:expr) => {
        let input = $input;
        let mut lexer = Lexer::new(input);
        let (s, token) = lexer.next().unwrap();
        assert_eq!(s, input);
        assert_eq!(token, Token::new($expected_token, 0, input.len()));
    };
}

#[test]
fn tokenize_string_literal() {
    test_tokenize!("'test'", TokenType::StringLiteral);
    test_tokenize!("'te''st'", TokenType::StringLiteral);
    // Unclosed string literal
    test_tokenize!("'test", TokenType::Error);
    // PG style string literal
    test_tokenize!("$$It's a test$$", TokenType::StringLiteral);
    // Unclosed PG style string literal
    test_tokenize!("$$It's a test$", TokenType::Error);
    test_tokenize!("$$It's a test", TokenType::Error);
}

#[test]
fn tokenize_ident() {
    test_tokenize!("c", TokenType::Identifier);
    test_tokenize!("col_1", TokenType::Identifier);
    test_tokenize!("\"Quoted ident\"", TokenType::QuotedName);
    // Empty quoted identifier
    test_tokenize!("\"\"", TokenType::QuotedName);
    // Escaped double quote
    test_tokenize!("\"escaped \"\" quotes \"\"\"", TokenType::QuotedName);
    // Unclosed quoted identifier
    test_tokenize!("\"Quoted ident", TokenType::Error);
    // Quoted identifier with multi byte unicode
    test_tokenize!("\"�\"", TokenType::QuotedName);

    test_tokenize!("2cab", TokenType::Error);
}

#[test]
fn tokenize_numbers() {
    test_tokenize!("0xDeadBeef", TokenType::Hexnumber);
}

#[test]
fn tokenize_uuid() {
    test_tokenize!("cbad2f6e-3fba-a2b1-bd0a-bd31bb0d0b40", TokenType::UUID);
    test_tokenize!("CBAD2F6E-3FBA-A2B1-BD0A-BD31BB0D0B40", TokenType::UUID);
    test_tokenize!("99b914b5-1382-4d84-a4b4-f244f40b833c", TokenType::UUID);
    test_tokenize!("cbad2f6e-3fba", TokenType::Error);
    test_tokenize!("cbad2f6e-", TokenType::Error);
}

#[test]
fn tokenize_duration() {
    // Duration unit
    test_tokenize!("123us", TokenType::Duration);
    test_tokenize!("123µs", TokenType::Duration);
    // ISO 8601 formats
    // 'P'/'PT' produce Duration token in Apache Cassandra.
    // I think this is not valid ISO8601 thuogh...
    test_tokenize!("P", TokenType::Duration);
    test_tokenize!("PT", TokenType::Duration);
    test_tokenize!("P1Y2M3D", TokenType::Duration);
    test_tokenize!("P1Y2M3DT9H8M6S", TokenType::Duration);
    test_tokenize!("P1W", TokenType::Duration);
    test_tokenize!("P2020-01-02T12:23:34", TokenType::Duration);

    // P\d{4} should be identified as `Identifier`
    test_tokenize!("P2020", TokenType::Identifier);
    // though P\d{4}- should be identified as `Error`
    test_tokenize!("P2020-", TokenType::Error);
    // Identifier chars after proper duration is identified as `Identifier`
    test_tokenize!("P1W1", TokenType::Identifier);
    test_tokenize!("P1Y_", TokenType::Identifier);
    test_tokenize!("PT_1", TokenType::Identifier);
}

#[test]
fn tokenize_singleline_comment() {
    // EOF
    test_tokenize!("-- This is a comment", TokenType::Comment(false));
    // CRLF
    test_tokenize!("-- This is a comment\r\n", TokenType::Comment(false));
    // LF with '//'
    test_tokenize!("// This is a comment\n", TokenType::Comment(false));
}

#[test]
fn multiline_comment_test() {
    test_tokenize!(
        "/*
 multiline
 comment
*/",
        TokenType::Comment(true)
    );
}

#[test]
fn create_table_test() {
    let mut lexer = Lexer::new(
        "
CREATE TABLE IF NOT EXISTS app.users (
user_id UUID,
updated_at timestamp,
name text,
PRIMARY KEY (user_id, updated_at)
) WITH comment = 'Table for user''s name history'
AND value = true
AND CLUSTERING ORDER BY (updated_at DESC);
    ",
    );
    while let Some(t) = lexer.next() {
        println!("{:?}", t);
    }
}
