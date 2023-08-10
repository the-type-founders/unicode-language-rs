include!(concat!(env!("OUT_DIR"), "/data.rs"));

use std::cmp;

#[derive(Debug)]
pub struct Match {
    /// ISO 639-1 language code.
    pub code: String,
    /// English name.
    pub name: String,
    /// Name in native script.
    pub native: String,
    /// Number of codepoints matched.
    pub count: u32,
    /// Score (number of codepoints matched divided by the total).
    pub score: f32,
}

/// Detects language support in a font given a list of Unicode
/// codepoint ranges.
///
/// # Arguments
///
/// * `codepoints` - An iterator of codepoint ranges. The iterator
///   must not contain overlapping ranges and must be sorted in
///   ascending order.
/// * `threshold` - The minimum score a language must have to be
/// returned as a match. Value must be between 0 and 1.
///
/// Returns a vector of language matches.
pub fn detect<T>(codepoints: T, threshold: f32) -> Vec<Match>
where
    T: IntoIterator<Item = Range<Codepoint>>,
{
    let mut counts = [0; LANGUAGE_COUNT];

    for [input_lower, input_upper] in codepoints {
        for i in 0..LANGUAGE_COUNT {
            for [range_lower, range_upper] in RANGES[i] {
                if input_lower.ge(range_lower) && input_lower.le(range_upper)
                    || input_upper.ge(range_lower) && input_upper.le(range_upper)
                {
                    counts[i] += cmp::min(input_upper, *range_upper)
                        - cmp::max(input_lower, *range_lower)
                        + 1;
                }

                if input_lower.lt(range_lower) {
                    break;
                }
            }
        }
    }

    let mut result: Vec<Match> = Vec::new();

    for i in 0..LANGUAGE_COUNT {
        let score = counts[i] as f32 / TOTALS[i] as f32;
        if score >= threshold && counts[i] > 0 {
            result.push(Match {
                code: METADATA[i].code.to_string(),
                name: METADATA[i].name.to_string(),
                native: METADATA[i].native_name.to_string(),
                count: counts[i],
                score,
            });
        }
    }

    result.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_an_empty_array() {
        let result = detect([], 0.5);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn it_returns_an_empty_array_with_an_invalid_codepoint() {
        let result = detect([[256, 256]], 0.5);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn it_returns_the_test_language() {
        let result = detect([[1, 1]], 0.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1")
    }

    #[test]
    fn it_does_not_return_if_threshold_not_met() {
        let result = detect([[1, 2]], 1.0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn it_returns_if_threshold_is_met() {
        let result = detect([[1, 3]], 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
    }

    #[test]
    fn it_returns_if_threshold_is_partially_met() {
        let result = detect([[1, 2]], 0.6);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
    }

    #[test]
    fn it_returns_multiple_languages() {
        let result = detect([[1, 1], [4, 4]], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
        assert_eq!(result[1].code, "t2");
        assert_eq!(result[1].name, "test2");
    }

    #[test]
    fn it_returns_overlapping_languages() {
        let result = detect([[8, 8]], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t3");
        assert_eq!(result[0].name, "test3");
        assert_eq!(result[1].code, "t4");
        assert_eq!(result[1].name, "test4");
    }

    #[test]
    fn it_returns_correct_counts_on_partial_range_matches() {
        let result = detect([[3, 5]], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
        assert_eq!(result[0].count, 1);
        assert_eq!(result[1].code, "t2");
        assert_eq!(result[1].name, "test2");
        assert_eq!(result[1].count, 2);
    }
}
