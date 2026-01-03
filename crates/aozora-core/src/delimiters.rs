//! 青空文庫形式のデリミタ定数
//!
//! 青空文庫形式で使用される全角デリミタ文字を定義

/// ルビ親文字開始 ｜ (U+FF5C)
pub const RUBY_PREFIX: char = '｜';

/// ルビ開始 《 (U+300A)
pub const RUBY_BEGIN: char = '《';

/// ルビ終了 》 (U+300B)
pub const RUBY_END: char = '》';

/// コマンド開始 ［ (U+FF3B)
pub const COMMAND_BEGIN: char = '［';

/// コマンド終了 ］ (U+FF3D)
pub const COMMAND_END: char = '］';

/// コマンド識別子 ＃ (U+FF03)
pub const IGETA: char = '＃';

/// 外字マーク ※ (U+203B)
pub const GAIJI_MARK: char = '※';

/// アクセント開始 〔 (U+3014)
pub const ACCENT_BEGIN: char = '〔';

/// アクセント終了 〕 (U+3015)
pub const ACCENT_END: char = '〕';

/// アクセント記号一覧
/// ' ` ^ ~ : & _ , / @
pub const ACCENT_MARKS: &[char] = &['\'', '`', '^', '~', ':', '&', '_', ',', '/', '@'];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter_unicode_values() {
        assert_eq!(RUBY_PREFIX as u32, 0xFF5C);
        assert_eq!(RUBY_BEGIN as u32, 0x300A);
        assert_eq!(RUBY_END as u32, 0x300B);
        assert_eq!(COMMAND_BEGIN as u32, 0xFF3B);
        assert_eq!(COMMAND_END as u32, 0xFF3D);
        assert_eq!(IGETA as u32, 0xFF03);
        assert_eq!(GAIJI_MARK as u32, 0x203B);
        assert_eq!(ACCENT_BEGIN as u32, 0x3014);
        assert_eq!(ACCENT_END as u32, 0x3015);
    }
}
