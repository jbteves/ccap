/// Library to help sort out a few things

use std::{fmt, fs, error::Error, path::Path, ffi::OsStr};

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
/// use ccap::SimpleTime;
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
/// use ccap::SimpleTime;
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
/// use ccap::SimpleTime;
/// let mut t = SimpleTime::from_parts(0, 0, 0, 0);
/// t.offset(1000).expect("We should be fine");
/// assert_eq!(t.hour(), 0);
/// assert_eq!(t.minute(), 0);
/// assert_eq!(t.second(), 1);
/// assert_eq!(t.millisecond(), 0);
/// ```
#[derive(Debug, Clone, PartialEq)]
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

impl Error for NegativeSimpleTime {}

impl fmt::Display for NegativeSimpleTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "attempted to create negative SimpleTime")
    }
}

/// General parser for any caption file
pub fn parse_file(fname: &str) -> Result<Caption, Box<dyn Error>> {
    match Path::new(&fname).extension().and_then(OsStr::to_str) {
        Some(ext) => {
            match ext {
                "vtt" | "txt" => Ok(VttParser::from_file(fname)?),
                "srt" => Ok(SrtParser::from_file(fname)?),
                _ => Err(CaptionParserError::UnsupportedFileType(ext.to_string()))?,
            }
        }
        None => Err(CaptionParserError::UnknownExtension(fname.to_string()))?,
    }
}

/// General writer for any caption file
pub fn write_caption(fname: &str, caption: &Caption) -> Result<(), Box<dyn Error>> {
    match Path::new(&fname).extension().and_then(OsStr::to_str) {
        Some(ext) => {
            match ext {
                "vtt" | "txt" => VttWriter::to_file(&fname, &caption)?,
                "srt" => SrtWriter::to_file(&fname, &caption)?,
                _ => Err(CaptionParserError::UnsupportedFileType(fname.to_string()))?,
            }
        },
        _ => {
            Err(CaptionParserError::UnknownExtension(fname.to_string()))?
        },
    }
    Ok(())
}

/// Error for parser
#[derive(Debug, Clone)]
pub enum CaptionParserError {
    UnsupportedFileType(String),
    UnknownExtension(String)
}

