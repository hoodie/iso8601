use core::str::FromStr;

use alloc::string::String;

use crate::error::AmbiguousDuration;
use crate::parsers;

/// A time duration.
/// Durations:
/// <https://www.rfc-editor.org/rfc/rfc3339#page-13>
///    dur-second        = 1*DIGIT "S"
///    dur-minute        = 1*DIGIT "M" [dur-second]
///    dur-hour          = 1*DIGIT "H" [dur-minute]
///    dur-time          = "T" (dur-hour / dur-minute / dur-second)
///    dur-day           = 1*DIGIT "D"
///    dur-week          = 1*DIGIT "W"
///    dur-month         = 1*DIGIT "M" [dur-day]
///    dur-year          = 1*DIGIT "Y" [dur-month]
///    dur-date          = (dur-day / dur-month / dur-year) [dur-time]
///    duration          = "P" (dur-date / dur-time / dur-week)
/// ```
///# use std::str::FromStr;
/// assert_eq!(iso8601::Duration::from_str("P2021Y11M16DT23H26M59.123S"), Ok(iso8601::Duration::YMDHMS{ year: 2021, month: 11, day: 16, hour: 23, minute: 26, second: 59, millisecond: 123 }))
/// ```
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Duration {
    /// A duration specified by year, month, day, hour, minute and second units
    YMDHMS {
        /// Number of calendar years
        year: u32,
        /// Number of months
        month: u32,
        /// Number of days
        day: u32,
        /// Number of hours
        hour: u32,
        /// Number of minutes
        minute: u32,
        /// Number of seconds
        second: u32,
        /// Number of milliseconds
        millisecond: u32,
    },
    /// consists of week units
    Weeks(u32),
}

impl Duration {
    /// Whether this duration represents a zero duration.
    pub fn is_zero(&self) -> bool {
        *self
            == Duration::YMDHMS {
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
            }
            || *self == Duration::Weeks(0)
    }

    /// Returns the duration in seconds.
    ///
    /// # Caveat
    ///
    /// `year` and `month` components are not fixed spans of time; their
    /// length depends on the calendar date they are anchored to (leap
    /// years, varying month lengths). This method approximates them as
    /// `365` and `30` days respectively, so the result is inexact for any
    /// duration with a non-zero `year` or `month`. Use a calendar-aware
    /// conversion anchored to a specific date if exactness matters.
    pub fn as_secs(&self) -> u64 {
        match self {
            Duration::YMDHMS {
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
            } => {
                *year as u64 * 365 * 24 * 60 * 60
                    + *month as u64 * 30 * 24 * 60 * 60
                    + *day as u64 * 24 * 60 * 60
                    + *hour as u64 * 60 * 60
                    + *minute as u64 * 60
                    + *second as u64
                    + *millisecond as u64 / 1000
            }
            Duration::Weeks(weeks) => *weeks as u64 * 7 * 24 * 60 * 60,
        }
    }

    /// Returns the duration in milliseconds.
    ///
    /// # Caveat
    ///
    /// `year` and `month` components are not fixed spans of time; their
    /// length depends on the calendar date they are anchored to (leap
    /// years, varying month lengths). This method approximates them as
    /// `365` and `30` days respectively, so the result is inexact for any
    /// duration with a non-zero `year` or `month`. Use a calendar-aware
    /// conversion anchored to a specific date if exactness matters.
    pub fn as_millis(&self) -> u64 {
        match self {
            Duration::YMDHMS {
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
            } => {
                *year as u64 * 365 * 24 * 60 * 60 * 1000
                    + *month as u64 * 30 * 24 * 60 * 60 * 1000
                    + *day as u64 * 24 * 60 * 60 * 1000
                    + *hour as u64 * 60 * 60 * 1000
                    + *minute as u64 * 60 * 1000
                    + *second as u64 * 1000
                    + *millisecond as u64
            }
            Duration::Weeks(weeks) => *weeks as u64 * 7 * 24 * 60 * 60 * 1000,
        }
    }
}

impl Default for Duration {
    fn default() -> Duration {
        Duration::YMDHMS {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
        }
    }
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        duration(s)
    }
}

impl TryFrom<Duration> for ::core::time::Duration {
    type Error = AmbiguousDuration;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        match duration {
            Duration::YMDHMS {
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
            } => {
                if year != 0 || month != 0 {
                    return Err(AmbiguousDuration);
                }
                let secs = u64::from(day) * 86_400
                    + u64::from(hour) * 3600
                    + u64::from(minute) * 60
                    + u64::from(second);
                let nanos = millisecond * 1_000_000;
                Ok(Self::new(secs, nanos))
            }
            Duration::Weeks(week) => {
                let secs = u64::from(week) * 7 * 86_400;
                Ok(Self::from_secs(secs))
            }
        }
    }
}

