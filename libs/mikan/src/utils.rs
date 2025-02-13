use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime};

pub fn smart_parse_date(date_str: &str) -> Result<NaiveDateTime> {
    // 先尝试解析完整的日期时间格式
    let datetime_layouts = vec![
        "%Y-%m-%dT%H:%M:%S%.f", // 2006-01-02T15:04:05.000
        "%Y-%m-%dT%H:%M:%S",    // 2006-01-02T15:04:05
        "%Y/%m/%dT%H:%M:%S%.f", // 2006/01/02T15:04:05.000
        "%Y/%m/%dT%H:%M:%S",    // 2006/01/02T15:04:05
        "%Y/%m/%d %H:%M",       // 2006/01/02 15:04
        "%Y/%m/%d %H:%M:%S",    // 2006/01/02 15:04:05
        "%Y/%m/%d %H:%M:%S%.f", // 2006/01/02 15:04:05.000
    ];

    // 尝试完整的日期时间格式
    for layout in datetime_layouts {
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, layout) {
            return Ok(dt);
        }
    }

    // 如果失败，尝试只解析日期部分
    let date_layouts = vec![
        "%Y-%m-%d", // 2006-01-02
        "%Y/%m/%d", // 2006/01/02
    ];

    for layout in date_layouts {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, layout) {
            // 将日期转换为日期时间，使用 00:00:00 作为默认时间
            return Ok(date.and_hms_opt(0, 0, 0).unwrap());
        }
    }

    Err(anyhow!("unable parse date: {}", date_str))
}

#[cfg(test)]

mod test {
    use super::*;
    #[test]
    fn test_parse_date() {
        smart_parse_date("2025/01/28").unwrap();
    }
}
