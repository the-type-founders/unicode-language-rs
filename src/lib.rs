include!(concat!(env!("OUT_DIR"), "/data.rs"));

use std::cmp;

#[derive(Debug)]
pub struct Match {
    /// ISO 639-1 language codes.
    pub code: String,
    /// English name.
    pub name: String,
    /// Name in native script.
    pub native: String,
    /// Number of codepoints matched.
    pub count: u32,
    /// The score (number of codepoints matches divided by the total language codepoints).
    pub score: f32,
}

/// A Unicode codepoint
pub type Codepoint = u32;

/// A range of Unicode codepoints.
pub type Range<T> = [T; 2];

pub fn detect<T>(codepoints: T, threshold: f32) -> Vec<Match>
where
    T: IntoIterator<Item = Range<Codepoint>>,
{
    let mut counts = [0; LANGUAGE_COUNT];
    let ranges = ranges();

    for [input_lower, input_upper] in codepoints {
        for i in 0..LANGUAGE_COUNT {
            for k in 0..ranges[i].len() {
                let (range_lower, range_upper) = ranges[i][k];

                if input_lower >= range_lower && input_lower <= range_upper
                    || input_upper >= range_lower && input_upper <= range_upper
                {
                    counts[i] +=
                        cmp::min(input_upper, range_upper) - cmp::max(input_lower, range_lower) + 1;
                }

                if range_lower > input_lower {
                    break;
                }
            }
        }
    }

    let totals = totals();
    let metadata = metadata();
    let mut result: Vec<Match> = Vec::new();

    for i in 0..LANGUAGE_COUNT {
        let score = counts[i] as f32 / totals[i] as f32;
        if score >= threshold && counts[i] > 0 {
            result.push(Match {
                code: metadata[i].code.to_string(),
                name: metadata[i].name.to_string(),
                native: metadata[i].native_name.to_string(),
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
}
