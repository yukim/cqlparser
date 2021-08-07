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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseError {
    path: Option<String>,
    line: String,
    continued_line: Option<String>,
}

impl ParseError {
    pub fn new() -> Self {
        ParseError {
            path: None,
            line: String::from(""),
            continued_line: None,
        }
    }

    pub fn with_message(message: String) -> Self {
        ParseError {
            path: None,
            line: message,
            continued_line: None,
        }
    }
}