impl Error for CaptionParserError {}
impl fmt::Display for CaptionParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CaptionParserError::UnsupportedFileType(s) => write!(f, "type {} is unsupported", s),
            CaptionParserError::UnknownExtension(s) => write!(f, "unknown extension for file {}", s),
        }
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
    /// Parse File into a Caption
    pub fn from_file(fname: &str) -> Result<Caption, Box<dyn Error>> {
        let s = fs::read_to_string(fname)?;
        let cap = VttParser::parse(&s)?;
        Ok(cap)
    }
    /// Parse a Caption
    pub fn parse(contents: &str) -> Result<Caption, VttParserError> {
        // First, find the header
        let (header, vtt_line) = VttParser::header(&contents)?;
        let start_line = vtt_line + 1;
        // Get the length of the file in lines, to check blocking
        let total_lines = contents.lines().count();
        // Figure out if the total number of lines remaining is going to break into even blocks
        let blocks_remaining = (total_lines - start_line) / 4;
        if (blocks_remaining as f32) != ((total_lines as f32 - start_line as f32) / 4.0) {
            return Err(VttParserError::UnexpectedEndOfFile)?;
        }
        // We have the right number of blocks.
        // Vector for storing CaptionBlock items
        let mut blocks: Vec<CaptionBlock> = Vec::with_capacity(blocks_remaining);
        // Skip lines we've already seen
        let mut line_iter = contents.lines();
        for _ in 0..(start_line) {
            line_iter.next();
        }
        // Create a vector of all remaining lines
        let lines: Vec<&str> = line_iter.collect();
        // Iterate and process blocks
        for i in 0..blocks_remaining {
            let block_line_start = i * 4;
            let block_line_end = (i * 4) + 3;
            let current_block = lines[block_line_start..(block_line_end + 1)]
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            blocks.push(VttParser::block(&current_block)?);
        }
        // We're all good, pass along the caption object
        Ok(
            Caption {
                header,
                blocks
            }
        )
    }
    /// Parse the header
    fn header(s: &str) -> Result<(Option<String>, usize), VttParserError> {
        // Find "WEBVTT"
        let is_webvtt = |x| x == "WEBVTT";

        if let Some(n) = s.lines().position(is_webvtt) {
            let header_opt = match n {
                0 | 1 | 2 => None,
                nn => {
                    // Fetch all the lines preceding N -2
                    let header = s.lines().take(nn - 2)
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");
                    Some(header)
                },
            };
            Ok((header_opt, n))
        }
        else {
            Err(VttParserError::UnexpectedEndOfFile)
        }
    }
    /// Parse a block
    fn block(s: &str) -> Result<CaptionBlock, VttParserError> {
        // Make sure we have exactly four lines to parse
        if s.lines().count() != 4 {
            return Err(VttParserError::UnexpectedEndOfFile);
        }

        // Make an iterator and view line by line
        let mut s_iter = s.lines();
        match s_iter.next() {
            Some("") => {},
            Some(s) => {
                return Err(VttParserError::ExpectedBlankLine(s.to_string()));
            },
            _ => { return Err(VttParserError::UnexpectedEndOfFile) },
        }
        let block_line = s_iter.next().ok_or(VttParserError::UnexpectedEndOfFile)?;
        let _ = VttParser::block_number(block_line)?;
        let header_line = s_iter.next().ok_or(VttParserError::UnexpectedEndOfFile)?;
        let (speaker, start, end) = VttParser::block_header(header_line)?;
        let text_line = s_iter.next().ok_or(VttParserError::UnexpectedEndOfFile)?;
        let text = VttParser::block_text(text_line);
        Ok(CaptionBlock {
            speaker,
            start,
            end,
            text,
        })
    }
    /// Parse a string slice into a block number
    fn block_number(s: &str) -> Result<usize, VttParserError> {
        let r = s.parse::<usize>();
        match r {
            Ok(n) => Ok(n),
            Err(_) => Err(VttParserError::ExpectedBlockNumber(String::from(s))),
        }
    }
    /// Parse a VTT timestamp
    pub fn block_timestamp(s: &str) -> Result<SimpleTime, VttParserError> {
        let vtt_timestamp_len: usize = 12;
        if s.len() != vtt_timestamp_len {
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
            // Find first timestamp
            let first_loc = match s.find(char::is_numeric) {
                Some(n) => n,
                None => Err(VttParserError::BlockHeaderInvalid(
                        String::from(s)))?,
            };
            // Make sure we have a space before
            match s.get(first_loc - 1..first_loc) {
                Some(" ") => {},
                _ => {
                    return Err(VttParserError::BlockHeaderInvalid(String::from(s)));
                },
            };
            // Find the name, which is everything preceding the space
            let name = match s.get(..first_loc - 1) {
                Some(x) => x,
                _ => {
                    return Err(VttParserError::BlockHeaderInvalid(
                            String::from(s)));
                },
            };
            let (start, end) = VttParser::block_header_timestamps(
                match s.get(first_loc..) {
                    Some(s) => s,
                    None => {
                        return Err(VttParserError::BlockHeaderInvalid(
                            String::from(s)));
                    },
                }
            )?;
            return Ok((Some(name.to_string()), start, end));
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
    /// Parse the text of a block; thin wrapper for to_string()
    fn block_text(s: &str) -> String {
        s.to_string()
    }
}

/// Error type for VttParser
#[derive(Debug, Clone)]
pub enum VttParserError {
    UnexpectedEndOfFile,
    FileNotReadable(String),
    ExpectedBlankLine(String),
    ExpectedBlockNumber(String),
    BlockHeaderInvalid(String),
    InvalidTimestamp(String),
}

impl fmt::Display for VttParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VttParserError::UnexpectedEndOfFile => write!(f, "unexpected end of file"),
            VttParserError::FileNotReadable(s) => {
                write!(f, "could not read file {}", s)
            },
            VttParserError::ExpectedBlankLine(s) => {
                write!(f, "expected blank line, got {}", s)
            },
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

impl Error for VttParserError {}

/// Associated Functions for VTT writing
pub struct VttWriter;

impl VttWriter {
    /// Write a full VTT file to disk
    pub fn to_file(fname: &str, cap: &Caption) -> Result<(), Box<dyn Error>> {
        fs::write(fname, VttWriter::write(&cap))?;
        Ok(())
    }
    /// Write a full VTT file to a string
    pub fn write(cap: &Caption) -> String {
        let mut components: Vec<String> = Vec::with_capacity(cap.blocks.len() + 1);
        components.push(VttWriter::header(&cap));
        let mut block_num = 1;

        for block in cap.blocks.iter() {
            components.push(VttWriter::block(block, block_num));
            block_num += 1;
        }
        components.join("\n")
    }
    /// Write a VTT block
    fn block(cb: &CaptionBlock, n: usize) -> String {
        let ts_start = VttWriter::timestamp(&cb.start);
        let ts_end = VttWriter::timestamp(&cb.end);

        format!(
            "{}\n{}\n{}\n",
            n,
            match &cb.speaker {
                Some(person) => format!("{} {} --> {}", person, ts_start, ts_end),
                None => format!("{} --> {}", ts_start, ts_end),
            },
            cb.text
        )
    }
    /// Write a VTT timestamp
    fn timestamp(t: &SimpleTime) -> String {
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            t.hour(),
            t.minute(),
            t.second(),
            t.millisecond()
        )
    }
    /// Write a VTT header
    fn header(cap: &Caption) -> String {
        let webvtt = "WEBVTT\n";
        match &cap.header {
            Some(s) => format!("{}\n\n{}", s, webvtt),
            None => webvtt.to_string(),
        }
    }
}

