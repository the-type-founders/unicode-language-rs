use std::env;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use glob::glob;
use langtag::LangTag;
use serde::{de::Error, Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq)]
struct Range(u32, u32);

#[derive(Debug, Deserialize)]
struct Language {
    anglicized_name: String,
    native_name: String,
    codepoints: Vec<Range>,
    tag: Option<String>,
}

#[derive(Debug)]
pub struct Metadata {
    pub tag: String,
    pub name: String,
    pub native_name: String,
}

impl<'l> Deserialize<'l> for Range {
    fn deserialize<T>(deserializer: T) -> Result<Self, T::Error>
    where
        T: Deserializer<'l>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input {
            Number(u32),
            Text(String),
        }

        match Input::deserialize(deserializer)? {
            Input::Number(value) => Ok(Range(value, value)),
            Input::Text(value) => {
                if let Some((lower, upper)) = value.split_once("..") {
                    let lower = lower.parse::<u32>().map_err(T::Error::custom)?;
                    let upper = upper.parse::<u32>().map_err(T::Error::custom)?;
                    Ok(Range(lower, upper))
                } else {
                    value
                        .parse::<u32>()
                        .map(|value| Range(value, value))
                        .map_err(T::Error::custom)
                }
            }
        }
    }
}

// The speakeasy data files are Ruby YAML and encode ranges as tagged scalars,
// for example `- !ruby/range 65..90`. This build script does not need Ruby tag
// semantics; it only needs the scalar range text. Normalize tagged ranges into
// quoted YAML strings and leave ordinary numeric codepoints alone.
fn normalize(value: &str) -> String {
    value
        .lines()
        .map(|line| {
            let Some(tag_start) = line.find("!ruby/range ") else {
                return line.to_string();
            };
            let prefix = &line[..tag_start];
            let rest = &line[tag_start + "!ruby/range ".len()..];
            let range = rest.split_whitespace().next().unwrap();
            format!("{prefix}\"{range}\"")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse<T: AsRef<Path>>(path: T) -> Language {
    let path = path.as_ref();

    let s = read_to_string(path).unwrap();

    let mut d: Language = serde_saphyr::from_str(&normalize(&s)).unwrap();

    d.tag = Some(
        path.file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap(),
    );

    // Sort the ranges so we can exit early when running the detection code.
    d.codepoints.sort_by_key(|c| c.0);

    d
}

fn main() {
    let languages: Vec<Language> = glob("./speakeasy/data/*")
        .unwrap()
        .map(Result::unwrap)
        .map(parse)
        .filter(|l| LangTag::new(l.tag.as_ref().unwrap()).is_ok())
        .collect();

    let ranges: Vec<Vec<Range>> = languages.iter().map(|l| l.codepoints.to_vec()).collect();
    let totals: Vec<u32> = ranges
        .iter()
        .map(|ranges| ranges.iter().map(|c| c.1 - c.0 + 1).sum::<u32>())
        .collect();

    let metadata: Vec<Metadata> = languages
        .into_iter()
        .map(|l| Metadata {
            tag: l.tag.as_ref().unwrap().clone(),
            name: l.anglicized_name.clone(),
            native_name: l.native_name.clone(),
        })
        .collect();

    let language_count = ranges.len();

    let ranges_str = ranges
        .iter()
        .map(|ranges| {
            format!(
                "&[{}]",
                ranges
                    .iter()
                    .map(|c| format!("[{}, {}]", c.0, c.1))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data.rs");
    let mut f = File::create(dest_path).unwrap();

    write!(
        f,
        r#"
/// A Unicode codepoint
pub type Codepoint = u32;

/// A range of Unicode codepoints.
pub type Range<T> = [T; 2];

struct Metadata {{
    tag: &'static str,
    name: &'static str,
    native_name: &'static str,
}}

#[cfg(not(test))]
const LANGUAGE_COUNT: usize = {language_count};

#[cfg(test)]
const LANGUAGE_COUNT: usize = 5;

#[cfg(not(test))]
const RANGES: [&[Range<Codepoint>]; LANGUAGE_COUNT] = [{ranges_str}];

#[cfg(test)]
const RANGES: [&[Range<Codepoint>]; LANGUAGE_COUNT] = [&[[1, 3]], &[[4, 6]], &[[7, 9]], &[[8, 8]], &[[16,16]]];

#[cfg(not(test))]
const TOTALS: [u32; LANGUAGE_COUNT] = {totals:?};

#[cfg(test)]
const TOTALS: [u32; LANGUAGE_COUNT] = [3, 3, 3, 1, 1];

#[cfg(not(test))]
const METADATA: [Metadata; LANGUAGE_COUNT] = {metadata:?};

#[cfg(test)]
const METADATA: [Metadata; LANGUAGE_COUNT] = [
  Metadata {{ tag: "t1", name: "test1", native_name: "ntest1" }},
  Metadata {{ tag: "t2", name: "test2", native_name: "ntest2" }},
  Metadata {{ tag: "t3", name: "test3", native_name: "ntest3" }},
  Metadata {{ tag: "t4", name: "test4", native_name: "ntest4" }},
  Metadata {{ tag: "t5", name: "test5", native_name: "ntest5" }},
];
"#
    )
    .unwrap();
}
