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

//! - Number with duration unit (case insensitive)
//!     - `\d+(Y|MO|W|D|H|M|S|MS|US|\u{00B5}S|NS)(\d+(Y|MO|W|D|H|M|S|MS|US|\u{00B5}S|NS))*`
//! - ISO8601 Duration
//!     - Three supported formats:
//!         - Format with designators
//!             - `P(\d+Y)?(\d+M)?(\d+D)?(T(\d+H)?(\d+M)?(\d+S)?)?`
//!         - Week duration
//!             - `P\d+W`
//!         - Alternative format
//!             - `P\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}`

use super::{StateMachine, StateTransition};

// States for parsing duration unit format
#[derive(PartialEq)]
enum DurationUnitParseState {
    Initial,
    ParseDigit,
    YearParsed,
    MonthParsed,
    WeekParsed,
    DayParsed,
    HourParsed,
    MinuteParsed,
    SecondParsed,
    MillisecondParsed,
    MicroParsed,
    MicrosecondParsed,
    NanoParsed,
    NanosecondParsed,
}

impl StateTransition for DurationUnitParseState {
    fn is_final(&self) -> bool {
        match self {
            Self::Initial | Self::ParseDigit => false,
            _ => true,
        }
    }

    fn next_state(&self, c: &char) -> Result<Self, ()> {
        match self {
            Self::Initial => match c {
                '0'..='9' => Ok(Self::ParseDigit),
                _ => Err(()),
            },
            Self::ParseDigit => match c {
                '0'..='9' => Ok(Self::ParseDigit),
                'Y' | 'y' => Ok(Self::YearParsed),
                'W' | 'w' => Ok(Self::WeekParsed),
                'D' | 'd' => Ok(Self::DayParsed),
                'H' | 'h' => Ok(Self::HourParsed),
                'M' | 'm' => Ok(Self::MinuteParsed),
                'S' | 's' => Ok(Self::SecondParsed),
                'U' | 'u' => Ok(Self::MicroParsed),
                'N' | 'n' => Ok(Self::NanoParsed),
                '\u{00B5}' => Ok(Self::MicroParsed),
                _ => Err(()),
            },
            Self::YearParsed
            | Self::MonthParsed
            | Self::WeekParsed
            | Self::DayParsed
            | Self::HourParsed
            | Self::SecondParsed
            | Self::MillisecondParsed
            | Self::MicrosecondParsed
            | Self::NanosecondParsed => match c {
                '0'..='9' => Ok(Self::ParseDigit),
                _ => Err(()),
            },
            Self::MinuteParsed => match c {
                '0'..='9' => Ok(Self::ParseDigit),
                'O' | 'o' => Ok(Self::MonthParsed),
                'S' | 's' => Ok(Self::MillisecondParsed),
                _ => Err(()),
            },
            Self::MicroParsed => match c {
                'S' | 's' => Ok(Self::MicrosecondParsed),
                _ => Err(()),
            },
            Self::NanoParsed => match c {
                'S' | 's' => Ok(Self::NanosecondParsed),
                _ => Err(()),
            },
        }
    }
}

/// Parse number with duration units
///
/// - Number with duration unit (case insensitive)
///     - `\d+(Y|MO|W|D|H|M|S|MS|US|\u{00B5}S|NS)(\d+(Y|MO|W|D|H|M|S|MS|US|\u{00B5}S|NS))*`
pub struct DurationUnitParser {
    state: StateMachine<DurationUnitParseState>,
}

impl DurationUnitParser {
    /// Creates parser with the initial state
    pub fn new() -> Self {
        Self {
            state: StateMachine::new(DurationUnitParseState::Initial),
        }
    }

    /// Returns true if given char is accepted by the current state, and advances the state.
    pub fn accept(&mut self, c: &char) -> bool {
        self.state.accept(c)
    }

    pub fn is_valid(&self) -> bool {
        self.state.is_final()
    }
}

#[derive(PartialEq)]
enum Iso8601ParseState {
    Initial,
    ParseStart,
    ParseYear,
    YearParsed,
    ParseMonth,
    MonthParsed,
    WeekParsed,
    ParseDay,
    DayParsed,
    ParseTime,
    ParseHour,
    HourParsed,
    ParseMinute,
    MinuteParsed,
    ParseSecond,
    SecondParsed,
}

impl StateTransition for Iso8601ParseState {
    fn is_final(&self) -> bool {
        match self {
            Self::ParseStart
            | Self::ParseTime
            | Self::YearParsed
            | Self::MonthParsed
            | Self::DayParsed
            | Self::HourParsed
            | Self::MinuteParsed
            | Self::SecondParsed
            | Self::WeekParsed => true,
            _ => false,
        }
    }

