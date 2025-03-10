pub mod models;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use serde::Deserialize;
    use std::fs;

    #[derive(Debug, Deserialize)]
    struct ExpectedResult {
        name_en: Option<String>,
        name_zh: Option<String>,
        name_jp: Option<String>,
        episode: Option<i32>,
        season: Option<i32>,
        subtitle_group: Option<String>,
        resolution: Option<String>,
        sub_type: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct TestCase {
        description: String,
        input: String,
        expected: ExpectedResult,
    }

    #[derive(Debug, Deserialize)]
    struct TestData {
        success_cases: Vec<TestCase>,
    }

    #[test]
    fn test_example() {
        let parser = Parser::new();
        let result = parser.parse("[ANi]  超超超超超喜欢你的 100 个女朋友 - 21 [1080P][Baha][WEB-DL][AAC AVC][CHT][MP4]").unwrap();
        println!("Result: {:?}", result);
    }

    #[test]
    fn test_parser() {
        let parser = Parser::new();

        // 读取测试数据
        let test_data_str =
            fs::read_to_string("testdata/test_cases.json").expect("Failed to read test data file");
        let test_data: TestData =
            serde_json::from_str(&test_data_str).expect("Failed to parse test data");

        // 测试成功用例
        for case in test_data.success_cases {
            let result = parser
                .parse(&case.input)
                .unwrap_or_else(|_| panic!("Failed to parse: {}", case.description));

            assert_eq!(
                result.name_en, case.expected.name_en,
                "Failed on '{}' - name_en mismatch",
                case.description
            );
            assert_eq!(
                result.name_zh, case.expected.name_zh,
                "Failed on '{}' - name_zh mismatch",
                case.description
            );
            assert_eq!(
                result.name_jp, case.expected.name_jp,
                "Failed on '{}' - name_jp mismatch",
                case.description
            );
            assert_eq!(
                result.episode, case.expected.episode,
                "Failed on '{}' - episode mismatch",
                case.description
            );
            assert_eq!(
                result.season, case.expected.season,
                "Failed on '{}' - season mismatch",
                case.description
            );
            assert_eq!(
                result.subtitle_group, case.expected.subtitle_group,
                "Failed on '{}' - subtitle_group mismatch",
                case.description
            );
            assert_eq!(
                result.resolution, case.expected.resolution,
                "Failed on '{}' - resolution mismatch",
                case.description
            );
            assert_eq!(
                result.sub_type, case.expected.sub_type,
                "Failed on '{}' - sub_type mismatch",
                case.description
            );
        }
    }
}
