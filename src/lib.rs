/// Library to help sort out a few things

use std::{
    fmt,
};

// Useful constants
const MILLIS_PER_SECOND: usize = 1000;
const MILLIS_PER_MINUTE: usize = 60 * MILLIS_PER_SECOND;
const MILLIS_PER_HOUR: usize = 60 * MILLIS_PER_MINUTE;

/// Simple time object to store hours, minutes, seconds, and milliseconds.
///
/// # Requirements
/// Minutes must be in the range 0-60 inclusive, seconds in the range 0-60
/// inclusive, and milliseconds in the range 0-999 inclusive.  There is no
/// support for sub-millisecond resolution.  It is recommended to use the
/// offset function to subtract or add times.
///
/// # Examples
/// Convert from hours, minutes, seconds, milliseconds to SimpleTime
/// ```
/// use offset_caption::SimpleTime;
///
/// let t = SimpleTime::from_parts(1, 2, 3, 4);
/// assert_eq!(t.hour(), 1);
/// assert_eq!(t.minute(), 2);
/// assert_eq!(t.second(), 3);
/// assert_eq!(t.millisecond(), 4);
/// ```
///
/// Convert from milliseconds to a SimpleTime
/// ```
/// use offset_caption::SimpleTime;
///
/// let t = SimpleTime::from_milliseconds(47_703_450);
/// assert_eq!(t.hour(), 13);
/// assert_eq!(t.minute(), 15);
/// assert_eq!(t.second(), 3);
/// assert_eq!(t.millisecond(), 450);
/// ```
///
/// Add one second to the simple time
/// ```
/// use offset_caption::SimpleTime;
/// let mut t = SimpleTime::from_parts(0, 0, 0, 0);
/// t.offset(1000).expect("We should be fine");
/// assert_eq!(t.hour(), 0);
/// assert_eq!(t.minute(), 0);
/// assert_eq!(t.second(), 1);
/// assert_eq!(t.millisecond(), 0);
/// ```
pub struct SimpleTime {
    hours: usize,
    minutes: usize,
    seconds: usize,
    milliseconds: usize,
}

impl SimpleTime {
    /// Create a SimpleTime from hours, minutes, seconds, milliseconds
    /// This will panic if invalid values are submitted (see documentation for SimpleTime).
    pub fn from_parts(
    hours: usize, minutes: usize, seconds: usize, milliseconds: usize) -> SimpleTime {
        if minutes >= 60 {
            panic!("SimpleTime requires minutes be in [0, 60] (got {})", minutes);
        }
        if seconds >= 60 {
            panic!("SimpleTime requires seconds be in [0, 60] (got {})", seconds);
        }
        if milliseconds >= 999 {
            panic!("SimpleTime requires milliseconds be in [0, 999] (got {})", milliseconds);
        }

        SimpleTime {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }

    /// Create a SimpleTime from milliseconds of time
    pub fn from_milliseconds(m: usize) -> SimpleTime {
        // Do conversions for units of second and larger
        let mut t = m;
        let hours = t / MILLIS_PER_HOUR;
        t -= hours * MILLIS_PER_HOUR;
        let minutes = t / MILLIS_PER_MINUTE;
        t -= minutes * MILLIS_PER_MINUTE;
        let seconds = t / MILLIS_PER_SECOND;
        t -= seconds * MILLIS_PER_SECOND;
        let milliseconds = t;
    
        SimpleTime {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }
    /// Create a float time from a SimpleTime
    pub fn to_milliseconds(&self) -> usize {
        self.hours * MILLIS_PER_HOUR
            + self.minutes * MILLIS_PER_MINUTE
            + self.seconds * MILLIS_PER_SECOND
            + self.milliseconds
    }
    /// Get hours
    pub fn hour(&self) -> usize { self.hours }
    /// Get minutes
    pub fn minute(&self) -> usize { self.minutes }
    /// Get seconds
    pub fn second(&self) -> usize { self.seconds }
    /// Get milliseconds
    pub fn millisecond(&self) -> usize { self.milliseconds }
    /// Offset this timestamp by milliseconds
    pub fn offset(&mut self, offset: isize) 
    -> Result<(), NegativeSimpleTime> {
        // Note: upcast to 128 in case large number; should be rare case
        let new_millis: i128 = self.to_milliseconds() as i128 + offset as i128;
        if new_millis < 0 {
            return Err(NegativeSimpleTime)
        }
        else {
            *self = SimpleTime::from_milliseconds(new_millis as usize);
            return Ok(())
        }
    }
}

/// Error type for trying to make a negative SimpleTime
#[derive(Debug, Clone)]
pub struct NegativeSimpleTime;

impl fmt::Display for NegativeSimpleTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "attempted to create negative SimpleTime")
    }
}