/// Parser utilities for SRT files
/// This parser assumes a format of:
/// - Blocks of caption with
///   - Line 1: Block Number
///   - Line 2: HH:MM:SS.mmm --> HH:MM:SS.mmm
///   - Line 3: [Speaker] subtitle
///     - Note: Speaker is optional, and will be parsed if enclosed in brackets. If the speaker is
///       identified in some other way, then it will be displayed for other formats, but may not be
///       formatted as the speaker.
/// and will return a Caption object when asked to parse.
pub struct SrtParser;

impl SrtParser {
    /// Parse File into a Caption
    pub fn from_file(fname: &str) -> Result<Caption, Box<dyn Error>> {
        let s = fs::read_to_string(fname)?;
        let cap = SrtParser::parse(&s)?;
        Ok(cap)
    }
    /// Parse a Caption
    pub fn parse(contents: &str) -> Result<Caption, SrtParserError> {
        // Inject a newline for simplicity in processing
        let contents = &("\n".to_owned() + contents);
        let total_lines = contents.lines().count();
        // Figure out if the total number of lines remaining is going to break into even blocks
        let blocks_remaining = total_lines / 4;
        if (blocks_remaining as f32) != (total_lines as f32 ) / 4.0 {
            return Err(SrtParserError::UnexpectedEndOfFile)?;
        }
        // We have the right number of blocks.
        // Vector for storing CaptionBlock items
        let mut blocks: Vec<CaptionBlock> = Vec::with_capacity(blocks_remaining);
        // Create a vector of all remaining lines, prepending one blank line
        let lines: Vec<&str> = contents.lines().collect();
        // Iterate and process blocks
        for i in 0..blocks_remaining {
            let block_line_start = i * 4;
            let block_line_end = (i * 4) + 3;
            let current_block = lines[block_line_start..(block_line_end + 1)]
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            blocks.push(SrtParser::block(&current_block)?);
        }
        // We're all good, pass along the caption object
        Ok(
            Caption {
                header: None,
                blocks
            }
        )
    }
    /// Parse a block
    fn block(s: &str) -> Result<CaptionBlock, SrtParserError> {
        // Make sure we have exactly four lines to parse
        if s.lines().count() != 4 {
            return Err(SrtParserError::UnexpectedEndOfFile);
        }

        // Make an iterator and view line by line
        let mut s_iter = s.lines();
        match s_iter.next() {
            Some("") => {},
            Some(s) => {
                return Err(SrtParserError::ExpectedBlankLine(s.to_string()));
            },
            _ => { return Err(SrtParserError::UnexpectedEndOfFile) },
        }
        let block_line = s_iter.next().ok_or(SrtParserError::UnexpectedEndOfFile)?;
        let _ = SrtParser::block_number(block_line)?;
        let header_line = s_iter.next().ok_or(SrtParserError::UnexpectedEndOfFile)?;
        let (start, end) = SrtParser::block_timestamps(header_line)?;
        let text_line = s_iter.next().ok_or(SrtParserError::UnexpectedEndOfFile)?;
        let (speaker, text) = SrtParser::block_text(text_line)?;
        Ok(CaptionBlock {
            speaker,
            start,
            end,
            text,
        })
    }
    /// Parse a string slice into a block number
    fn block_number(s: &str) -> Result<usize, SrtParserError> {
        let r = s.parse::<usize>();
        match r {
            Ok(n) => Ok(n),
            Err(_) => Err(SrtParserError::ExpectedBlockNumber(String::from(s))),
        }
    }
    /// Parse an SRT timestamp
    pub fn block_timestamp(s: &str) -> Result<SimpleTime, SrtParserError> {
        let vtt_timestamp_len: usize = 12;
        if s.len() != vtt_timestamp_len {
            return Err(SrtParserError::InvalidTimestamp(String::from(s)));
        }
        // We have correct length, parse
        // Get hours
        let hours = match s[0..2].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                return Err(SrtParserError::InvalidTimestamp(String::from(s)));
            },
        };
        // Check first colon
        if s.chars().nth(2).unwrap() != ':' {
            return Err(SrtParserError::InvalidTimestamp(
                    String::from(s)));
        }
        // Get minutes
        let minutes = match s[3..5].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                return Err(SrtParserError::InvalidTimestamp(String::from(s)));
            },
        };
        // Check second colon
        if s.chars().nth(2).unwrap() != ':' {
            return Err(SrtParserError::InvalidTimestamp(
                    String::from(s)));
        }
        // Get seconds
        let seconds = match s[6..8].parse::<usize>() {
            Ok(n) => {
                n
            },
            Err(_) => {
                return Err(SrtParserError::InvalidTimestamp(String::from(s)));
            },
        };
        // Check comma
        if s.chars().nth(8).unwrap() != ',' {
             return Err(SrtParserError::InvalidTimestamp(
                    String::from(s)));
        }
        // Get milliseconds
        let milliseconds = match s[9..12].parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                return Err(SrtParserError::InvalidTimestamp(String::from(s)));
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
    /// Parse the remainder of a line for start, end timestamps
    fn block_timestamps(s: &str) -> Result<(SimpleTime, SimpleTime), SrtParserError> {
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
                        let start = SrtParser::block_timestamp(ts1)?;
                        let end = SrtParser::block_timestamp(ts2)?;
                        return Ok((start, end));

                    } else {
                        return Err(
                            SrtParserError::InvalidTimestamp(
                                String::from(s)));
                    }
                } else {
                    return Err(
                        SrtParserError::InvalidTimestamp(
                            String::from(s)));
                }
            } else {
                return Err(SrtParserError::InvalidTimestamp(
                    String::from(s)));
            }
        } else {
            return Err(
                SrtParserError::InvalidTimestamp(String::from(s)));
        }
    }
    /// Parse the text and optional speaker of a block
    fn block_text(s: &str) -> Result<(Option<String>, String), SrtParserError> {
        // See if we have a speaker
        if let Some(n0) = s.chars().position(|x| x == '[') {
            if n0 != 0 {
                return Err(SrtParserError::InvalidSpeakerPlacement(s.to_string()));
            }
            if let Some(n1) = s.chars().position(|x| x == ']') {
                // Valid Speaker
                let speaker = s.get((n0 + 1)..n1).unwrap().to_string();
                let text = s.get((n1 + 2)..).unwrap().to_string();
                return Ok((Some(speaker.to_string()), text.to_string()));
            }
            else {
                return Err(SrtParserError::InvalidSpeakerPlacement(s.to_string()));
            }
        }
        // No Speaker
        Ok((None, s.to_string()))
    }
}

