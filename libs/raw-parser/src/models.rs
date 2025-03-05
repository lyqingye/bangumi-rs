#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseResult {
    pub name_en: Option<String>,
    pub name_zh: Option<String>,
    pub name_jp: Option<String>,
    pub episode: Option<i32>,
    pub season: Option<i32>,
    pub subtitle_group: Option<String>,
    pub resolution: Option<String>,
    pub sub_type: Vec<String>,
}

// 中文数字映射
pub static CHINESE_NUMBER_MAP: phf::Map<&'static str, i32> = phf::phf_map! {
    "一" => 1,
    "二" => 2,
    "三" => 3,
    "四" => 4,
    "五" => 5,
    "六" => 6,
    "七" => 7,
    "八" => 8,
    "九" => 9,
    "十" => 10,
};