/// Type for parsing VTT caption files.
/// This parser assumes a format of:
/// - Header
/// - Blocks of caption with
///   - Blank Line
///   - Line 1: Block Number
///   - Line 2: Speaker: HH:MM:SS.mmm --> HH:MM:SS.mmm
///     - NOTE: Speaker is optional
///   - Line 3: Text to display for the caption
/// and will return a Caption object when asked to parse.
pub struct VttParser;

impl VttParser {
    /// Parse a string slice into a block number
    fn block_number(s: &str) -> Result<usize, VttParserError> {
        let r = s.parse::<usize>();
        match r {
            Ok(n) => Ok(n),
            Err(_) => Err(VttParserError::ExpectedBlockNumber(String::from(s))),
        }
    }
    /// Parse a VTT timestamp
    fn block_timestamp(s: &str) -> Result<SimpleTime, VttParserError> {
        let VTT_TIMESTAMP_LEN: usize = 12;
        if s.len() != VTT_TIMESTAMP_LEN {
            return Err(VttParserError::InvalidTimestamp(String::from(s)));
        }
        // We have correct length, parse
        // Get hours
        let hours = match s[0..2].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                return Err(VttParserError::InvalidTimestamp(String::from(s)));
            },
        };
        // Check first colon
        if s.chars().nth(2).unwrap() != ':' {
            return Err(VttParserError::InvalidTimestamp(
                    String::from(s)));
        }
        // Get minutes
        let minutes = match s[3..5].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                return Err(VttParserError::InvalidTimestamp(String::from(s)));
            },
        };
        // Check second colon
        if s.chars().nth(2).unwrap() != ':' {
            return Err(VttParserError::InvalidTimestamp(
                    String::from(s)));
        }
        // Get seconds
        let seconds = match s[6..8].parse::<usize>() {
            Ok(n) => {
                println!("{}", n);
                n
            },
            Err(_) => {
                return Err(VttParserError::InvalidTimestamp(String::from(s)));
            },
        };
        // Check period
        if s.chars().nth(8).unwrap() != '.' {
             return Err(VttParserError::InvalidTimestamp(
                    String::from(s)));
        }
        // Get milliseconds
        let milliseconds = match s[9..12].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                return Err(VttParserError::InvalidTimestamp(String::from(s)));
            },
        };

        Ok(
            SimpleTime::from_parts(
                hours,
                minutes,
                seconds,
                milliseconds
            )
        )
    }
    /// Parse a string slice into a tuple of block header information
    fn block_header(s: &str) -> Result<(Option<String>, SimpleTime, SimpleTime), VttParserError> {
        // See if we have a line to begin with
        if s.len() == 0 {
            return Err(VttParserError::UnexpectedEndOfFile);
        }
        if s.chars().nth(0).unwrap().is_numeric() {
            // Pass entire string to have timestamps parsed
            let (start, end) = VttParser::block_header_timestamps(s)?;
            return Ok((None, start, end));
        } else {
            // TODO: add name checking
            panic!("In progress");
        }
    }
    /// Parse the remainder of a line for start, end timestamps
    fn block_header_timestamps(s: &str) -> Result<(SimpleTime, SimpleTime), VttParserError> {
        // Make sure we have exactly three "words"
        let total_words = s.split(' ').count();
        if total_words == 3 {
            // We're good to go, probably
            let first = s.split(' ').nth(0);
            let second = s.split(' ').nth(1);
            let third = s.split(' ').nth(2);
            if let Some(ts1) = first {
                if let Some("-->") = second {
                    if let Some(ts2) = third {
                        // Need to process the timestamps
                        let start = VttParser::block_timestamp(ts1)?;
                        let end = VttParser::block_timestamp(ts2)?;
                        return Ok((start, end));

                    } else {
                        return Err(
                            VttParserError::InvalidTimestamp(
                                String::from(s)));
                    }
                } else {
                    return Err(
                        VttParserError::InvalidTimestamp(
                            String::from(s)));
                }
            } else {
                return Err(VttParserError::InvalidTimestamp(
                    String::from(s)));
            }
        } else {
            return Err(
                VttParserError::InvalidTimestamp(String::from(s)));
        }
    }

}

