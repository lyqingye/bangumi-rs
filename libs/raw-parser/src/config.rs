use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 是否启用严格模式（在严格模式下，某些必要字段缺失会返回错误）
    pub strict_mode: bool,
    /// 自定义的正则表达式规则
    pub custom_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            strict_mode: false,
            custom_patterns: Vec::new(),
        }
    }
}
