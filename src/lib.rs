/// Library to help sort out a few things

// Useful constants
const MILLIS_PER_SECOND: usize = 1000;
const MILLIS_PER_MINUTE: usize = 60 * MILLIS_PER_SECOND;
const MILLIS_PER_HOUR: usize = 60 * MILLIS_PER_MINUTE;

/// Simple time object to store hours, minutes, seconds, and milliseconds.
///
/// # Examples
/// Convert a simple timestamp
/// ```
/// use offset_caption::SimpleTime;
///
/// let t = SimpleTime::from_srt("13:15:03.450");
/// assert_eq!(t.hour(), 13);
/// assert_eq!(t.minute(), 15);
/// assert_eq!(t.second(), 3);
/// assert_eq!(t.millisecond(), 450);
/// assert_eq!(t.to_milliseconds(), 47_703_450)
/// ```
///
/// Convert a float to a SimpleTime
/// ```
/// use offset_caption::SimpleTime;
///
/// let t = SimpleTime::from_milliseconds(47_703_450);
/// assert_eq!(t.hour(), 13);
/// assert_eq!(t.minute(), 15);
/// assert_eq!(t.second(), 3);
/// assert_eq!(t.millisecond(), 450);
/// ```
pub struct SimpleTime {
    hours: usize,
    minutes: usize,
    seconds: usize,
    milliseconds: usize,
}

impl SimpleTime {
    /// Create a SimpleTime from an srt string
    pub fn from_srt(time: &str) -> SimpleTime {
        // Check to make sure that the passed string is the correct size
        if !(time.len() == 12) {
            panic!("Time {} is length {}, should be 12", time, time.len());
        }
        let invalid_message = format!(
            "{} is not a valid srt time expression ({})",
            time,
            "HH:MM:SS.LLL"
        );
        let delim_1 = time.chars().nth(2).expect(&invalid_message);
        let delim_2 = time.chars().nth(5).expect(&invalid_message);
        let delim_3 = time.chars().nth(8).expect(&invalid_message);
        if !(delim_1 == ':' && delim_2 == ':') {
            panic!("{}", invalid_message);
        }
        if !(delim_3 == '.' || delim_3 == ',') {
            panic!("{}", invalid_message);
        }
        let hours = time[0..2].parse::<usize>()
            .expect(&invalid_message);
        let minutes = time[3..5].parse::<usize>()
            .expect(&invalid_message);
        let seconds = time[6..8].parse::<usize>()
            .expect(&invalid_message);
        let milliseconds = time[9..].parse::<usize>()
            .expect(&invalid_message);

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
    /// Create a string from a SimpleTime
    pub fn to_str<'a>(self) -> String {
        format!(
            "{:02}:{:02}:{:02},{:03}",
            self.hours,
            self.minutes,
            self.seconds,
            self.milliseconds,
        )
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
    /// Offset this timestamp by adding more milliseconds
    pub fn offset(&mut self, offset: usize) -> () {
        *self = SimpleTime::from_milliseconds(
            self.to_milliseconds() + offset
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod srt {
        #[test]
        #[should_panic(expected = "Time cat is length 3, should be 12")]
        fn from_str_fails_length() {
            super::SimpleTime::from_srt("cat");
        }
        #[test]
        #[should_panic(expected = "             is not a valid srt time expression")]
        fn from_str_fails_spec() {
            super::SimpleTime::from_srt("            ");
        }
        #[test]
        fn test_str_works() {
            let st = super::SimpleTime::from_srt("23:54:17.837");
            assert_eq!(st.hour(), 23);
            assert_eq!(st.minute(), 54);
            assert_eq!(st.second(), 17);
            assert_eq!(st.millisecond(), 837);
        }
        #[test]
        fn test_to_from_str_works() {
            let st = super::SimpleTime::from_srt("03:05:06.001");
            assert_eq!(st.to_str(), "03:05:06,001");
        }
        #[test]
        fn test_to_from_millis_works() {
            let st = super::SimpleTime::from_srt("23:54:17.837");
            assert_eq!(st.to_milliseconds(), 86057837);
            let st2 = super::SimpleTime::from_milliseconds(86897);
            assert_eq!(st2.to_milliseconds(), 86897);
        }
        #[test]
        fn test_offset() {
            const MILLS: usize = 86057837;
            let mut st = super::SimpleTime::from_srt("00:00:00.000");
            st.offset(MILLS);
            assert_eq!(st.to_milliseconds(), 86057837);
        }
    }
}
