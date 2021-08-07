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

//! Apache Cassandra CQL(Cassandra Query Language) parser.
//!
//! The library is aimed for providing AST of CQL statements,
//! with comprehensive error messages when parsing the statements fails.

#![forbid(unsafe_code)]
//#![warn(missing_docs)]
//#![warn(missing_doc_code_examples)]

pub mod ast;
mod error;
mod lexer;
mod literal;
mod parser;

pub use error::ParseError;
pub use lexer::{Keyword, Lexer, Token, TokenType};
pub use parser::Parser;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn parse(s: &str) -> Result<JsValue, JsValue> {
    match Parser::new(s).parse() {
        Ok(stmts) => Ok(serde_wasm_bindgen::to_value(&stmts)?),
        Err(e) => Err(serde_wasm_bindgen::to_value(&e)?),
    }
}
