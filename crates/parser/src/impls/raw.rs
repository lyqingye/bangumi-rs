use crate::{ParseResult, Parser};
use anyhow::Result;
use async_trait::async_trait;
use raw_parser::parser::Parser as RawParser;
use tracing::warn;

#[derive(Default)]
pub struct Raw {
    parser: RawParser,
}

impl Raw {
    pub fn new() -> Self {
        Self {
            parser: RawParser::new(),
        }
    }
}

#[async_trait]
impl Parser for Raw {
    async fn parse_file_names(&self, file_names: Vec<String>) -> Result<Vec<ParseResult>> {
        let mut results = Vec::new();
        for file_name in file_names {
            match self.parser.parse(&file_name) {
                Ok(parse_result) => {
                    results.push(ParseResult {
                        file_name,
                        release_group: parse_result.subtitle_group,
                        title: parse_result
                            .name_zh
                            .or(parse_result.name_jp)
                            .or(parse_result.name_en),
                        year: None,
                        season: parse_result.season,
                        episode: parse_result.episode,
                        video_resolution: parse_result
                            .resolution
                            .map(|resolution| resolution.as_str().into()),
                        languages: parse_result
                            .sub_type
                            .iter()
                            .map(|sub_type| sub_type.as_str().into())
                            .collect(),
                    });
                }
                Err(e) => {
                    warn!("解析文件名失败: {}", e);
                    continue;
                }
            }
        }
        Ok(results)
    }

    fn max_file_name_length(&self) -> usize {
        100
    }

    fn name(&self) -> String {
        "raw".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_file_names() -> Result<()> {
        let parser = Raw::new();
        let resp = parser.parse_file_names(vec![
        "[ANBU]_Princess_Lover!_-_01_[2048A39A].mkv".to_string(),
        "[ANBU-Menclave]_Canaan_-_01_[1024x576_H.264_AAC][12F00E89].mkv".to_string(),
        "[52wy][SlamDunk][001][Jpn_Chs_Cht][x264_aac][DVDRip][7FE2C873].mkv".to_string(),
        "[Yameii] NieR Automata Ver1.1a - S01E10 [English Dub] [CR WEB-DL 1080p] [3703AD3A]".to_string(),
        "[LoliHouse] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [WebRip 1080p HEVC-10bit AAC][简繁内封字幕]".to_string(),
        "[Skymoon-Raws] 孤nsingle人的异世界攻略 / Hitoribocchi no Isekai Kouryaku - 12 [ViuTV][WEB-DL][1080p][AVC AAC]".to_string(),
        "[ANi] Loner Life in Another World / 孤nsingle人的异世界攻略 - 12 [1080P][Baha][WEB-DL][AAC AVC][简日内嵌][MP4]".to_string(),
        "【幻樱字幕组】【1月新番】【魔法制造者 ~异世界魔法的制作方法~ Magic Maker ~Isekai Mahou no Tsukurikata~】【04】【BIG5】【1920X1080】".to_string()
        ]).await?;
        for result in resp {
            println!("{:?}", result);
        }
        Ok(())
    }
}