/// Error type for VttParser
#[derive(Debug, Clone)]
pub enum VttParserError {
    UnexpectedEndOfFile,
    ExpectedBlockNumber(String),
    BlockHeaderInvalid(String),
    InvalidTimestamp(String),
}

impl fmt::Display for VttParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VttParserError::UnexpectedEndOfFile => write!(f, "unexpected end of file"),
            VttParserError::ExpectedBlockNumber(s) => {
                write!(f, "expected VTT block number, got {}", s)
            },
            VttParserError::BlockHeaderInvalid(s) => {
                write!(f, "invalid VTT block from line {}", s)
            },
            VttParserError::InvalidTimestamp(s) => {
                write!(f, "invalid VTT block from word {}", s)
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod simple_time {
        #[test]
        fn test_to_from_millis_works() {
            let st = super::SimpleTime::from_parts(23, 54, 17, 837);
            assert_eq!(st.to_milliseconds(), 86057837);
            let st2 = super::SimpleTime::from_milliseconds(86897);
            assert_eq!(st2.to_milliseconds(), 86897);
        }
        #[test]
        fn test_offset() {
            const MILLS: isize = 123456;
            let mut st = super::SimpleTime::from_parts(0, 0, 0, 0);
            st.offset(MILLS).expect("Failed offset");
            assert_eq!(st.to_milliseconds(), 123456);
        }
        #[test]
        fn test_offset_negative_time() {
            const MILLS: isize = -123;
            let mut st = super::SimpleTime::from_milliseconds(0);
            let r = st.offset(MILLS);
            match r {
                Ok(()) => panic!("Test failure; was okay going negative"),
                Err(_) => assert_eq!(0, 0),
            };
        }
    }
    mod vtt_parser {
        use super::{VttParser, VttParserError};
        #[test]
        fn test_parse_block_no() {
            let n = VttParser::block_number("1").expect("");
            assert_eq!(n, 1);

            let n = VttParser::block_number("a");
            match n {
                Ok(_) => panic!("Test failure! VttParser parses 'a'"),
                Err(e) => {
                    match e {
                        VttParserError::UnexpectedEndOfFile => {
                            panic!("Test failure! VttParser wrong err");
                        },
                        VttParserError::ExpectedBlockNumber(s) => {
                            assert_eq!(s, "a");
                        },
                        _ => panic!("Unknown test failure")
                    };
                },
            };
        }
        #[test]
        fn test_parse_block_header_no_name() {
            // Test with no speaker listed
            let test_str_1 = "00:00:00.000 --> 00:00:01.001";
            let r = VttParser::block_header(test_str_1);
            match r {
                Ok((None, start, end)) => {
                    assert_eq!(start.to_milliseconds(), 0);
                    assert_eq!(end.to_milliseconds(), 1001);
                }
                _ => panic!("Test failed"),
            }
        }
        #[test]
        fn test_parse_block_header_with_name() {
            // Test with speaker listed
            let test_str_2 = "Pete Molfese 00:00:00.000 --> 00:00:01.001";
            let r = VttParser::block_header(test_str_2);
            match r {
                Ok((Some(s), start, end)) => {
                    assert_eq!(s, "Pete Molfese");
                    assert_eq!(start.to_milliseconds(), 0);
                    assert_eq!(end.to_milliseconds(), 1001);
                }
                _ => panic!("Test failed when given name"),
            }
        }
        #[test]
        fn test_parse_block_header_missing_start() {
            // Test that we fail for no block start
            let test_str_3 = "--> 00:00:01.001";
            let r = VttParser::block_header(test_str_3);
            match r {
                Ok(_) => panic!("Things should not be okay"),
                Err(e) => {
                    match e {
                        VttParserError::BlockHeaderInvalid(s) => {},
                        _ => panic!("Test failed in unexpected way"),
                    };
                },
            };
        }
    }
}
