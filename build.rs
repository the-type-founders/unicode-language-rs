use std::env;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use glob::glob;
use serde::{de::Error, Deserialize, Deserializer};
use serde_yaml::{self};

#[derive(Clone, Debug, PartialEq)]
struct Codepoint(u32, u32);

#[derive(Debug, Deserialize)]
struct Language {
    anglicized_name: String,
    native_name: String,
    codepoints: Vec<Codepoint>,
    code: Option<String>,
}

#[derive(Debug)]
pub struct Metadata {
    pub code: String,
    pub name: String,
    pub native_name: String,
}

impl<'de> Deserialize<'de> for Codepoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        if s.contains("..") {
            s.split("..")
                .map(|x| x.parse::<u32>())
                .collect::<Result<Vec<_>, _>>()
                .map(|v| Codepoint(v[0], v[1]))
                .map_err(D::Error::custom)
        } else {
            s.parse::<u32>()
                .map(|i| Codepoint(i, i))
                .map_err(D::Error::custom)
        }
    }
}

fn parse_yaml<T: AsRef<Path>>(path: T) -> Language {
    let path = path.as_ref();

    let s = read_to_string(path).unwrap();

    // The Serde YAML parser expects YAML types to have names that are valid
    // Rust identifiers. Sadly, that is not the case here, so we manually perform
    // a string replace to patch up the data.
    let mut d: Language = serde_yaml::from_str(&s.replace("ruby/range", "Range")).unwrap();

    d.code = Some(
        path.file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap(),
    );

    // Sort the codepoints so we can exit early when running the detection code.
    d.codepoints.sort_by_key(|c| c.0);

    d
}

fn main() {
    let languages: Vec<Language> = glob("./speakeasy/data/*")
        .unwrap()
        .map(|path| path.unwrap())
        .map(parse_yaml)
        .collect();

    let ranges: Vec<Vec<Codepoint>> = languages.iter().map(|l| l.codepoints.to_vec()).collect();
    let totals: Vec<u32> = ranges
        .iter()
        .map(|codepoints| codepoints.iter().map(|c| c.1 - c.0 + 1).sum::<u32>())
        .collect();

    let metadata: Vec<Metadata> = languages
        .iter()
        .map(|l| Metadata {
            code: l.code.as_ref().unwrap().clone(),
            name: l.anglicized_name.clone(),
            native_name: l.native_name.clone(),
        })
        .collect();

    let language_count = ranges.len();

    let ranges_str = ranges
        .iter()
        .map(|codepoints| {
            format!(
                "vec![{}]",
                codepoints
                    .iter()
                    .map(|c| format!("({},{})", c.0, c.1))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let totals_str = format!("{:?}", totals);
    let metadata_str = format!("{:?}", metadata);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data.rs");
    let mut f = File::create(dest_path).unwrap();

    write!(
        f,
        r#"
use std::sync::OnceLock;

struct Metadata {{
    code: &'static str,
    name: &'static str,
    native_name: &'static str,
}}

#[cfg(not(test))]
fn ranges() -> &'static [Vec<(u32, u32)>; {language_count}] {{
  static RANGES: OnceLock<[Vec<(u32, u32)>; {language_count}]> = OnceLock::new();

  RANGES.get_or_init(|| {{
    [{ranges_str}]
  }})
}}

#[cfg(test)]
fn ranges() -> &'static [Vec<(u32, u32)>; 4] {{
  static RANGES: OnceLock<[Vec<(u32, u32)>; 4]> = OnceLock::new();

  RANGES.get_or_init(|| {{
    [vec![(1, 3)], vec![(4, 6)], vec![(7, 9)], vec![(8, 8)]]
  }})
}}

#[cfg(not(test))]
fn totals() -> &'static [u32; {language_count}] {{
  static TOTALS: OnceLock<[u32; {language_count}]> = OnceLock::new();

  TOTALS.get_or_init(|| {{
    {totals_str}
  }})
}}

#[cfg(test)]
fn totals() -> &'static [u32; 4] {{
  static TOTALS: OnceLock<[u32; 4]> = OnceLock::new();

  TOTALS.get_or_init(|| {{
    [3, 3, 3, 1]
  }})
}}

#[cfg(not(test))]
fn metadata() -> &'static [Metadata; {language_count}] {{
  static METADATA: OnceLock<[Metadata; {language_count}]> = OnceLock::new();

  METADATA.get_or_init(|| {{
    {metadata_str}
  }})
}}

#[cfg(test)]
fn metadata() -> &'static [Metadata; 4] {{
  static METADATA: OnceLock<[Metadata; 4]> = OnceLock::new();

  METADATA.get_or_init(|| {{
    [
      Metadata {{ code: "t1", name: "test1", native_name: "ntest1" }},
      Metadata {{ code: "t2", name: "test2", native_name: "ntest2" }},
      Metadata {{ code: "t3", name: "test3", native_name: "ntest3" }},
      Metadata {{ code: "t4", name: "test4", native_name: "ntest4" }},
    ]
  }})
}}

#[cfg(not(test))]
const LANGUAGE_COUNT: usize = {language_count};

#[cfg(test)]
const LANGUAGE_COUNT: usize = 4;
"#
    )
    .unwrap();
}