impl From<::core::time::Duration> for Duration {
    fn from(duration: ::core::time::Duration) -> Self {
        let millisecond = duration.subsec_millis();

        let mut remaining_secs = duration.as_secs();
        let day = remaining_secs / 86_400;
        remaining_secs %= 86_400;
        let hour = remaining_secs / 3_600;
        remaining_secs %= 3_600;
        let minute = remaining_secs / 60;
        let second = remaining_secs % 60;

        Duration::YMDHMS {
            year: 0,
            month: 0,
            day: day as u32,
            hour: hour as u32,
            minute: minute as u32,
            second: second as u32,
            millisecond,
        }
    }
}

/// Parses a duration string.
///
/// A string starts with `P` and can have one of the following formats:
///
/// * Fully-specified duration: `P1Y2M3DT4H5M6S`
/// * Duration in weekly intervals: `P1W`
/// * Fully-specified duration in [`DateTime`](`crate::DateTime`) format: `P<datetime>`
///
/// Both fully-specified formats get parsed into the YMDHMS Duration variant.
/// The weekly interval format gets parsed into the Weeks Duration variant.
///
/// The ranges for each of the individual units are not expected to exceed
/// the next largest unit.
///
/// These ranges (inclusive) are as follows:
///
/// * Year (any valid u32)
/// * Month 0 - 12
/// * Week 0 - 52
/// * Day 0 - 31
/// * Hour 0 - 24
/// * Minute 0 - 60
/// * Second 0 - 60
///
/// ## Examples
///
/// ```rust
/// let duration = iso8601::duration("P1Y2M3DT4H5M6S").unwrap();
/// let duration = iso8601::duration("P1W").unwrap();
/// let duration = iso8601::duration("P2015-11-03T21:56").unwrap();
/// ```
pub fn duration(string: &str) -> Result<Duration, String> {
    if let Ok((_left_overs, parsed)) = parsers::parse_duration(string.as_bytes()) {
        Ok(parsed)
    } else {
        Err(format!("Failed to parse duration: {}", string))
    }
}

#[cfg(test)]
mod test_as_secs {
    use super::Duration;

    #[test]
    fn weeks() {
        assert_eq!(Duration::Weeks(2).as_secs(), 2 * 7 * 24 * 60 * 60);
    }

    #[test]
    fn ymdhms() {
        let duration = Duration::YMDHMS {
            year: 1,
            month: 1,
            day: 1,
            hour: 1,
            minute: 1,
            second: 1,
            millisecond: 500,
        };
        let expected = 365 * 86_400 + 30 * 86_400 + 86_400 + 3_600 + 60 + 1;
        assert_eq!(duration.as_secs(), expected);
    }

    #[test]
    fn zero() {
        assert_eq!(Duration::default().as_secs(), 0);
    }
}

#[cfg(test)]
mod test_as_millis {
    use super::Duration;

    #[test]
    fn weeks() {
        assert_eq!(Duration::Weeks(2).as_millis(), 2 * 7 * 24 * 60 * 60 * 1000);
    }

    #[test]
    fn ymdhms() {
        let duration = Duration::YMDHMS {
            year: 1,
            month: 1,
            day: 1,
            hour: 1,
            minute: 1,
            second: 1,
            millisecond: 500,
        };
        let expected = (365 * 86_400 + 30 * 86_400 + 86_400 + 3_600 + 60 + 1) * 1000 + 500;
        assert_eq!(duration.as_millis(), expected);
    }

    #[test]
    fn zero() {
        assert_eq!(Duration::default().as_millis(), 0);
    }
}

#[cfg(test)]
mod test_from_core_duration {
    use super::Duration;

    #[test]
    fn seconds_only() {
        let core_duration = ::core::time::Duration::from_secs(90);
        let duration: Duration = core_duration.into();
        assert_eq!(
            duration,
            Duration::YMDHMS {
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 1,
                second: 30,
                millisecond: 0,
            }
        );
    }

    #[test]
    fn with_milliseconds() {
        let core_duration = ::core::time::Duration::from_millis(1_500);
        let duration: Duration = core_duration.into();
        assert_eq!(
            duration,
            Duration::YMDHMS {
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                second: 1,
                millisecond: 500,
            }
        );
    }

    #[test]
    fn spanning_days_and_hours() {
        // 1 day, 2 hours, 3 minutes, 4 seconds
        let secs = 86_400 + 2 * 3_600 + 3 * 60 + 4;
        let core_duration = ::core::time::Duration::from_secs(secs);
        let duration: Duration = core_duration.into();
        assert_eq!(
            duration,
            Duration::YMDHMS {
                year: 0,
                month: 0,
                day: 1,
                hour: 2,
                minute: 3,
                second: 4,
                millisecond: 0,
            }
        );
    }

    #[test]
    fn zero() {
        let core_duration = ::core::time::Duration::from_secs(0);
        let duration: Duration = core_duration.into();
        assert_eq!(duration, Duration::default());
    }

    #[test]
    fn round_trips_through_secs() {
        let core_duration = ::core::time::Duration::from_secs(123_456);
        let duration: Duration = core_duration.into();
        assert_eq!(duration.as_secs(), core_duration.as_secs());
    }
}
