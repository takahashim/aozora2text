//! ブロック関連の型定義

use super::MidashiLevel;

/// ブロックタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// 字下げ
    Jisage,
    /// 地付き
    Chitsuki,
    /// 字詰め
    Jizume,
    /// 罫囲み
    Keigakomi,
    /// 見出し
    Midashi,
    /// 横組み
    Yokogumi,
    /// 太字
    Futoji,
    /// 斜体
    Shatai,
    /// 大きな文字
    FontDai,
    /// 小さな文字
    FontSho,
    /// 縦中横
    Tcy,
    /// キャプション
    Caption,
    /// 割り注
    Warigaki,
}

impl BlockType {
    /// コマンド名からブロックタイプを取得
    pub fn from_command(command: &str) -> Option<Self> {
        if command.contains("字下げ") {
            Some(BlockType::Jisage)
        } else if command.contains("地付き") || command.contains("地から") {
            Some(BlockType::Chitsuki)
        } else if command.contains("字詰め") {
            Some(BlockType::Jizume)
        } else if command.contains("罫囲み") {
            Some(BlockType::Keigakomi)
        } else if command.contains("見出し") {
            Some(BlockType::Midashi)
        } else if command.contains("横組み") {
            Some(BlockType::Yokogumi)
        } else if command.contains("太字") {
            Some(BlockType::Futoji)
        } else if command.contains("斜体") {
            Some(BlockType::Shatai)
        } else if command.contains("大きな文字") {
            Some(BlockType::FontDai)
        } else if command.contains("小さな文字") {
            Some(BlockType::FontSho)
        } else if command.contains("縦中横") {
            Some(BlockType::Tcy)
        } else if command.contains("キャプション") {
            Some(BlockType::Caption)
        } else if command.contains("割り注") {
            Some(BlockType::Warigaki)
        } else {
            None
        }
    }
}

/// ブロックパラメータ
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockParams {
    /// 幅（字下げの字数など）
    pub width: Option<u32>,
    /// 見出しレベル
    pub level: Option<MidashiLevel>,
    /// フォントサイズの段階
    pub font_size: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type_from_command() {
        assert_eq!(BlockType::from_command("2字下げ"), Some(BlockType::Jisage));
        assert_eq!(BlockType::from_command("地付き"), Some(BlockType::Chitsuki));
        assert_eq!(BlockType::from_command("太字"), Some(BlockType::Futoji));
    }
}
