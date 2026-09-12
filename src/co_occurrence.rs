use super::{Reflection, CoOccurrence};
use std::collections::HashMap;

/// Maximum number of co-occurrence pairs returned, ordered by count.
const TOP_CO_OCCURRENCES: usize = 20;

/// Compute emotion co-occurrence matrix
pub fn compute_co_occurrence(reflections: &[Reflection]) -> Vec<CoOccurrence> {
    // Keyed by an ordered (lesser, greater) pair borrowed from the input, so no
    // delimiter encoding or per-pair allocation is needed while counting.
    let mut co_occurrence_map: HashMap<(&str, &str), usize> = HashMap::new();
    let total = reflections.len();

    for reflection in reflections {
        let emotions = reflection_emotions(reflection);

        for i in 0..emotions.len() {
            for j in (i + 1)..emotions.len() {
                *co_occurrence_map.entry(ordered_pair(emotions[i], emotions[j])).or_insert(0) += 1;
            }
        }
    }

    let mut result: Vec<CoOccurrence> = co_occurrence_map
        .into_iter()
        .map(|((first, second), count)| CoOccurrence {
            emotion_pair: [first.to_string(), second.to_string()],
            count,
            percentage: if total > 0 {
                (count as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    // Sort by count descending
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result.truncate(TOP_CO_OCCURRENCES);

    result
}

/// Primary emotion followed by related emotions, borrowed from the reflection.
fn reflection_emotions(reflection: &Reflection) -> Vec<&str> {
    reflection
        .emotion_id
        .as_deref()
        .into_iter()
        .chain(reflection.related_emotions.iter().flatten().map(String::as_str))
        .collect()
}

/// Order a pair so (a, b) and (b, a) count as the same co-occurrence.
fn ordered_pair<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reflection(emotion_id: &str, emotion_name: &str, related: Option<&str>) -> Reflection {
        Reflection {
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            emotion_id: Some(emotion_id.to_string()),
            emotion_name: Some(emotion_name.to_string()),
            intensity: Some(7.0),
            related_emotions: related.map(|id| vec![id.to_string()]),
            location: None,
            people: None,
            coping_strategies: None,
            mood_before: None,
            mood_after: None,
        }
    }

    #[test]
    fn test_compute_co_occurrence() {
        let reflections = vec![reflection("joy", "Joy", Some("excitement"))];

        let result = compute_co_occurrence(&reflections);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compute_co_occurrence_hyphenated_ids() {
        let reflections = vec![reflection("mixed-joy", "Mixed Joy", Some("deep-sadness"))];

        let result = compute_co_occurrence(&reflections);
        assert!(!result.is_empty());
        // Verify hyphenated emotion IDs are preserved correctly
        let pair = &result[0].emotion_pair;
        assert!(
            (pair[0] == "deep-sadness" && pair[1] == "mixed-joy")
                || (pair[0] == "mixed-joy" && pair[1] == "deep-sadness"),
            "Expected hyphenated emotion IDs to be preserved, got: {:?}", pair
        );
    }

    #[test]
    fn test_compute_co_occurrence_no_related() {
        let reflections = vec![reflection("joy", "Joy", None)];

        let result = compute_co_occurrence(&reflections);
        // No pairs if only one emotion
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_compute_co_occurrence_pair_order_independent() {
        let reflections = vec![
            reflection("joy", "Joy", Some("calm")),
            reflection("calm", "Calm", Some("joy")),
        ];

        let result = compute_co_occurrence(&reflections);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].emotion_pair, ["calm".to_string(), "joy".to_string()]);
        assert_eq!(result[0].count, 2);
        assert_eq!(result[0].percentage, 100.0);
    }
}