/// Error type for SrtParser
#[derive(Debug, Clone)]
pub enum SrtParserError {
    UnexpectedEndOfFile,
    FileNotReadable(String),
    ExpectedBlankLine(String),
    ExpectedBlockNumber(String),
    BlockHeaderInvalid(String),
    InvalidTimestamp(String),
    InvalidSpeakerPlacement(String),
}

impl fmt::Display for SrtParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SrtParserError::UnexpectedEndOfFile => write!(f, "unexpected end of file"),
            SrtParserError::FileNotReadable(s) => {
                write!(f, "could not read file {}", s)
            },
            SrtParserError::ExpectedBlankLine(s) => {
                write!(f, "expected blank line, got {}", s)
            },
            SrtParserError::ExpectedBlockNumber(s) => {
                write!(f, "expected SRT block number, got {}", s)
            },
            SrtParserError::BlockHeaderInvalid(s) => {
                write!(f, "invalid SRT block from line {}", s)
            },
            SrtParserError::InvalidTimestamp(s) => {
                write!(f, "invalid SRT block from word {}", s)
            },
            SrtParserError::InvalidSpeakerPlacement(s) => {
                write!(f, "invalid SRT speaker placement in line {}", s)
            },
        }
    }
}

impl Error for SrtParserError {}


/// Writer utilities for SRT files
// TODO: add more speaker formatting options
pub struct SrtWriter;

