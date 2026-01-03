//! 装飾タイプ定義

/// 装飾タイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleType {
    // 傍点系
    SesameDot,
    WhiteSesameDot,
    BlackCircle,
    WhiteCircle,
    BlackTriangle,
    WhiteTriangle,
    Bullseye,
    Fisheye,
    Saltire,

    // 傍線系
    UnderlineSolid,
    UnderlineDouble,
    UnderlineDotted,
    UnderlineDashed,
    UnderlineWave,

    // 文字スタイル
    Bold,
    Italic,
    Subscript,
    Superscript,
}

impl StyleType {
    /// コマンド名から装飾タイプを取得
    pub fn from_command(command: &str) -> Option<Self> {
        match command {
            "傍点" => Some(StyleType::SesameDot),
            "白ゴマ傍点" => Some(StyleType::WhiteSesameDot),
            "丸傍点" => Some(StyleType::BlackCircle),
            "白丸傍点" => Some(StyleType::WhiteCircle),
            "黒三角傍点" => Some(StyleType::BlackTriangle),
            "白三角傍点" => Some(StyleType::WhiteTriangle),
            "二重丸傍点" => Some(StyleType::Bullseye),
            "蛇の目傍点" => Some(StyleType::Fisheye),
            "ばつ傍点" => Some(StyleType::Saltire),
            "傍線" => Some(StyleType::UnderlineSolid),
            "二重傍線" => Some(StyleType::UnderlineDouble),
            "鎖線" => Some(StyleType::UnderlineDotted),
            "破線" => Some(StyleType::UnderlineDashed),
            "波線" => Some(StyleType::UnderlineWave),
            "太字" => Some(StyleType::Bold),
            "斜体" => Some(StyleType::Italic),
            "下付き小文字" | "行左小書き" => Some(StyleType::Subscript),
            "上付き小文字" | "行右小書き" => Some(StyleType::Superscript),
            _ => None,
        }
    }

    /// コマンド名を取得
    pub fn command_name(&self) -> &'static str {
        match self {
            StyleType::SesameDot => "傍点",
            StyleType::WhiteSesameDot => "白ゴマ傍点",
            StyleType::BlackCircle => "丸傍点",
            StyleType::WhiteCircle => "白丸傍点",
            StyleType::BlackTriangle => "黒三角傍点",
            StyleType::WhiteTriangle => "白三角傍点",
            StyleType::Bullseye => "二重丸傍点",
            StyleType::Fisheye => "蛇の目傍点",
            StyleType::Saltire => "ばつ傍点",
            StyleType::UnderlineSolid => "傍線",
            StyleType::UnderlineDouble => "二重傍線",
            StyleType::UnderlineDotted => "鎖線",
            StyleType::UnderlineDashed => "破線",
            StyleType::UnderlineWave => "波線",
            StyleType::Bold => "太字",
            StyleType::Italic => "斜体",
            StyleType::Subscript => "下付き小文字",
            StyleType::Superscript => "上付き小文字",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_type_from_command() {
        assert_eq!(StyleType::from_command("傍点"), Some(StyleType::SesameDot));
        assert_eq!(StyleType::from_command("太字"), Some(StyleType::Bold));
        assert_eq!(StyleType::from_command("未知"), None);
    }
}
