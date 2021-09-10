/// Library to help sort out a few things

use std::fmt;

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
            panic!("SimpleTime requires minutes be in [0, 60]");
        }
        if seconds >= 60 {
            panic!("SimpleTime requires seconds be in [0, 60]");
        }
        if milliseconds >= 999 {
            panic!("SimpleTime requires milliseconds be in [0, 999]");
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
}
