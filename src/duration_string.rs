use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DurationString {
    pub duration: chrono::Duration,
}

impl DurationString {
    pub fn new(duration: chrono::Duration) -> Self {
        DurationString { duration }
    }
}

impl fmt::Display for DurationString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // pretty print the duration
        let mut duration = self.duration;
        if duration.num_seconds() <= 0 {
            return write!(f, "0 seconds");
        }
        if duration.num_days() > 0 {
            write!(f, "{} days", duration.num_days())?;
            duration = duration - chrono::Duration::days(duration.num_days());
        }
        if duration.num_hours() > 0 {
            write!(f, " {} hours", duration.num_hours())?;
            duration = duration - chrono::Duration::hours(duration.num_hours());
        }
        if duration.num_minutes() > 0 {
            write!(f, " {} minutes", duration.num_minutes())?;
            duration = duration - chrono::Duration::minutes(duration.num_minutes());
        }
        if duration.num_seconds() > 0 {
            write!(f, " {} seconds", duration.num_seconds())?;
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_string() {
        let duration = chrono::Duration::days(1)
            + chrono::Duration::hours(2)
            + chrono::Duration::minutes(3)
            + chrono::Duration::seconds(4);
        let duration_string = DurationString::new(duration);
        assert_eq!(
            format!("{}", duration_string),
            "1 days 2 hours 3 minutes 4 seconds"
        );
    }

    #[test]
    fn test_duration_string_when_duration_is_smaller_than_seconds() {
        let duration = chrono::Duration::milliseconds(4);
        let duration_string = DurationString::new(duration);
        assert_eq!(format!("{}", duration_string), "0 seconds");
    }
}
