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

use std::{
    env,
    fs::File,
    io::{Read, Result},
    path::Path,
};

use cqlparser::Parser;

/// Dumps parsed AST from the schema file output from `desc keyspace` command.
pub fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        // no arguments passed
        1 => {
            println!("usage: {} <path to schema cql file>", args[0]);
        }
        // one argument passed
        2 => {
            // Create a path to the desired file
            let path = Path::new(&args[1]);

            // Open the path in read-only mode, returns `io::Result<File>`
            let mut file = File::open(&path)?;

            // Read the file contents into a string, returns `io::Result<usize>`
            let mut s = String::new();
            file.read_to_string(&mut s)?;
            let parser = Parser::new(&s);
            match parser.parse() {
                Ok(stmts) => {
                    for stmt in stmts.into_iter() {
                        println!("{:?}", stmt);
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        _ => {
            println!("usage: {} <path to schema cql file>", args[0]);
        }
    }
    Ok(())
}
