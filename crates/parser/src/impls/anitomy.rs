#![allow(dead_code)]
use crate::{ParseResult, Parser};
use anitomy::KeywordConfig;
use anyhow::Result;
use async_trait::async_trait;

#[allow(dead_code)]
pub struct AnitomyParser {
    config: KeywordConfig,
}

impl AnitomyParser {
    pub fn new(config: KeywordConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Parser for AnitomyParser {
    async fn parse_file_names(&self, _file_names: Vec<String>) -> Result<Vec<ParseResult>> {
        // let mut results = Vec::new();
        // for file_name in file_names {
        //     let parse_result = anitomy::parse(&file_name);
        //     results.push(parse_result);
        // }
        Ok(vec![])
    }

    fn name(&self) -> String {
        "anitomy".to_string()
    }

    fn max_file_name_length(&self) -> usize {
        usize::MAX
    }
}

impl AnitomyParser {
    fn parse_file_name(&self, file_name: &str) -> Result<()> {
        let parse_result = anitomy::parse_with_options_and_config(
            file_name,
            anitomy::Options::default(),
            &self.config,
        );
        println!("{:?}", parse_result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_file_name() -> Result<()> {
        let parser = AnitomyParser::new(KeywordConfig::default());
        parser.parse_file_name("【幻樱字幕组】【1月新番】【魔法制造者 ~异世界魔法的制作方法~ Magic Maker ~Isekai Mahou no Tsukurikata~】【04】【BIG5_GB】【1920X1080】").unwrap();
        Ok(())
    }
}
