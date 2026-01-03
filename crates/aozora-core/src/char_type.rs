//! 文字種別判定
//!
//! ルビの親文字を自動抽出する際に使用する文字種別判定機能を提供します。
//!
//! # 文字種別一覧
//!
//! | 種別 | 説明 |
//! |------|------|
//! | Hiragana | ひらがな（ぁ-ん、ゝ、ゞ） |
//! | Katakana | カタカナ（ァ-ン、ー、ヽ、ヾ、ヴ） |
//! | Zenkaku | 全角英数・ギリシャ・キリル文字 |
//! | Hankaku | 半角英数と一部記号 |
//! | Kanji | CJK統合漢字と特殊文字（々、※、仝、〆、〇、ヶ） |
//! | HankakuTerminate | 半角終端記号（.;"?!)） |
//! | Else | その他（句読点、括弧など） |

/// 文字種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharType {
    /// ひらがな（ぁ-ん、ゝ、ゞ）
    Hiragana,
    /// カタカナ（ァ-ン、ー、ヽ、ヾ、ヴ）
    Katakana,
    /// 全角英数・ギリシャ・キリル文字
    Zenkaku,
    /// 半角英数と一部記号
    Hankaku,
    /// CJK統合漢字と特殊文字
    Kanji,
    /// 半角終端記号
    HankakuTerminate,
    /// その他
    Else,
}

impl CharType {
    /// 文字種別を判定
    ///
    /// # Examples
    ///
    /// ```
    /// use aozora_core::char_type::CharType;
    ///
    /// assert_eq!(CharType::classify('あ'), CharType::Hiragana);
    /// assert_eq!(CharType::classify('ア'), CharType::Katakana);
    /// assert_eq!(CharType::classify('漢'), CharType::Kanji);
    /// assert_eq!(CharType::classify('A'), CharType::Hankaku);
    /// assert_eq!(CharType::classify('Ａ'), CharType::Zenkaku);
    /// assert_eq!(CharType::classify('.'), CharType::HankakuTerminate);
    /// assert_eq!(CharType::classify('。'), CharType::Else);
    /// ```
    pub fn classify(c: char) -> Self {
        // ひらがな: ぁ-ん (U+3041-U+3093) + ゝゞ (U+309D-U+309E)
        if matches!(c, 'ぁ'..='ん' | 'ゝ' | 'ゞ') {
            return CharType::Hiragana;
        }

        // カタカナ: ァ-ン (U+30A1-U+30F3) + ー (U+30FC) + ヽヾ (U+30FD-U+30FE) + ヴ (U+30F4)
        if matches!(c, 'ァ'..='ン' | 'ー' | 'ヽ' | 'ヾ' | 'ヴ') {
            return CharType::Katakana;
        }

        // 全角英数: ０-９ (U+FF10-U+FF19), Ａ-Ｚ (U+FF21-U+FF3A), ａ-ｚ (U+FF41-U+FF5A)
        // + ギリシャ大文字 Α-Ω (U+0391-U+03A9), 小文字 α-ω (U+03B1-U+03C9)
        // + キリル大文字 А-Я (U+0410-U+042F), 小文字 а-я (U+0430-U+044F)
        // + 記号 − (U+2212), ＆ (U+FF06), ' (U+2019), ， (U+FF0C), ． (U+FF0E)
        if matches!(c,
            '０'..='９' | 'Ａ'..='Ｚ' | 'ａ'..='ｚ' |
            'Α'..='Ω' | 'α'..='ω' |
            'А'..='Я' | 'а'..='я' |
            '−' | '＆' | '\u{2019}' | '，' | '．'
        ) {
            return CharType::Zenkaku;
        }

        // 半角英数: A-Z, a-z, 0-9, #, -, &, ', ,
        if matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '#' | '-' | '&' | '\'' | ',') {
            return CharType::Hankaku;
        }

        // 漢字: CJK統合漢字 (U+4E00-U+9FFF) + 特殊文字
        // 々 (U+3005), ※ (U+203B), 〆 (U+3006), 〇 (U+3007), ヶ (U+30F6)
        // 注: 仝 (U+4EDD) はCJK範囲内なので別途指定不要
        if matches!(c, '\u{4E00}'..='\u{9FFF}' | '々' | '※' | '〆' | '〇' | 'ヶ') {
            return CharType::Kanji;
        }

        // 半角終端記号: . ; " ? ! )
        if matches!(c, '.' | ';' | '"' | '?' | '!' | ')') {
            return CharType::HankakuTerminate;
        }

        // その他
        CharType::Else
    }

    /// この種別がルビ親文字になれるかどうか
    ///
    /// `:else` 以外の種別はルビ親文字になれる
    pub fn can_be_ruby_base(&self) -> bool {
        !matches!(self, CharType::Else)
    }
}

/// 文字種別を取得する拡張トレイト
pub trait CharTypeExt {
    /// 文字種別を取得
    fn char_type(&self) -> CharType;
}

