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

//! # CQL literal parsers
//!
//! This module contains literal parser for the following:
//!
//! - Duration literals
//! - Numeric literals
//!     - Integer
//!     - Float
//! - Hexnumber literal
//! - UUID literal
mod duration;
mod numeric;
mod uuid;

pub use duration::{DurationUnitParser, Iso8601AlternativeParser, Iso8601Parser};
pub use numeric::{HexnumberParser, NumberParser};
pub use uuid::UUIDParser;

/// Trait that define transition of states.
pub trait StateTransition: Sized {
    /// Returns next state when given input
    fn next_state(&self, c: &char) -> Result<Self, ()>;

    /// Returns true if in final state
    fn is_final(&self) -> bool;
}

/// State machine that can be used by literal parsers
pub struct StateMachine<S: StateTransition> {
    state: S,
}

impl<S: StateTransition> StateMachine<S> {
    /// Creates new StateMachine with the given initial state
    pub fn new(initial_state: S) -> Self {
        Self {
            state: initial_state,
        }
    }

    /// Returns true if the state machine accept the char and move to the next state.
    /// If the state machine continue to receive char after stop accepting char, `is_error`
    /// turns into `true`.
    pub fn accept(&mut self, c: &char) -> bool {
        match self.state.next_state(c) {
            Ok(next) => {
                self.state = next;
                true
            }
            _ => false,
        }
    }

    pub fn is_final(&self) -> bool {
        self.state.is_final()
    }
}
