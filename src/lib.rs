include!(concat!(env!("OUT_DIR"), "/data.rs"));

#[derive(Debug)]
pub struct Match {
    pub code: String,   // ISO 639-1 language codes;
    pub name: String,   // English name;
    pub native: String, // Name in native script;
    pub count: u32,     // The number of codepoints matched;
    pub score: f32, // The score (number of codepoints match divided by the total for the language).
}

pub fn detect<T>(codepoints: T, threshold: f32) -> Vec<Match>
where
    T: IntoIterator<Item = u32>,
{
    let mut counts = [0; SIZE];
    let ranges = ranges();

    for value in codepoints {
        for i in 0..SIZE {
            for k in 0..ranges[i].len() {
                if value >= ranges[i][k].0 && value <= ranges[i][k].1 {
                    counts[i] += 1;
                }

                if ranges[i][k].0 > value {
                    break;
                }
            }
        }
    }

    let totals = totals();
    let metadata = metadata();
    let mut result: Vec<Match> = Vec::new();

    for i in 0..SIZE {
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
        let result = detect([256], 0.5);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn it_returns_the_test_language() {
        let result = detect([1], 0.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1")
    }

    #[test]
    fn it_does_not_return_if_threshold_not_met() {
        let result = detect([1, 2], 1.0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn it_returns_if_threshold_is_met() {
        let result = detect([1, 2, 3], 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
    }

    #[test]
    fn it_returns_if_threshold_is_partially_met() {
        let result = detect([1, 2], 0.6);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
    }

    #[test]
    fn it_returns_multiple_languages() {
        let result = detect([1, 4], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t1");
        assert_eq!(result[0].name, "test1");
        assert_eq!(result[1].code, "t2");
        assert_eq!(result[1].name, "test2");
    }

    #[test]
    fn it_returns_overlapping_languages() {
        let result = detect([8], 0.0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code, "t3");
        assert_eq!(result[0].name, "test3");
        assert_eq!(result[1].code, "t4");
        assert_eq!(result[1].name, "test4");
    }
}
