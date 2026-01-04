//! パーサーユーティリティ
//!
//! パース処理で共通して使用するユーティリティ関数です。

/// 文字列から数字を抽出（全角数字も対応）
pub fn extract_number(s: &str) -> Option<u32> {
    let digits: String = s
        .chars()
        .filter_map(|c| {
            if c.is_ascii_digit() {
                Some(c)
            } else if ('０'..='９').contains(&c) {
                // 全角数字をASCII数字に変換
                Some((c as u32 - '０' as u32 + '0' as u32) as u8 as char)
            } else {
                None
            }
        })
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("2字下げ"), Some(2));
        assert_eq!(extract_number("10字詰め"), Some(10));
        assert_eq!(extract_number("字下げ"), None);
    }

    #[test]
    fn test_extract_number_fullwidth() {
        assert_eq!(extract_number("２字下げ"), Some(2));
        assert_eq!(extract_number("３字下げ"), Some(3));
        assert_eq!(extract_number("１０字詰め"), Some(10));
    }
}
