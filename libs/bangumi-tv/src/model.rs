use std::fmt;

use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum IntOrFloat {
    Int(i32),
    Float(f32),
}

impl Default for IntOrFloat {
    fn default() -> Self {
        IntOrFloat::Int(0)
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct CalendarResponse {
    #[serde(default)]
    pub weekday: Weekday,
    #[serde(default)]
    pub items: Vec<LegacySubjectSmall>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Weekday {
    #[serde(default)]
    pub en: EnglishDay,
    #[serde(default)]
    pub cn: ChineseDay,
    #[serde(default)]
    pub ja: JapaneseDay,
    #[serde(default)]
    pub id: i32,
}

#[derive(Debug, Deserialize, Default)]
pub enum EnglishDay {
    #[serde(rename = "Mon")]
    #[default]
    Monday,
    #[serde(rename = "Tue")]
    Tuesday,
    #[serde(rename = "Wed")]
    Wednesday,
    #[serde(rename = "Thu")]
    Thursday,
    #[serde(rename = "Fri")]
    Friday,
    #[serde(rename = "Sat")]
    Saturday,
    #[serde(rename = "Sun")]
    Sunday,
}

#[derive(Debug, Deserialize, Default)]
pub enum ChineseDay {
    #[serde(rename = "星期一")]
    #[default]
    Monday,
    #[serde(rename = "星期二")]
    Tuesday,
    #[serde(rename = "星期三")]
    Wednesday,
    #[serde(rename = "星期四")]
    Thursday,
    #[serde(rename = "星期五")]
    Friday,
    #[serde(rename = "星期六")]
    Saturday,
    #[serde(rename = "星期日")]
    Sunday,
}

#[derive(Debug, Deserialize, Default)]
pub enum JapaneseDay {
    #[serde(rename = "月耀日")]
    #[default]
    Monday,
    #[serde(rename = "火耀日")]
    Tuesday,
    #[serde(rename = "水耀日")]
    Wednesday,
    #[serde(rename = "木耀日")]
    Thursday,
    #[serde(rename = "金耀日")]
    Friday,
    #[serde(rename = "土耀日")]
    Saturday,
    #[serde(rename = "日耀日")]
    Sunday,
}

#[derive(Debug, Deserialize, Default)]
pub struct LegacySubjectSmall {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub url: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub subject_type: SubjectType,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_cn: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub air_date: Option<NaiveDate>,
    #[serde(default)]
    pub air_weekday: i32,
    #[serde(default)]
    pub images: Images,
    #[serde(default)]
    pub eps: i32,
    #[serde(default)]
    pub eps_count: i32,
    #[serde(default)]
    pub rating: Rating,
    #[serde(default)]
    pub rank: i32,
    #[serde(default)]
    pub collection: Collection,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Images {
    pub large: String,
    pub common: String,
    pub medium: String,
    pub small: String,
    pub grid: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Rating {
    pub total: i32,
    pub count: RatingCount,
    pub score: f64,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct RatingCount {
    #[serde(rename = "1")]
    pub count_1: i32,
    #[serde(rename = "2")]
    pub count_2: i32,
    #[serde(rename = "3")]
    pub count_3: i32,
    #[serde(rename = "4")]
    pub count_4: i32,
    #[serde(rename = "5")]
    pub count_5: i32,
    #[serde(rename = "6")]
    pub count_6: i32,
    #[serde(rename = "7")]
    pub count_7: i32,
    #[serde(rename = "8")]
    pub count_8: i32,
    #[serde(rename = "9")]
    pub count_9: i32,
    #[serde(rename = "10")]
    pub count_10: i32,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Collection {
    pub wish: i32,
    pub collect: i32,
    pub doing: i32,
    pub on_hold: i32,
    pub dropped: i32,
}

#[derive(Deserialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum SubjectType {
    #[default]
    Anime = 1,
    Manga = 2,
    Book = 3,
    Game = 4,
    Music = 5,
    Movie = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize_repr, Default)]
#[serde(default)]
#[repr(u8)]
pub enum EpisodeType {
    #[default]
    Normal = 0,
    Special = 1,
    Opening = 2,
    Ending = 3,
    Mad = 4,
    Other = 6,
}

impl fmt::Display for EpisodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpisodeType::Normal => write!(f, "EP"),
            EpisodeType::Special => write!(f, "SP"),
            EpisodeType::Opening => write!(f, "OP"),
            EpisodeType::Ending => write!(f, "ED"),
            EpisodeType::Mad => write!(f, "MAD"),
            EpisodeType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Pagination {
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Episode {
    pub id: i32,
    #[serde(rename = "type")]
    pub ep_type: EpisodeType,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub name_cn: Option<String>,
    pub ep: IntOrFloat,
    pub sort: IntOrFloat,
    #[serde(default, deserialize_with = "parse_date_as_option")]
    pub airdate: Option<NaiveDate>,
    pub comment: i32,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub desc: Option<String>,
    pub disc: i32,
    pub duration: String,
    pub duration_seconds: u64,
}

impl Episode {
    /// FIXME： 这里需要支持小数类型的剧集Number
    pub fn get_ep(&self) -> Option<i32> {
        match self.sort {
            IntOrFloat::Int(i) => Some(i),
            _ => match self.ep {
                IntOrFloat::Int(i) => Some(i),
                _ => None,
            },
        }
    }
}
fn parse_date_as_option<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let date_str: String = String::deserialize(deserializer)?;
    if date_str.is_empty() {
        return Ok(None);
    }

    // 尝试不同的日期格式
    let date_layouts = vec![
        "%Y-%m-%d", // 2006-01-02
        "%Y/%m/%d", // 2006/01/02
    ];

    for layout in date_layouts {
        if let Ok(date) = NaiveDate::parse_from_str(&date_str, layout) {
            return Ok(Some(date));
        }
    }

    // 如果所有格式都解析失败，返回 None
    Ok(None)
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct EpisodeList {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub data: Vec<Episode>,
}

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Subject {
    pub id: i32,
    #[serde(rename = "type")]
    pub subject_type: SubjectType,
    pub name: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub name_cn: Option<String>,
    pub summary: String,
    pub series: bool,
    pub date: Option<NaiveDate>,
    pub platform: Platform,
    pub images: Images,
    pub eps: i32,
    pub total_episodes: i32,
    pub rating: Rating,
    pub collection: Collection,
}
impl Subject {
    pub fn get_air_date(&self) -> Option<NaiveDateTime> {
        if let Some(date) = self.date {
            return date.and_hms_opt(0, 0, 0);
        }
        None
    }
    pub fn get_eps(&self) -> i32 {
        if self.eps != 0 {
            self.eps
        } else {
            self.total_episodes
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub enum Platform {
    #[serde(rename = "TV")]
    TV,
    #[serde(rename = "Web")]
    Web,
    #[serde(rename = "DLC")]
    DLC,
    #[serde(rename = "剧场版")]
    Movie,
    #[serde(other)]
    #[default]
    Unknown,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_week_deserialize() {
        let json = r#"
    [
        {
            "en": "Mon",
            "cn": "星期一",
            "ja": "月耀日",
            "id": 1
        },
        {
            "en": "Tue",
            "cn": "星期二",
            "ja": "火耀日",
            "id": 2
        },
        {
            "en": "Wed",
            "cn": "星期三",
            "ja": "水耀日",
            "id": 3
        },
        {
            "en": "Thu",
            "cn": "星期四",
            "ja": "木耀日",
            "id": 4
        },
        {
            "en": "Fri",
            "cn": "星期五",
            "ja": "金耀日",
            "id": 5
        },
        {
            "en": "Sat",
            "cn": "星期六",
            "ja": "土耀日",
            "id": 6
        },
        {
            "en": "Sun",
            "cn": "星期天",
            "ja": "日耀日",
            "id": 7
        }
    ]
        "#;
        let res: Vec<Weekday> = serde_json::from_str(json).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_rating_deserialize() {
        let json = r#"
                {
                    "total": 8,
                    "count": {
                        "1": 0,
                        "2": 0,
                        "3": 0,
                        "4": 1,
                        "5": 0,
                        "6": 5,
                        "7": 0,
                        "8": 2,
                        "9": 0,
                        "10": 0
                    },
                    "score": 6.3
                }
        "#;
        let res: RatingCount = serde_json::from_str(json).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_image_deserialize() {
        let json = r#"
            {
                    "large": "http://lain.bgm.tv/pic/cover/l/61/09/533096_n9lJ8.jpg",
                    "common": "http://lain.bgm.tv/pic/cover/c/61/09/533096_n9lJ8.jpg",
                    "medium": "http://lain.bgm.tv/pic/cover/m/61/09/533096_n9lJ8.jpg",
                    "small": "http://lain.bgm.tv/pic/cover/s/61/09/533096_n9lJ8.jpg",
                    "grid": "http://lain.bgm.tv/pic/cover/g/61/09/533096_n9lJ8.jpg"
                }
        "#;
        let res: Images = serde_json::from_str(json).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_subject_deserialize() {
        let json = r#"
        {
                "type": 2,
                "name": "猫猫のひとりごと 第2期",
                "name_cn": "猫猫的呢喃 第二季",
                "summary": "",
                "air_weekday": 1,
                "rating": {
                    "total": 8,
                    "count": {
                        "1": 0,
                        "2": 0,
                        "3": 0,
                        "4": 1,
                        "5": 0,
                        "6": 5,
                        "7": 0,
                        "8": 2,
                        "9": 0,
                        "10": 0
                    },
                    "score": 6.3
                }
            }
        "#;
        let res: LegacySubjectSmall = serde_json::from_str(json).unwrap();
        println!("{:?}", res);
    }
}
