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

use super::{StateMachine, StateTransition};

#[derive(PartialEq, Debug)]
pub enum UUIDParsingState {
    Initial,
    /// 8 hex digits.
    /// Integer giving the low 32 bits of the time.
    TimeLow(u8),
    /// 4 hex digits.
    /// Integer giving the middle 16 bits of the time
    TimeMid(u8),
    /// 4 hex digits.
    /// 4-bit "version" in the most significant bits,
    /// followed by the high 12 bits of the time.
    TimeHiAndVersion(u8),
    /// 4 hex digits.
    /// 1 to 3-bit "variant" in the most significant bits,
    /// followed by the 13 to 15-bit clock sequence.
    ClockSeg(u8),
    /// 12 hex digits
    /// The 48-bit node id
    Node(u8),
    End,
}

impl StateTransition for UUIDParsingState {
    fn next_state(&self, c: &char) -> Result<Self, ()> {
        match self {
            UUIDParsingState::Initial => {
                if c.is_ascii_hexdigit() {
                    Ok(UUIDParsingState::TimeLow(1))
                } else {
                    Err(())
                }
            }
            UUIDParsingState::TimeLow(digits) => {
                if *digits < 8 && c.is_ascii_hexdigit() {
                    Ok(UUIDParsingState::TimeLow(digits + 1))
                } else if *digits == 8 && *c == '-' {
                    Ok(UUIDParsingState::TimeMid(0))
                } else {
                    Err(())
                }
            }
            UUIDParsingState::TimeMid(digits) => {
                if *digits < 4 && c.is_ascii_hexdigit() {
                    Ok(UUIDParsingState::TimeMid(digits + 1))
                } else if *digits == 4 && *c == '-' {
                    Ok(UUIDParsingState::TimeHiAndVersion(0))
                } else {
                    Err(())
                }
            }
            UUIDParsingState::TimeHiAndVersion(digits) => {
                if *digits < 4 && c.is_ascii_hexdigit() {
                    Ok(UUIDParsingState::TimeHiAndVersion(digits + 1))
                } else if *digits == 4 && *c == '-' {
                    Ok(UUIDParsingState::ClockSeg(0))
                } else {
                    Err(())
                }
            }
            UUIDParsingState::ClockSeg(digits) => {
                if *digits < 4 && c.is_ascii_hexdigit() {
                    Ok(UUIDParsingState::ClockSeg(digits + 1))
                } else if *digits == 4 && *c == '-' {
                    Ok(UUIDParsingState::Node(0))
                } else {
                    Err(())
                }
            }
            UUIDParsingState::Node(digits) => {
                if *digits < 11 && c.is_ascii_hexdigit() {
                    Ok(UUIDParsingState::Node(digits + 1))
                } else if *digits == 11 {
                    Ok(UUIDParsingState::End)
                } else {
                    Err(())
                }
            }
            UUIDParsingState::End => Err(()),
        }
    }

    fn is_final(&self) -> bool {
        *self == UUIDParsingState::End
    }
}

/// UUID Parser
pub struct UUIDParser {
    state: StateMachine<UUIDParsingState>,
}

impl UUIDParser {
    pub fn new() -> Self {
        Self {
            state: StateMachine::new(UUIDParsingState::Initial),
        }
    }

    pub fn accept(&mut self, c: &char) -> bool {
        self.state.accept(c)
    }

    /// return true if this parsed valid UUID
    pub fn is_valid(&self) -> bool {
        self.state.is_final()
    }
}

#[cfg(test)]
impl std::str::FromStr for UUIDParser {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = UUIDParser::new();
        for c in s.chars() {
            if !parser.accept(&c) {
                break;
            }
        }
        if parser.is_valid() {
            Ok(parser)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::UUIDParser;

    #[test]
    fn test_uuid() {
        // valid
        assert!("67e55044-10b1-426f-9247-bb680e5fe0c8"
            .parse::<UUIDParser>()
            .unwrap()
            .is_valid());
        assert!("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4"
            .parse::<UUIDParser>()
            .unwrap()
            .is_valid());
        // invalid character
        assert!("F9168C5E-CEB2-4faaXB6BFF329BF39FA1E4"
            .parse::<UUIDParser>()
            .is_err());
    }
}
