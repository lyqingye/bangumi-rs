use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// 是否启用严格模式（在严格模式下，某些必要字段缺失会返回错误）
    #[serde(default)]
    pub strict_mode: bool,
    /// 自定义的正则表达式规则
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}
