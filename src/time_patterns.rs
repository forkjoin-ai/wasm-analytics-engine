use super::{Reflection, TimePattern, EmotionCount, TimePatternsResult};
use crate::calendar::{average, parse_date, split_date_time};
use std::collections::HashMap;

const DAY_NAMES: [&str; 7] = [
    "sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday",
];

const TIME_OF_DAY_NAMES: [&str; 4] = ["morning", "afternoon", "evening", "night"];

/// Compute time patterns from reflections
pub fn compute_time_patterns(
    reflections: &[Reflection],
) -> TimePatternsResult {
    let mut day_of_week_map: HashMap<String, PatternData> = HashMap::new();
    let mut time_of_day_map: HashMap<String, PatternData> = HashMap::new();
    let mut month_map: HashMap<String, PatternData> = HashMap::new();

    for reflection in reflections {
        // Parse timestamp
        let timestamp = match parse_timestamp(&reflection.timestamp) {
            Some(ts) => ts,
            None => continue,
        };

        let day_of_week = DAY_NAMES[timestamp.weekday() as usize];
        let time_of_day = time_of_day_bucket(timestamp.hour());
        let month = format!("{:04}-{:02}", timestamp.year(), timestamp.month());

        let emotion_id = reflection.emotion_id.as_deref().unwrap_or("unknown");
        let emotion_name = reflection.emotion_name.as_deref().unwrap_or("Unknown");

        for (map, period) in [
            (&mut day_of_week_map, day_of_week),
            (&mut time_of_day_map, time_of_day),
            (&mut month_map, month.as_str()),
        ] {
            update_pattern_data(map, period, emotion_id, emotion_name, reflection.intensity);
        }
    }

    TimePatternsResult {
        day_of_week: format_patterns(day_of_week_map, &DAY_NAMES),
        time_of_day: format_patterns(time_of_day_map, &TIME_OF_DAY_NAMES),
        month: format_patterns(month_map, &[]),
    }
}

/// Bucket an hour (0-23) into one of `TIME_OF_DAY_NAMES`.
fn time_of_day_bucket(hour: u32) -> &'static str {
    match hour {
        5..=11 => "morning",
        12..=16 => "afternoon",
        17..=21 => "evening",
        _ => "night",
    }
}

struct PatternData {
    count: usize,
    intensities: Vec<f64>,
    emotions: HashMap<String, (String, usize)>, // emotion_id -> (emotion_name, count)
}

fn update_pattern_data(
    map: &mut HashMap<String, PatternData>,
    period: &str,
    emotion_id: &str,
    emotion_name: &str,
    intensity: Option<f64>,
) {
    let data = map.entry(period.to_string()).or_insert_with(|| PatternData {
        count: 0,
        intensities: Vec::new(),
        emotions: HashMap::new(),
    });

    data.count += 1;
    if let Some(int) = intensity {
        data.intensities.push(int);
    }
    let emotion_entry = data
        .emotions
        .entry(emotion_id.to_string())
        .or_insert_with(|| (emotion_name.to_string(), 0));
    emotion_entry.1 += 1;
}

fn format_patterns(
    map: HashMap<String, PatternData>,
    order: &[&str],
) -> Vec<TimePattern> {
    let mut patterns: Vec<TimePattern> = map
        .into_iter()
        .map(|(period, data)| TimePattern {
            period,
            count: data.count,
            average_intensity: average(&data.intensities),
            top_emotions: top_emotions(data.emotions, 5),
        })
        .collect();

    // Sort by order if provided, otherwise by count
    if !order.is_empty() {
        patterns.sort_by(|a, b| compare_by_period_order(a, b, order));
    } else {
        patterns.sort_by(|a, b| b.count.cmp(&a.count));
    }

    patterns
}

/// The `limit` most frequent emotions, most frequent first.
fn top_emotions(emotions: HashMap<String, (String, usize)>, limit: usize) -> Vec<EmotionCount> {
    let mut top: Vec<EmotionCount> = emotions
        .into_iter()
        .map(|(emotion_id, (emotion_name, count))| EmotionCount {
            emotion_id,
            emotion_name,
            count,
        })
        .collect();
    top.sort_by(|a, b| b.count.cmp(&a.count));
    top.truncate(limit);
    top
}

/// Known periods follow `order`; unknown periods sort after them by count descending.
fn compare_by_period_order(a: &TimePattern, b: &TimePattern, order: &[&str]) -> std::cmp::Ordering {
    let a_idx = order.iter().position(|&x| x == a.period);
    let b_idx = order.iter().position(|&x| x == b.period);
    match (a_idx, b_idx) {
        (Some(a_i), Some(b_i)) => a_i.cmp(&b_i),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.count.cmp(&a.count),
    }
}