impl SrtWriter {
    /// Write a full VTT file to disk
    pub fn to_file(fname: &str, cap: &Caption) -> Result<(), Box<dyn Error>> {
        fs::write(fname, SrtWriter::write(&cap))?;
        Ok(())
    }
    /// Write a full VTT file to a string
    pub fn write(cap: &Caption) -> String {
        let mut components: Vec<String> = Vec::with_capacity(cap.blocks.len());
        let mut block_num = 1;

        for block in cap.blocks.iter() {
            components.push(SrtWriter::block(block, block_num));
            block_num += 1;
        }
        components.join("\n")
    }
    /// Write a VTT block
    fn block(cb: &CaptionBlock, n: usize) -> String {
        let ts_start = SrtWriter::timestamp(&cb.start);
        let ts_end = SrtWriter::timestamp(&cb.end);

        format!(
            "{}\n{}\n{}\n",
            n,
            format!("{} --> {}", ts_start, ts_end),
            match &cb.speaker {
                Some(person) => format!("[{}] {}", person, cb.text),
                None => format!("{}", cb.text),
            },
        )
    }
    /// Write a VTT timestamp
    fn timestamp(t: &SimpleTime) -> String {
        format!(
            "{:02}:{:02}:{:02},{:03}",
            t.hour(),
            t.minute(),
            t.second(),
            t.millisecond()
        )
    }
}

        

/// Caption blocks contain an optional speaker, start and end times, and the text that will be
/// displayed on the screen during the block.
/// Not particularly useful on their own.
///
/// # Examples
/// Create a CaptionBlock with no speaker, from 0 seconds to 1 second, and a text of "Hello!"
/// ```
/// use ccap::{CaptionBlock, SimpleTime};
///
/// let block = CaptionBlock::from(
///     None,
///     SimpleTime::from_milliseconds(0),
///     SimpleTime::from_milliseconds(1000),
///     String::from("Hello!")).unwrap();
/// assert_eq!(block.speaker(), None);
/// assert_eq!(block.start().second(), 0);
/// assert_eq!(block.end().second(), 1);
/// assert_eq!(block.text(), "Hello!");
/// ```
#[derive(Debug, Clone)]
pub struct CaptionBlock {
    speaker: Option<String>,
    start: SimpleTime,
    end: SimpleTime,
    text: String,
}

impl CaptionBlock {
    /// Construct a CaptionBlock from its parts
    pub fn from(speaker: Option<String>, start: SimpleTime, end: SimpleTime, text: String) -> Result<CaptionBlock, CaptionBlockError> {
        // Verify start is less than end
        let diff = (end.to_milliseconds() as i128) - (start.to_milliseconds() as i128);
        if diff < 0 {
            Err(CaptionBlockError::EndsBeforeStart(start, end))
        }
        else {
            Ok(
                CaptionBlock {
                    speaker,
                    start,
                    end,
                    text,
                }
            )
        }
    }
    /// Get a copy of this block's text
    pub fn text(&self) -> String {
        self.text.clone()
    }
    /// Get a copy of this block's speaker
    pub fn speaker(&self) -> Option<String> {
        self.speaker.clone()
    }
    /// Get a copy of this caption block's start time
    pub fn start(&self) -> SimpleTime {
        self.start.clone()
    }
    /// Get a copy of this caption block's end time
    pub fn end(&self) -> SimpleTime {
        self.end.clone()
    }
    /// Offset the times in this caption block
    pub fn offset_milliseconds(&mut self, n: isize) -> Result<(), NegativeSimpleTime> {
        self.start.offset(n)?;
        self.end.offset(n)?;
        Ok(())
    }
}

/// Error types for CaptionBlock
#[derive(Debug)]
pub enum CaptionBlockError {
    EndsBeforeStart(SimpleTime, SimpleTime)
}

/// Type for storing captions
///
/// They are generated by parsers, and can be used by generators to write out new caption files.
#[derive(Debug)]
pub struct Caption {
    pub header: Option<String>,
    pub blocks: Vec<CaptionBlock>,
}

