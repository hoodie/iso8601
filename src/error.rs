/// Error returned when converting a [`Duration`](crate::Duration) that has
/// a non-zero `year` or `month` component into an exact, fixed-length
/// duration.
///
/// `year` and `month` are not fixed spans of time: their length depends on
/// the calendar date they are anchored to (leap years, varying month
/// lengths). There is no context-free, correct number of seconds they
/// convert to, so rather than guessing (e.g. approximating a year as `365`
/// days), the conversion fails.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct AmbiguousDuration;

impl core::fmt::Display for AmbiguousDuration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            "duration has a non-zero year or month component and cannot be converted to an exact duration",
        )
    }
}