/// Simple timestamp parser (ISO 8601 format)
fn parse_timestamp(ts: &str) -> Option<SimpleDateTime> {
    // Try to parse ISO 8601 format: "2024-01-15T10:00:00Z" or "2024-01-15T10:00:00.000Z"
    let (date_part, time_part) = split_date_time(ts)?;
    let (year, month, day) = parse_date(date_part)?;
    let (hour, minute) = parse_hour_minute(time_part)?;

    Some(SimpleDateTime {
        year,
        month,
        day,
        hour,
        minute,
        weekday: calculate_weekday(year, month, day),
    })
}

/// Parse `HH:MM[:SS[.fff]][Z]`, rejecting out-of-range hours and minutes.
fn parse_hour_minute(time_part: &str) -> Option<(u32, u32)> {
    let time_part = time_part.trim_end_matches('Z');
    // Strip fractional seconds (e.g., ".000" from "10:00:00.000Z")
    let time_part = time_part.split('.').next().unwrap_or(time_part);
    let mut fields = time_part.split(':');
    let hour = fields.next()?.parse::<u32>().ok()?;
    let minute = fields.next()?.parse::<u32>().ok()?;

    if hour > 23 || minute > 59 {
        return None;
    }
    Some((hour, minute))
}

#[allow(dead_code)]
struct SimpleDateTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    weekday: u32, // 0 = Sunday, 6 = Saturday
}

impl SimpleDateTime {
    fn weekday(&self) -> u32 {
        self.weekday
    }

    fn hour(&self) -> u32 {
        self.hour
    }

    fn year(&self) -> i32 {
        self.year
    }

    fn month(&self) -> u32 {
        self.month
    }

    #[cfg(test)]
    fn day(&self) -> u32 {
        self.day
    }

    #[cfg(test)]
    fn minute(&self) -> u32 {
        self.minute
    }
}

/// Calculate weekday using Zeller's congruence
/// Returns 0=Sunday, 1=Monday, ..., 6=Saturday
fn calculate_weekday(year: i32, month: u32, day: u32) -> u32 {
    let mut y = year;
    let mut m = month as i32;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;
    let h = ((day as i32 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7 + 7) % 7;
    ((h + 6) % 7) as u32 // Convert Zeller (0=Sat) to 0=Sunday, 6=Saturday
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        let ts = parse_timestamp("2024-01-15T10:00:00Z");
        assert!(ts.is_some());
        let dt = ts.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_parse_timestamp_with_fractional_seconds() {
        let ts = parse_timestamp("2024-01-15T10:30:00.000Z");
        assert!(ts.is_some());
        let dt = ts.unwrap();
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn test_parse_timestamp_rejects_invalid_month() {
        assert!(parse_timestamp("2024-13-15T10:00:00Z").is_none());
        assert!(parse_timestamp("2024-00-15T10:00:00Z").is_none());
    }

    #[test]
    fn test_parse_timestamp_rejects_invalid_day() {
        assert!(parse_timestamp("2024-02-30T10:00:00Z").is_none());
        assert!(parse_timestamp("2024-01-32T10:00:00Z").is_none());
        assert!(parse_timestamp("2024-01-00T10:00:00Z").is_none());
    }

    #[test]
    fn test_parse_timestamp_rejects_invalid_hour() {
        assert!(parse_timestamp("2024-01-15T25:00:00Z").is_none());
    }

    #[test]
    fn test_parse_timestamp_rejects_invalid_minute() {
        assert!(parse_timestamp("2024-01-15T10:60:00Z").is_none());
    }

    #[test]
    fn test_leap_year_feb_29() {
        // 2024 is a leap year
        let ts = parse_timestamp("2024-02-29T10:00:00Z");
        assert!(ts.is_some());
        // 2023 is not a leap year
        let ts = parse_timestamp("2023-02-29T10:00:00Z");
        assert!(ts.is_none());
    }

    #[test]
    fn test_calculate_weekday() {
        // January 15, 2024 is a Monday (1)
        let weekday = calculate_weekday(2024, 1, 15);
        assert_eq!(weekday, 1);
        // January 14, 2024 is a Sunday (0)
        let weekday = calculate_weekday(2024, 1, 14);
        assert_eq!(weekday, 0);
        // January 20, 2024 is a Saturday (6)
        let weekday = calculate_weekday(2024, 1, 20);
        assert_eq!(weekday, 6);
    }
}