impl CharTypeExt for char {
    fn char_type(&self) -> CharType {
        CharType::classify(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hiragana() {
        assert_eq!(CharType::classify('あ'), CharType::Hiragana);
        assert_eq!(CharType::classify('ん'), CharType::Hiragana);
        assert_eq!(CharType::classify('ゝ'), CharType::Hiragana);
        assert_eq!(CharType::classify('ゞ'), CharType::Hiragana);
    }

    #[test]
    fn test_katakana() {
        assert_eq!(CharType::classify('ア'), CharType::Katakana);
        assert_eq!(CharType::classify('ン'), CharType::Katakana);
        assert_eq!(CharType::classify('ー'), CharType::Katakana);
        assert_eq!(CharType::classify('ヽ'), CharType::Katakana);
        assert_eq!(CharType::classify('ヾ'), CharType::Katakana);
        assert_eq!(CharType::classify('ヴ'), CharType::Katakana);
    }

    #[test]
    fn test_zenkaku() {
        assert_eq!(CharType::classify('Ａ'), CharType::Zenkaku);
        assert_eq!(CharType::classify('ａ'), CharType::Zenkaku);
        assert_eq!(CharType::classify('０'), CharType::Zenkaku);
        assert_eq!(CharType::classify('９'), CharType::Zenkaku);
        // ギリシャ文字
        assert_eq!(CharType::classify('Α'), CharType::Zenkaku);
        assert_eq!(CharType::classify('α'), CharType::Zenkaku);
        // キリル文字
        assert_eq!(CharType::classify('А'), CharType::Zenkaku);
        assert_eq!(CharType::classify('а'), CharType::Zenkaku);
    }

    #[test]
    fn test_hankaku() {
        assert_eq!(CharType::classify('A'), CharType::Hankaku);
        assert_eq!(CharType::classify('z'), CharType::Hankaku);
        assert_eq!(CharType::classify('0'), CharType::Hankaku);
        assert_eq!(CharType::classify('9'), CharType::Hankaku);
        assert_eq!(CharType::classify('#'), CharType::Hankaku);
        assert_eq!(CharType::classify('-'), CharType::Hankaku);
        assert_eq!(CharType::classify('&'), CharType::Hankaku);
        assert_eq!(CharType::classify('\''), CharType::Hankaku);
        assert_eq!(CharType::classify(','), CharType::Hankaku);
    }

    #[test]
    fn test_kanji() {
        assert_eq!(CharType::classify('漢'), CharType::Kanji);
        assert_eq!(CharType::classify('字'), CharType::Kanji);
        assert_eq!(CharType::classify('々'), CharType::Kanji);
        assert_eq!(CharType::classify('※'), CharType::Kanji);
        assert_eq!(CharType::classify('仝'), CharType::Kanji);
        assert_eq!(CharType::classify('〆'), CharType::Kanji);
        assert_eq!(CharType::classify('〇'), CharType::Kanji);
        assert_eq!(CharType::classify('ヶ'), CharType::Kanji);
    }

    #[test]
    fn test_hankaku_terminate() {
        assert_eq!(CharType::classify('.'), CharType::HankakuTerminate);
        assert_eq!(CharType::classify(';'), CharType::HankakuTerminate);
        assert_eq!(CharType::classify('"'), CharType::HankakuTerminate);
        assert_eq!(CharType::classify('?'), CharType::HankakuTerminate);
        assert_eq!(CharType::classify('!'), CharType::HankakuTerminate);
        assert_eq!(CharType::classify(')'), CharType::HankakuTerminate);
    }

    #[test]
    fn test_else() {
        assert_eq!(CharType::classify('。'), CharType::Else);
        assert_eq!(CharType::classify('、'), CharType::Else);
        assert_eq!(CharType::classify('「'), CharType::Else);
        assert_eq!(CharType::classify('」'), CharType::Else);
        assert_eq!(CharType::classify('（'), CharType::Else);
        assert_eq!(CharType::classify('）'), CharType::Else);
    }

    #[test]
    fn test_can_be_ruby_base() {
        assert!(CharType::Hiragana.can_be_ruby_base());
        assert!(CharType::Katakana.can_be_ruby_base());
        assert!(CharType::Zenkaku.can_be_ruby_base());
        assert!(CharType::Hankaku.can_be_ruby_base());
        assert!(CharType::Kanji.can_be_ruby_base());
        assert!(CharType::HankakuTerminate.can_be_ruby_base());
        assert!(!CharType::Else.can_be_ruby_base());
    }

    #[test]
    fn test_char_type_ext() {
        assert_eq!('あ'.char_type(), CharType::Hiragana);
        assert_eq!('ア'.char_type(), CharType::Katakana);
        assert_eq!('漢'.char_type(), CharType::Kanji);
    }

    #[test]
    fn test_edge_case_ke() {
        // ヶは漢字として扱う（青空文庫の指針）
        assert_eq!(CharType::classify('ヶ'), CharType::Kanji);
    }

    #[test]
    fn test_edge_case_long_vowel() {
        // 長音記号はカタカナとして扱う
        assert_eq!(CharType::classify('ー'), CharType::Katakana);
    }
}
