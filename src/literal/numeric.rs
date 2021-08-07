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

//! Module for parsing numeric literals
use super::{StateMachine, StateTransition};

enum NumericState {
    Initial,
    Integer,
    FloatingPoint,
    Float,
    Exponent,
    PlusMinus,
    ExponentDigit,
    /// INTEGER '..' case
    IntegerRange,
    /// INTEGER '.' '..' case
    FloatRange,
}

impl StateTransition for NumericState {
    fn is_final(&self) -> bool {
        match self {
            NumericState::Integer
            | NumericState::FloatingPoint
            | NumericState::Float
            | NumericState::ExponentDigit
            | NumericState::IntegerRange
            | NumericState::FloatRange => true,
            _ => false,
        }
    }

    fn next_state(&self, c: &char) -> Result<Self, ()> {
        match self {
            Self::Initial => match c {
                '0'..='9' => Ok(Self::Integer),
                _ => Err(()),
            },
            Self::Integer => match c {
                '0'..='9' => Ok(Self::Integer),
                '.' => Ok(Self::FloatingPoint),
                'E' | 'e' => Ok(Self::Exponent),
                _ => Err(()),
            },
            Self::FloatingPoint => match c {
                '0'..='9' => Ok(Self::Float),
                '.' => Ok(Self::IntegerRange),
                _ => Err(()),
            },
            Self::Float => match c {
                '0'..='9' => Ok(Self::Float),
                'E' | 'e' => Ok(Self::Exponent),
                _ => Err(()),
            },
            Self::Exponent => match c {
                '0'..='9' => Ok(Self::ExponentDigit),
                '+' | '-' => Ok(Self::PlusMinus),
                _ => Err(()),
            },
            Self::PlusMinus => match c {
                '0'..='9' => Ok(Self::ExponentDigit),
                _ => Err(()),
            },
            Self::ExponentDigit => match c {
                '0'..='9' => Ok(Self::ExponentDigit),
                _ => Err(()),
            },
            Self::IntegerRange => match c {
                '.' => Ok(Self::FloatRange),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

/// Number literal parser
pub struct NumberParser {
    state: StateMachine<NumericState>,
}

impl NumberParser {
    pub fn new() -> Self {
        Self {
            state: StateMachine::new(NumericState::Initial),
        }
    }

    pub fn accept(&mut self, c: &char) -> bool {
        self.state.accept(c)
    }

    pub fn is_float(&self) -> bool {
        self.state.is_final()
            && match self.state.state {
                NumericState::FloatingPoint | NumericState::Float | NumericState::ExponentDigit => {
                    true
                }
                _ => false,
            }
    }

    pub fn is_valid(&self) -> bool {
        self.state.is_final()
    }
}

#[derive(PartialEq)]
enum HexnumberState {
    Initial,
    ZeroParsed,
    PrefixParsed,
    HexParsing,
}

impl StateTransition for HexnumberState {
    fn next_state(&self, c: &char) -> Result<Self, ()> {
        match self {
            HexnumberState::Initial => match c {
                '0' => Ok(HexnumberState::ZeroParsed),
                _ => Err(()),
            },
            HexnumberState::ZeroParsed => match c {
                'X' | 'x' => Ok(HexnumberState::PrefixParsed),
                _ => Err(()),
            },
            HexnumberState::PrefixParsed | HexnumberState::HexParsing => {
                if c.is_ascii_hexdigit() {
                    Ok(HexnumberState::HexParsing)
                } else {
                    Err(())
                }
            }
        }
    }

    fn is_final(&self) -> bool {
        *self == HexnumberState::HexParsing
    }
}

/// Hexnumber literal parser
pub struct HexnumberParser {
    state: StateMachine<HexnumberState>,
}

impl HexnumberParser {
    pub fn new() -> Self {
        Self {
            state: StateMachine::new(HexnumberState::Initial),
        }
    }

    pub fn accept(&mut self, c: &char) -> bool {
        self.state.accept(c)
    }

    pub fn is_valid(&self) -> bool {
        self.state.is_final()
    }
}

#[cfg(test)]
impl std::str::FromStr for NumberParser {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = NumberParser::new();
        let chars = s.chars();
        for c in chars {
            if !parser.accept(&c) {
                break;
            }
        }
        if parser.is_valid() {
            Ok(parser)
        } else {
            Err("Invalid".to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use super::NumberParser;

    #[test]
    fn test_numerics() {
        assert!("100".parse::<NumberParser>().unwrap().is_valid());
        assert!(!"100".parse::<NumberParser>().unwrap().is_float());
        assert!("100.".parse::<NumberParser>().unwrap().is_valid());
        assert!("100.".parse::<NumberParser>().unwrap().is_float());
        assert!("100.0".parse::<NumberParser>().unwrap().is_valid());
        assert!("100.0".parse::<NumberParser>().unwrap().is_float());
        assert!("100e10".parse::<NumberParser>().unwrap().is_valid());
        assert!("100e10".parse::<NumberParser>().unwrap().is_float());
        assert!("100E+1".parse::<NumberParser>().unwrap().is_valid());
        assert!("100E+1".parse::<NumberParser>().unwrap().is_float());
        assert!("100E-1".parse::<NumberParser>().unwrap().is_valid());
        assert!("100E-1".parse::<NumberParser>().unwrap().is_float());
        assert!("100.E1".parse::<NumberParser>().unwrap().is_valid());
        assert!("100.E1".parse::<NumberParser>().unwrap().is_float());
        assert!("100.0E1".parse::<NumberParser>().unwrap().is_valid());
        assert!("100.0E1".parse::<NumberParser>().unwrap().is_float());
        assert!("100.0e+1".parse::<NumberParser>().unwrap().is_valid());
        assert!("100.0e+1".parse::<NumberParser>().unwrap().is_float());

        // with ranges
        assert!("100..".parse::<NumberParser>().unwrap().is_valid());
        assert!("100...".parse::<NumberParser>().unwrap().is_valid());

        assert!("abc".parse::<NumberParser>().is_err());
    }
}
