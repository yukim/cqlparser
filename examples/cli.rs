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

use cqlparser::Parser;
use std::io::{self, Result, Write};

pub fn main() -> Result<()> {
    loop {
        print!("cql> ");
        let _ = io::stdout().flush();

        let stdin = io::stdin();
        let mut raw_input = String::new();
        match stdin.read_line(&mut raw_input) {
            Ok(_) => {
                let input = str::trim(&raw_input);
                if input.eq_ignore_ascii_case("exit") {
                    break;
                }
                let p = Parser::new(input);
                println!("{:?}", p.parse());
            }
            Err(error) => println!("error: {}", error),
        }
    }
    Ok(())
}