    fn next_state(&self, c: &char) -> Result<Self, ()> {
        match self {
            Self::Initial => match c {
                'P' => Ok(Self::ParseStart),
                _ => Err(()),
            },
            Self::ParseStart => match c {
                '0'..='9' => Ok(Self::ParseYear),
                'T' => Ok(Self::ParseTime),
                _ => Err(()),
            },
            Self::ParseYear => match c {
                '0'..='9' => Ok(Self::ParseYear),
                'Y' => Ok(Self::YearParsed),
                'M' => Ok(Self::MonthParsed),
                'D' => Ok(Self::DayParsed),
                'W' => Ok(Self::WeekParsed),
                _ => Err(()),
            },
            Self::YearParsed => match c {
                '0'..='9' => Ok(Self::ParseMonth),
                'T' => Ok(Self::ParseTime),
                _ => Err(()),
            },
            Self::ParseMonth => match c {
                '0'..='9' => Ok(Self::ParseMonth),
                'M' => Ok(Self::MonthParsed),
                'D' => Ok(Self::DayParsed),
                _ => Err(()),
            },
            Self::MonthParsed => match c {
                '0'..='9' => Ok(Self::ParseDay),
                'T' => Ok(Self::ParseTime),
                _ => Err(()),
            },
            Self::ParseDay => match c {
                '0'..='9' => Ok(Self::ParseDay),
                'D' => Ok(Self::DayParsed),
                _ => Err(()),
            },
            Self::DayParsed => match c {
                'T' => Ok(Self::ParseTime),
                _ => Err(()),
            },
            Self::ParseTime => match c {
                '0'..='9' => Ok(Self::ParseHour),
                _ => Err(()),
            },
            Self::ParseHour => match c {
                '0'..='9' => Ok(Self::ParseHour),
                'H' => Ok(Self::HourParsed),
                'M' => Ok(Self::MinuteParsed),
                'S' => Ok(Self::SecondParsed),
                _ => Err(()),
            },
            Self::HourParsed => match c {
                '0'..='9' => Ok(Self::ParseMinute),
                _ => Err(()),
            },
            Self::ParseMinute => match c {
                '0'..='9' => Ok(Self::ParseMinute),
                'M' => Ok(Self::MinuteParsed),
                'S' => Ok(Self::SecondParsed),
                _ => Err(()),
            },
            Self::MinuteParsed => match c {
                '0'..='9' => Ok(Self::ParseSecond),
                _ => Err(()),
            },
            Self::ParseSecond => match c {
                '0'..='9' => Ok(Self::ParseSecond),
                'S' => Ok(Self::SecondParsed),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

/// Parse ISO8601 format
///
/// - ISO8601 format
pub struct Iso8601Parser {
    state: StateMachine<Iso8601ParseState>,
}

impl Iso8601Parser {
    /// Creates parser with the initial state
    pub fn new() -> Self {
        Self {
            state: StateMachine::new(Iso8601ParseState::Initial),
        }
    }

    /// Returns true if given char is accepted by the current state, and advances the state.
    pub fn accept(&mut self, c: &char) -> bool {
        self.state.accept(c)
    }

    pub fn is_valid(&self) -> bool {
        self.state.is_final()
    }
}

#[derive(PartialEq)]
enum Iso8601AlternativeParseState {
    Initial,
    ParseYear(u8),
    ParseMonth(u8),
    ParseDay(u8),
    ParseHour(u8),
    ParseMinute(u8),
    ParseSecond(u8),
    End,
}

impl StateTransition for Iso8601AlternativeParseState {
    fn is_final(&self) -> bool {
        *self == Self::End
    }

    fn next_state(&self, c: &char) -> Result<Self, ()> {
        match self {
            Self::Initial => match c {
                'P' => Ok(Self::ParseYear(0)),
                _ => Err(()),
            },
            Self::ParseYear(digits) => {
                if *digits < 4 && c.is_ascii_digit() {
                    Ok(Self::ParseYear(digits + 1))
                } else if *digits == 4 && *c == '-' {
                    Ok(Self::ParseMonth(0))
                } else {
                    Err(())
                }
            }
            Self::ParseMonth(digits) => {
                if *digits < 2 && c.is_ascii_digit() {
                    Ok(Self::ParseMonth(digits + 1))
                } else if *digits == 2 && *c == '-' {
                    Ok(Self::ParseDay(0))
                } else {
                    Err(())
                }
            }
            Self::ParseDay(digits) => {
                if *digits < 2 && c.is_ascii_digit() {
                    Ok(Self::ParseDay(digits + 1))
                } else if *digits == 2 && *c == 'T' {
                    Ok(Self::ParseHour(0))
                } else {
                    Err(())
                }
            }
            Self::ParseHour(digits) => {
                if *digits < 2 && c.is_ascii_digit() {
                    Ok(Self::ParseHour(digits + 1))
                } else if *digits == 2 && *c == ':' {
                    Ok(Self::ParseMinute(0))
                } else {
                    Err(())
                }
            }
            Self::ParseMinute(digits) => {
                if *digits < 2 && c.is_ascii_digit() {
                    Ok(Self::ParseMinute(digits + 1))
                } else if *digits == 2 && *c == ':' {
                    Ok(Self::ParseSecond(0))
                } else {
                    Err(())
                }
            }
            Self::ParseSecond(digits) => {
                if *digits < 1 && c.is_ascii_digit() {
                    Ok(Self::ParseSecond(digits + 1))
                } else if *digits == 1 && c.is_ascii_digit() {
                    Ok(Self::End)
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

/// Parse ISO8601 Alternative format
///
/// - ISO8601 Alternative format
///     - `P\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}`
pub struct Iso8601AlternativeParser {
    state: StateMachine<Iso8601AlternativeParseState>,
}

impl Iso8601AlternativeParser {
    /// Creates parser with the initial state
    pub fn new() -> Self {
        Self {
            state: StateMachine::new(Iso8601AlternativeParseState::Initial),
        }
    }

    /// Returns true if given char is accepted by the current state, and advances the state.
    pub fn accept(&mut self, c: &char) -> bool {
        self.state.accept(c)
    }

    pub fn is_valid(&self) -> bool {
        self.state.is_final()
    }
}
