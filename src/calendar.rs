//! Calendar helpers shared by the time-pattern and trend analyses.

/// Split an ISO 8601 timestamp into its date and time components at the single `T`.
pub(crate) fn split_date_time(ts: &str) -> Option<(&str, &str)> {
    let (date_part, time_part) = ts.split_once('T')?;
    if time_part.contains('T') {
        return None;
    }
    Some((date_part, time_part))
}

/// Parse the `YYYY-MM-DD` date component, rejecting out-of-range months and days.
pub(crate) fn parse_date(date: &str) -> Option<(i32, u32, u32)> {
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }
    Some((year, month, day))
}

/// Return the number of days in a given month, accounting for leap years
pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) { 29 } else { 28 }
        }
        _ => 0,
    }
}

/// Check if a year is a leap year
pub(crate) fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Mean of the recorded values, or `None` when nothing was recorded.
pub(crate) fn average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average() {
        assert_eq!(average(&[]), None);
        assert_eq!(average(&[2.0, 4.0]), Some(3.0));
    }
}
