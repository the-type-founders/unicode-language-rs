include!(concat!(env!("OUT_DIR"), "/data.rs"));

use std::cmp;

#[derive(Debug)]
pub struct Match {
    /// ISO 639-1 language code.
    pub code: &'static str,
    /// English name.
    pub name: &'static str,
    /// Name in native script.
    pub native: &'static str,
    /// Number of codepoints matched.
    pub count: u32,
    /// Score (number of codepoints matched divided by the total).
    pub score: f64,
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
pub fn detect<T>(codepoints: T, threshold: f64) -> Vec<Match>
where
    T: IntoIterator<Item = Range<Codepoint>>,
{
    let mut counts = [0; LANGUAGE_COUNT];

    for [input_lower, input_upper] in codepoints {
        for i in 0..LANGUAGE_COUNT {
            for [range_lower, range_upper] in RANGES[i] {
                if input_lower <= *range_upper && *range_lower <= input_upper {
                    counts[i] += cmp::min(input_upper, *range_upper)
                        - cmp::max(input_lower, *range_lower)
                        + 1;
                }

                if input_upper <= *range_upper {
                    break;
                }
            }
        }
    }

    let mut result = Vec::new();

    for i in 0..LANGUAGE_COUNT {
        let score = counts[i] as f64 / TOTALS[i] as f64;
        if score >= threshold && counts[i] > 0 {
            result.push(Match {
                code: METADATA[i].code,
                name: METADATA[i].name,
                native: METADATA[i].native_name,
                count: counts[i],
                score,
            });
        }
    }

    result.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse());

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
    fn it_takes_a_vector() {
        let codepoints = vec![[1, 3]];

        let result = detect(codepoints, 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
    }

    #[test]
    fn it_takes_an_array() {
        let codepoints = [[1, 3]];

        let result = detect(codepoints, 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
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
        assert_eq!(result[0].code, "t4");
        assert_eq!(result[0].name, "test4");
        assert_eq!(result[1].code, "t3");
        assert_eq!(result[1].name, "test3");
    }

    #[test]
    fn it_returns_correct_counts_on_partial_range_matches() {
        let result = detect([[3, 5]], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t2");
        assert_eq!(result[0].name, "test2");
        assert_eq!(result[0].count, 2);
        assert_eq!(result[1].code, "t1");
        assert_eq!(result[1].name, "test1");
        assert_eq!(result[1].count, 1);
    }

    #[test]
    fn it_returns_sorted_results() {
        let result = detect([[1, 1], [4, 6]], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t2");
        assert_eq!(result[1].code, "t1");
    }

    #[test]
    fn it_handles_ranges_correctly() {
        let result = detect([[12, 20]], 0.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t5");
    }
}
