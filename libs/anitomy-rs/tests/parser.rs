use std::collections::HashMap;

use anitomy::{ElementKind, KeywordConfig};
use serde::{
    de::{value::SeqAccessDeserializer, Visitor},
    Deserialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestValue {
    String(String),
    Array(Vec<String>),
}

impl TestValue {
    fn append(&mut self, value: &str) {
        match self {
            TestValue::String(old) => {
                *self = Self::Array(vec![old.clone(), value.to_string()]);
            }
            TestValue::Array(values) => {
                values.push(value.to_string());
            }
        }
    }
}

struct TestValueVisitor;

impl<'de> Visitor<'de> for TestValueVisitor {
    type Value = TestValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(TestValue::String(v.to_string()))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        Vec::<String>::deserialize(SeqAccessDeserializer::new(seq)).map(TestValue::Array)
    }
}

impl<'de> Deserialize<'de> for TestValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(TestValueVisitor)
    }
}

macro_rules! trace_variables {
    ($($name:ident,)* $block:block, $skip:expr) => {
        let variables = [
            $((stringify!($name), &$name as &dyn ::std::fmt::Debug),)*
        ];
        if let Err(e) = std::panic::catch_unwind(|| $block) {
            eprintln!("----- TRACED VARIABLES -----");
            for (name, item) in variables {
                eprintln!("{} = {:#?}", name, item);
            }
            if !$skip {
                std::panic::resume_unwind(e);
            } else {
                eprintln!("----- SKIPPING -----")
            }
        }
    };
}

const fn always_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
struct OptionOverride {
    #[serde(default = "always_true")]
    episode: bool,
    #[serde(default = "always_true")]
    episode_title: bool,
    #[serde(default = "always_true")]
    file_checksum: bool,
    #[serde(default = "always_true")]
    file_extension: bool,
    #[serde(default = "always_true")]
    release_group: bool,
    #[serde(default = "always_true")]
    season: bool,
    #[serde(default = "always_true")]
    title: bool,
    #[serde(default = "always_true")]
    video_resolution: bool,
    #[serde(default = "always_true")]
    year: bool,
    #[serde(default = "always_true")]
    date: bool,
}

impl Default for OptionOverride {
    fn default() -> Self {
        Self {
            episode: true,
            episode_title: true,
            file_checksum: true,
            file_extension: true,
            release_group: true,
            season: true,
            title: true,
            video_resolution: true,
            year: true,
            date: true,
        }
    }
}

impl From<OptionOverride> for anitomy::Options {
    fn from(value: OptionOverride) -> Self {
        anitomy::Options::default()
            .episodes(value.episode)
            .episode_titles(value.episode_title)
            .file_checksums(value.file_checksum)
            .file_extensions(value.file_extension)
            .release_groups(value.release_group)
            .seasons(value.season)
            .titles(value.title)
            .video_resolutions(value.video_resolution)
            .years(value.year)
            .dates(value.date)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct InputData {
    input: String,
    #[serde(default)]
    skip: bool,
    output: HashMap<ElementKind, TestValue>,
    #[serde(default)]
    options: OptionOverride,
}

fn make_test_map(values: Vec<anitomy::Element<'_>>) -> HashMap<ElementKind, TestValue> {
    let mut result = HashMap::with_capacity(values.len());
    for value in values {
        result
            .entry(value.kind())
            .and_modify(|t: &mut TestValue| t.append(value.value()))
            .or_insert_with(|| TestValue::String(value.value().to_string()));
    }
    result
}

#[test]
fn test_json_data() {
    let data = include_str!("data.json");
    let tests: Vec<InputData> = serde_json::from_str(data).expect("could not parse JSON");
    let total = tests.len();

    for (index, mut test) in tests.into_iter().enumerate() {
        let input = test.input;
        let options = test.options.into();
        let keyword = KeywordConfig::default();
        let parsed = match std::panic::catch_unwind(|| {
            anitomy::parse_with_options_and_config(&input, options, &keyword)
        }) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("---- UNEXPECTED PANIC WHILE PARSING ---");
                eprintln!("input: {input:?}");
                std::panic::resume_unwind(e)
            }
        };
        test.output.remove(&ElementKind::EpisodeAlt); // this is untested
        let actual = make_test_map(parsed);
        for (key, expected) in test.output {
            let parsed = actual.get(&key);
            trace_variables!(
                input,
                key,
                index,
                total,
                expected,
                parsed,
                {
                    assert!(parsed.is_some());
                    assert_eq!(parsed.unwrap(), &expected);
                },
                test.skip
            );
        }
    }
}
