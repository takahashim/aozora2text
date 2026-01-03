//! アクセント分解記法の変換

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::gaiji::jis_to_unicode;

/// アクセント記号一覧
const ACCENT_MARKS: &[char] = &['\'', '`', '^', '~', ':', '&', '_', ',', '/', '@'];

/// アクセントテーブル（基底文字+記号 → JISコード）
static ACCENT_TABLE: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| include!(concat!(env!("OUT_DIR"), "/accent_table.rs")));

/// アクセント分解記法を変換
///
/// `cafe'` → `café` のように、基底文字+アクセント記号を
/// アクセント付き文字に変換する。
///
/// # Examples
///
/// ```
/// use aozora2text::accent::convert_accent;
///
/// assert_eq!(convert_accent("cafe'"), "café");
/// assert_eq!(convert_accent("A'"), "Á");
/// ```
pub fn convert_accent(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        // 3文字のリガチャをチェック (例: "ae&" → æ)
        if i + 2 < chars.len() && is_accent_mark(chars[i + 2]) {
            let key = format!("{}{}{}", chars[i], chars[i + 1], chars[i + 2]);
            if let Some(converted) = lookup_accent(&key) {
                result.push_str(&converted);
                i += 3;
                continue;
            }
        }

        // 2文字のアクセントをチェック (例: "e'" → é)
        if i + 1 < chars.len() && is_accent_mark(chars[i + 1]) {
            let key = format!("{}{}", chars[i], chars[i + 1]);
            if let Some(converted) = lookup_accent(&key) {
                result.push_str(&converted);
                i += 2;
                continue;
            }
        }

        // マッチしない場合はそのまま出力
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// 文字がアクセント記号かどうか
fn is_accent_mark(c: char) -> bool {
    ACCENT_MARKS.contains(&c)
}

/// アクセントテーブルを検索してUnicode文字を返す
fn lookup_accent(key: &str) -> Option<String> {
    ACCENT_TABLE
        .get(key)
        .and_then(|jis_code| jis_to_unicode(jis_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_accent() {
        assert_eq!(convert_accent("e'"), "é");
        assert_eq!(convert_accent("a`"), "à");
        assert_eq!(convert_accent("u:"), "ü");
    }

    #[test]
    fn test_word_with_accent() {
        assert_eq!(convert_accent("cafe'"), "café");
        assert_eq!(convert_accent("nai:ve"), "naïve");
    }

    #[test]
    fn test_uppercase() {
        assert_eq!(convert_accent("A'"), "Á");
        assert_eq!(convert_accent("E`"), "È");
    }

    #[test]
    fn test_ligature() {
        assert_eq!(convert_accent("ae&"), "æ");
        assert_eq!(convert_accent("AE&"), "Æ");
    }

    #[test]
    fn test_no_accent() {
        assert_eq!(convert_accent("hello"), "hello");
        assert_eq!(convert_accent("test"), "test");
    }

    #[test]
    fn test_unknown_combination() {
        // 未知の組み合わせはそのまま
        assert_eq!(convert_accent("z'"), "z'");
    }
}