impl Caption {
    /// Construct a Caption from its components
    pub fn from(header_str: Option<&str>, blocks: Vec<CaptionBlock>) -> Caption {
        Caption {
            header: match header_str {
                Some(s) => Some(s.to_string()),
                None => None,
            },
            blocks: blocks,
        }
    }
    /// Offset all of the timestamps in this caption
    pub fn offset_milliseconds(&mut self, n: isize) -> Result<(), NegativeSimpleTime> {
        for b in self.blocks.iter_mut() {
            b.start.offset(n)?;
            b.end.offset(n)?;
        }
        Ok(())
    }
    /// Get the first time in milliseconds from a caption
    pub fn time_head(&self) -> usize {
        self.blocks[0].start.to_milliseconds()
    }
    /// Get the last time in milliseconds from a caption
    pub fn time_tail(&self) -> usize {
        self.blocks.iter().last().unwrap().end.to_milliseconds()
    }
    /// Concatenate captions and generate new one; does not retain header information.
    /// This is because concatenating headers doesn't necessarily make sense.
    pub fn concatenate(captions: Vec<Caption>) -> Caption {
        let total_blocks = captions.iter()
            .map(|c| c.blocks.iter().count())
            .sum();
        let mut cb: Vec<CaptionBlock> = Vec::with_capacity(total_blocks);
        let mut last_ts: usize = 0;
        for cap in captions.iter() {
            for b in cap.blocks.iter() {
                let mut copy_b = b.clone();
                // Add on the time from the previous caption
                copy_b.offset_milliseconds(last_ts as isize)
                    .expect("Something logically impossible has occured");
                cb.push(copy_b);
            }
            // Update the last time
            last_ts += cap.time_tail();
        }
        Caption {
            header: None,
            blocks: cb,
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
    mod caption {
        use super::*;
        #[test]
        fn offset_caption() {
            let mut c = Caption {
                header: None,
                blocks: vec!(CaptionBlock {
                    speaker: None,
                    start: SimpleTime::from_milliseconds(0),
                    end: SimpleTime::from_milliseconds(1000),
                    text: "John Dies at the End".to_string(),
                })
            };
            c.offset_milliseconds(500).expect("Should be fine");
            assert_eq!(c.blocks[0].start.to_milliseconds(), 500);
            assert_eq!(c.blocks[0].end.to_milliseconds(), 1500);
        }
        #[test]
        fn concatenate_captions() {
            let c1 = Caption {
                header: None,
                blocks: vec!(CaptionBlock {
                    speaker: None,
                    start: SimpleTime::from_milliseconds(0),
                    end: SimpleTime::from_milliseconds(1000),
                    text: "John Dies at the End".to_string(),
                })
            };
            let c2 = Caption {
                header: None,
                blocks: vec!(CaptionBlock {
                    speaker: None,
                    start: SimpleTime::from_milliseconds(0),
                    end: SimpleTime::from_milliseconds(1000),
                    text: "John is dead now".to_string(),
                })
            };
            let c3 = Caption {
                header: None,
                blocks: vec!(
                    CaptionBlock {
                        speaker: None,
                        start: SimpleTime::from_milliseconds(0),
                        end: SimpleTime::from_milliseconds(1000),
                        text: "Go read the book!".to_string(),
                    },
                    CaptionBlock {
                        speaker: None,
                        start: SimpleTime::from_milliseconds(1200),
                        end: SimpleTime::from_milliseconds(2400),
                        text: "Seriously.".to_string(),
                    },
                ),
            };
            let c = Caption::concatenate(vec!(c1, c2, c3));
            assert_eq!(c.blocks[0].start.to_milliseconds(), 0);
            assert_eq!(c.blocks[0].end.to_milliseconds(), 1000);
            assert_eq!(c.blocks[1].start.to_milliseconds(), 1000);
            assert_eq!(c.blocks[1].end.to_milliseconds(), 2000);
            assert_eq!(c.blocks[2].start.to_milliseconds(), 2000);
            assert_eq!(c.blocks[2].end.to_milliseconds(), 3000);
            assert_eq!(c.blocks[3].start.to_milliseconds(), 3200);
            assert_eq!(c.blocks[3].end.to_milliseconds(), 4400);

        }
    }
    mod vtt_writer {
        use super::*;
        #[test]
        fn write() {
            let cap = Caption {
                header: None,
                blocks: vec!(
                    CaptionBlock {
                        speaker: Some("Pete Molfese".to_string()),
                        start: SimpleTime::from_milliseconds(0),
                        end: SimpleTime::from_milliseconds(1000),
                        text: "Hello, world!".to_string(),
                    }
                )
            };
            let should_get = format!(
                "WEBVTT\n\n{}\n{} {} --> {}\n{}\n",
                1,
                "Pete Molfese",
                "00:00:00.000",
                "00:00:01.000",
                "Hello, world!"
            );
            assert_eq!(VttWriter::write(&cap), should_get);
        }
        #[test]
        fn write_with_header() {
            let cap = Caption {
                header: Some("This is a VERY cool test".to_string()),
                blocks: vec!(
                    CaptionBlock {
                        speaker: None,
                        start: SimpleTime::from_milliseconds(0),
                        end: SimpleTime::from_milliseconds(1500),
                        text: "We're doing very cool things".to_string(),
                    },
                    CaptionBlock {
                        speaker: None,
                        start: SimpleTime::from_milliseconds(1500),
                        end: SimpleTime::from_milliseconds(2500),
                        text: "The COOLEST things!".to_string(),
                    }
                ),
            };
            let should_get = format!(
                "{}\n\nWEBVTT\n\n{}\n{} --> {}\n{}\n\n{}\n{} --> {}\n{}\n",
                "This is a VERY cool test",
                1,
                "00:00:00.000",
                "00:00:01.500",
                "We're doing very cool things",
                2,
                "00:00:01.500",
                "00:00:02.500",
                "The COOLEST things!"
            );
            assert_eq!(VttWriter::write(&cap), should_get);
        }


    }
    mod vtt_parser {
        use super::*;
        #[test]
        fn parse() {
            let block = format!(
                "\n{}\n{}\n{}\n",
                1,
                "Pete Molfese 00:00:00.000 --> 00:00:01.000",
                "Hello, welcome to the caption tool!"
            );
            let s = format!("WEBVTT\n{}", block);
            let cap = VttParser::parse(&s)
                .expect("Should have passed!");
            assert_eq!(cap.header, None);
            let expected_block = CaptionBlock::from(
                Some("Pete Molfese".to_string()),
                SimpleTime::from_milliseconds(0),
                SimpleTime::from_milliseconds(1000),
                "Hello, welcome to the caption tool!".to_string()
            ).unwrap();
            assert_eq!(cap.blocks.len(), 1);
            let received_block = &cap.blocks[0];
            assert_eq!(expected_block.speaker, received_block.speaker);
            assert_eq!(expected_block.start, received_block.start);
            assert_eq!(expected_block.end, received_block.end);
            assert_eq!(expected_block.text, received_block.text);
        }
        #[test]
        fn parse_header() {
            let h1 = "This is an event!";
            let h2 = "Loads of cool presenters, all fabulous!";
            let fake_block = format!("\n{}\n{}\n{}\n", 1, " ", " ");
            let s = format!("{}\n{}\n\n\nWEBVTT\n{}", h1, h2, fake_block);
            let (header, line_number) = VttParser::header(&s)
                .expect("Should not have failed to parse!");
            assert_eq!(header, Some(format!("{}\n{}", h1, h2)));
            assert_eq!(line_number, 4);
        }
        #[test]
        fn parse_no_header() {
            let fake_block = format!("\n{}\n{}\n{}\n", 1, " ", " ");
            let s = format!("WEBVTT\n{}", fake_block);
            let (header, line_number) = VttParser::header(&s)
                 .expect("Should not have failed to parse!");
            assert_eq!(header, None);
            assert_eq!(line_number, 0);
        }
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
                },
                Ok((None, _start, _end)) => {
                    panic!("Did not parse out any names");
                },
                Err(e) => {
                    panic!("Test failed with error {:?}", e );
                },
            }
        }
        #[test]
        fn test_parse_block_header_missing_start() {
            // Test that we fail for no block start
            let test_str_3 = "--> 00:00:01.001";
            let r = VttParser::block_header(test_str_3);
            match r {
                Ok((name, start, end)) => {
                    panic!("Parsed {:?}, {:?}, {:?} when should have failed", name, start, end);
                },
                Err(e) => {
                    match e {
                        VttParserError::InvalidTimestamp(_s) => {},
                        _ => panic!("Test failed in unexpected way"),
                    };
                },
            };
        }
        #[test]
        fn test_parse_block_text() {
            // Test to make sure we parse a line of text
            let test_str = "The quick brown fox jumps over the lazy dog.";
            let text = VttParser::block_text(test_str);
            assert_eq!(text, test_str.to_string());
        }
        #[test]
        fn test_parse_block() {
            // Test to make sure we parse an entire block
            let start = "00:00:00.000";
            let end = "00:00:01.000";
            let text = "The quick brown fox jumps over the lazy dog";
            let test_input = format!("\n{}\n{} --> {}\n{}\n", 1, start, end, text);
            let cb = VttParser::block(&test_input)
                .expect("Failed test");
            assert_eq!(cb.start().to_milliseconds(), 0);
            assert_eq!(cb.end().to_milliseconds(), 1000);
            assert_eq!(cb.speaker(), None);
            assert_eq!(cb.text(), text);
        }
        #[test]
        fn test_parse_block_fails_insufficient_lines() {
            // Test to make sure we fail for no blank
            let x = VttParser::block("thing\n");
            match x {
                Err(VttParserError::UnexpectedEndOfFile) => {},
                _ => panic!("Didn't get unexpected EOF {:?}", x),
            };
        }
    }
    mod srt_parser {
        use super::*;
        #[test]
        fn parse() {
            let s = format!(
                "{}\n{}\n{}\n",
                1,
                "00:00:00,000 --> 00:00:01,000",
                "[Peter Molfese] Hello, welcome to the caption tool!"
            );
            let cap = SrtParser::parse(&s)
                .expect("Should have passed!");
            assert_eq!(cap.header, None);
            let expected_block = CaptionBlock::from(
                Some("Peter Molfese".to_string()),
                SimpleTime::from_milliseconds(0),
                SimpleTime::from_milliseconds(1000),
                "Hello, welcome to the caption tool!".to_string()
            ).unwrap();
            assert_eq!(cap.blocks.len(), 1);
            let received_block = &cap.blocks[0];
            assert_eq!(expected_block.speaker, received_block.speaker);
            assert_eq!(expected_block.start, received_block.start);
            assert_eq!(expected_block.end, received_block.end);
            assert_eq!(expected_block.text, received_block.text);
        }
        #[test]
        fn test_parse_block_no() {
            let n = SrtParser::block_number("1").expect("");
            assert_eq!(n, 1);

            let n = SrtParser::block_number("a");
            match n {
                Ok(_) => panic!("Test failure! SrtParser parses 'a'"),
                Err(e) => {
                    match e {
                        SrtParserError::UnexpectedEndOfFile => {
                            panic!("Test failure! SrtParser wrong err");
                        },
                        SrtParserError::ExpectedBlockNumber(s) => {
                            assert_eq!(s, "a");
                        },
                        _ => panic!("Unknown test failure")
                    };
                },
            };
        }
        #[test]
        fn test_parse_block_timestamps() {
            let test_str_1 = "00:00:00,000 --> 00:00:01,001";
            let r = SrtParser::block_timestamps(test_str_1);
            match r {
                Ok((start, end)) => {
                    assert_eq!(start.to_milliseconds(), 0);
                    assert_eq!(end.to_milliseconds(), 1001);
                }
                _ => panic!("Test failed"),
            }
        }
        #[test]
        fn test_parse_block_timestamps_missing_start() {
            // Test that we fail for no block start
            let test_str_3 = "--> 00:00:01,001";
            let r = SrtParser::block_timestamps(test_str_3);
            match r {
                Ok((start, end)) => {
                    panic!("Parsed {:?}, {:?} when should have failed", start, end);
                },
                Err(e) => {
                    match e {
                        SrtParserError::InvalidTimestamp(_s) => {},
                        _ => panic!("Test failed in unexpected way"),
                    };
                },
            };
        }
        #[test]
        fn test_parse_block_text() {
            // Test to make sure we parse a line of text
            let spk = "Peter Molfese";
            let txt = "The quick brown fox jumps over the lazy dog.";

            let test_str = format!("[{}] {}", spk, txt);
            let (speaker, text) = SrtParser::block_text(&test_str)
                .expect("Should be fine");
            assert_eq!(speaker, Some(spk.to_string()));
            assert_eq!(text, txt.to_string());
        }
        #[test]
        fn test_parse_block() {
            // Test to make sure we parse an entire block
            let start = "00:00:00,000";
            let end = "00:00:01,000";
            let text = "The quick brown fox jumps over the lazy dog";
            let test_input = format!("\n{}\n{} --> {}\n{}\n", 1, start, end, text);
            let cb = SrtParser::block(&test_input)
                .expect("Failed test");
            assert_eq!(cb.start().to_milliseconds(), 0);
            assert_eq!(cb.end().to_milliseconds(), 1000);
            assert_eq!(cb.speaker(), None);
            assert_eq!(cb.text(), text);
        }
        #[test]
        fn test_parse_block_fails_insufficient_lines() {
            // Test to make sure we fail for no blank
            let x = SrtParser::block("thing\n");
            match x {
                Err(SrtParserError::UnexpectedEndOfFile) => {},
                _ => panic!("Didn't get unexpected EOF {:?}", x),
            };
        }
    }
    mod srt_writer {
        use super::*;
        #[test]
        fn write() {
            let cap = Caption {
                header: None,
                blocks: vec!(
                    CaptionBlock {
                        speaker: Some("Pete Molfese".to_string()),
                        start: SimpleTime::from_milliseconds(0),
                        end: SimpleTime::from_milliseconds(1000),
                        text: "Hello, world!".to_string(),
                    }
                )
            };
            let should_get = format!(
                "{}\n{} --> {}\n{}\n",
                1,
                "00:00:00,000",
                "00:00:01,000",
                "[Pete Molfese] Hello, world!"
            );
            assert_eq!(SrtWriter::write(&cap), should_get);
        }
    }
}
