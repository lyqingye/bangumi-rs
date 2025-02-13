use std::fmt;

macro_rules! dict_code {
    (
        $(
            // 定义分组
            group $group:ident {
                $(
                    // 每个枚举变体的定义：变体名 => (代码字符串, 描述, 排序值)
                    $variant:ident => ($code:expr, $desc:expr, $order:expr)
                ),* $(,)?
            }
        )*
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum DictCode {
            $(
                $(
                    $variant,
                )*
            )*
            Custom(String),
        }

        impl DictCode {
            pub fn group(&self) -> &'static str {
                match self {
                    $(
                        $(
                            DictCode::$variant => stringify!($group),
                        )*
                    )*
                    DictCode::Custom(_) => "custom"
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $(
                        $(
                            DictCode::$variant => $desc,
                        )*
                    )*
                    DictCode::Custom(_) => "自定义配置"
                }
            }

            pub fn sort_order(&self) -> i32 {
                match self {
                    $(
                        $(
                            DictCode::$variant => $order,
                        )*
                    )*
                    DictCode::Custom(_) => 1000
                }
            }
        }

        impl fmt::Display for DictCode {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        $(
                            DictCode::$variant => write!(f, $code),
                        )*
                    )*
                    DictCode::Custom(code) => write!(f, "{}", code)
                }
            }
        }

        impl From<String> for DictCode {
            fn from(s: String) -> Self {
                match s.as_str() {
                    $(
                        $(
                            $code => DictCode::$variant,
                        )*
                    )*
                    _ => DictCode::Custom(s)
                }
            }
        }
    };
}

// 使用宏定义所有字典代码
dict_code! {
    // 下载器相关配置
    group bangumi {
        CurrentSeasonSchedule => ("current_season_schedule", "当前季度番剧放送表", 100),
    }
}
