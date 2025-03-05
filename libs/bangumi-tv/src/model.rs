use std::fmt;

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug, Clone, Default)]
#[repr(u8)]
pub enum SubjectType {
    #[default]
    Book = 1,
    Anime = 2,
    Music = 3,
    Game = 4,
    Real = 6,
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
    pub platform: Option<Platform>,
    pub images: Images,
    pub image: Option<String>,
    pub eps: i32,
    pub total_episodes: i32,
    pub rating: Rating,
    pub collection: Collection,
    pub nsfw: bool,
    pub locked: bool,
    pub volumes: i32,
    pub meta_tags: Vec<String>,
    pub tags: Vec<Tag>,
    pub infobox: Vec<InfoboxItem>,
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

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Tag {
    pub name: String,
    pub count: i32,
    #[serde(rename = "total_cont")]
    pub total_count: i32,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct InfoboxItem {
    pub key: String,
    #[serde(default)]
    pub value: InfoboxValue,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum InfoboxValue {
    String(String),
    Array(Vec<InfoboxArrayItem>),
}

impl Default for InfoboxValue {
    fn default() -> Self {
        InfoboxValue::String(String::new())
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct InfoboxArrayItem {
    pub v: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SortType {
    #[default]
    Rank,
    Score,
    Name,
    AirDate,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SearchFilter {
    #[serde(default)]
    pub keyword: String,
    #[serde(default)]
    pub sort: SortType,
    #[serde(default)]
    pub filter: FilterCondition,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FilterCondition {
    #[serde(rename = "type", default)]
    pub subject_type: Vec<SubjectType>,
    #[serde(default)]
    pub meta_tags: Vec<String>,
    #[serde(default)]
    pub tag: Vec<String>,
    #[serde(default)]
    pub air_date: Vec<String>,
    #[serde(default)]
    pub rating: Vec<String>,
    #[serde(default)]
    pub rank: Vec<String>,
    #[serde(default)]
    pub nsfw: bool,
}

impl FilterCondition {
    pub fn add_start_date(&mut self, date: NaiveDate) {
        self.air_date.push(format!(">={}", date.format("%Y-%m-%d")));
    }
    pub fn add_end_date(&mut self, date: NaiveDate) {
        self.air_date.push(format!("<={}", date.format("%Y-%m-%d")));
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PageResponse<T> {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub data: Vec<T>,
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
            "cn": "星期日",
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

    #[test]
    fn test_search_filter_deserialize() {
        let json = r#"
        {
            "keyword": "string",
            "sort": "rank",
            "filter": {
                "type": [2],
                "meta_tags": ["童年", "原创"],
                "tag": ["童年", "原创"],
                "air_date": [">=2020-07-01", "<2020-10-01"],
                "rating": [">=6", "<8"],
                "rank": [">10", "<=18"],
                "nsfw": true
            }
        }
        "#;
        let res: SearchFilter = serde_json::from_str(json).unwrap();
        println!("{:?}", res);

        // 验证反序列化结果
        assert_eq!(res.keyword, "string");
        assert!(matches!(res.sort, SortType::Rank));
        assert_eq!(res.filter.subject_type, vec![SubjectType::Anime]);
        assert_eq!(res.filter.meta_tags, vec!["童年", "原创"]);
        assert_eq!(res.filter.tag, vec!["童年", "原创"]);
        assert_eq!(res.filter.air_date, vec![">=2020-07-01", "<2020-10-01"]);
        assert_eq!(res.filter.rating, vec![">=6", "<8"]);
        assert_eq!(res.filter.rank, vec![">10", "<=18"]);
        assert!(res.filter.nsfw);
    }

    #[test]
    fn test_legacy_subject_deserialize() {
        let json = r#"
        {
            "date": "2024-11-29",
            "platform": "剧场版",
            "images": {
                "small": "https://lain.bgm.tv/r/200/pic/cover/l/50/64/513314_XG0Y4.jpg",
                "grid": "https://lain.bgm.tv/r/100/pic/cover/l/50/64/513314_XG0Y4.jpg",
                "large": "https://lain.bgm.tv/pic/cover/l/50/64/513314_XG0Y4.jpg",
                "medium": "https://lain.bgm.tv/r/800/pic/cover/l/50/64/513314_XG0Y4.jpg",
                "common": "https://lain.bgm.tv/r/400/pic/cover/l/50/64/513314_XG0Y4.jpg"
            },
            "image": "https://lain.bgm.tv/pic/cover/l/50/64/513314_XG0Y4.jpg",
            "summary": "TVアニメ『俺だけレベルアップな件Season 2 -Arise from the Shadow-』の放送を記念して、\r\nアニメ『俺だけレベルアップな件 -ReAwakening-』の全世界劇場上映が決定！\r\n第１期の特別編集版と第２期の１・２話を先行上映。\r\n日本では11/29公開、海外では韓国にて11/28、その他地域も順次公開予定。\r\n続報をお楽しみに。",
            "name": "俺だけレベルアップな件 -ReAwakening-",
            "name_cn": "我独自升级 -ReAwakening-",
            "tags": [
                {
                    "name": "剧场版",
                    "count": 21,
                    "total_cont": 0
                },
                {
                    "name": "2024",
                    "count": 13,
                    "total_cont": 0
                }
            ],
            "infobox": [
                {
                    "key": "中文名",
                    "value": "我独自升级 -ReAwakening-"
                },
                {
                    "key": "别名",
                    "value": [
                        {
                            "v": "나 혼자만 레벨업 -ReAwakening-"
                        },
                        {
                            "v": "Solo Leveling -ReAwakening-"
                        }
                    ]
                }
            ],
            "rating": {
                "rank": 0,
                "total": 39,
                "count": {
                    "1": 0,
                    "2": 0,
                    "3": 0,
                    "4": 1,
                    "5": 3,
                    "6": 13,
                    "7": 13,
                    "8": 6,
                    "9": 1,
                    "10": 2
                },
                "score": 6.8
            },
            "collection": {
                "on_hold": 13,
                "dropped": 8,
                "wish": 104,
                "collect": 94,
                "doing": 34
            },
            "id": 513314,
            "eps": 1,
            "meta_tags": [
                "剧场版",
                "日本",
                "奇幻",
                "战斗"
            ],
            "volumes": 0,
            "series": false,
            "locked": false,
            "nsfw": false,
            "type": 2
        }
        "#;
        let res: Subject = serde_json::from_str(json).unwrap();
        println!("{:?}", res);

        // 验证反序列化结果
        assert_eq!(res.id, 513314);
        assert_eq!(res.name, "俺だけレベルアップな件 -ReAwakening-");
        assert_eq!(res.name_cn, Some("我独自升级 -ReAwakening-".to_string()));
        assert_eq!(res.eps, 1);
        assert_eq!(res.volumes, 0);
        assert_eq!(res.meta_tags, vec!["剧场版", "日本", "奇幻", "战斗"]);
        assert!(!res.nsfw);
        assert!(!res.locked);
        assert!(!res.series);
        assert_eq!(res.subject_type, SubjectType::Anime);
        assert_eq!(res.platform, Some(Platform::Movie));

        // 验证标签
        assert_eq!(res.tags.len(), 2);
        assert_eq!(res.tags[0].name, "剧场版");
        assert_eq!(res.tags[0].count, 21);

        // 验证信息框
        assert_eq!(res.infobox.len(), 2);
        assert_eq!(res.infobox[0].key, "中文名");
        match &res.infobox[0].value {
            InfoboxValue::String(s) => assert_eq!(s, "我独自升级 -ReAwakening-"),
            _ => panic!("Expected String value"),
        }
    }
}
