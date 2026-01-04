//! HTMLレンダラー
//!
//! ASTノードをHTMLに変換します。

use aozora_core::document::{
    extract_bibliographical_lines, extract_body_lines, extract_header_info, HeaderInfo,
};
use aozora_core::gaiji::{parse_gaiji, GaijiResult};
use aozora_core::node::{
    BlockParams, BlockType, MidashiLevel, MidashiStyle, Node, RubyDirection, StyleType,
};
use aozora_core::parser::parse;
use aozora_core::parser::reference_resolver::resolve_inline_ruby;
use aozora_core::tokenizer::tokenize;

use super::options::RenderOptions;

/// 青空文庫パブリッシャー名
const AOZORA_BUNKO: &str = "青空文庫";

// ============================================================================
// プレゼンテーションロジック（CSSクラス、HTMLタグのマッピング）
// ============================================================================

/// StyleType のCSSクラス名を取得
fn style_css_class(style_type: StyleType) -> &'static str {
    match style_type {
        StyleType::SesameDot => "sesame_dot",
        StyleType::WhiteSesameDot => "white_sesame_dot",
        StyleType::BlackCircle => "black_circle",
        StyleType::WhiteCircle => "white_circle",
        StyleType::BlackTriangle => "black_up-pointing_triangle",
        StyleType::WhiteTriangle => "white_up-pointing_triangle",
        StyleType::Bullseye => "bullseye",
        StyleType::Fisheye => "fisheye",
        StyleType::Saltire => "saltire",
        StyleType::UnderlineSolid => "underline_solid",
        StyleType::UnderlineDouble => "underline_double",
        StyleType::UnderlineDotted => "underline_dotted",
        StyleType::UnderlineDashed => "underline_dashed",
        StyleType::UnderlineWave => "underline_wave",
        StyleType::Bold => "futoji",
        StyleType::Italic => "shatai",
        StyleType::Subscript => "subscript",
        StyleType::Superscript => "superscript",
    }
}

/// StyleType のHTMLタグ名を取得
fn style_html_tag(style_type: StyleType) -> &'static str {
    match style_type {
        StyleType::Subscript => "sub",
        StyleType::Superscript => "sup",
        StyleType::Bold | StyleType::Italic => "span",
        _ => "em",
    }
}

/// MidashiLevel のCSSクラス名を取得
fn midashi_css_class(level: MidashiLevel) -> &'static str {
    match level {
        MidashiLevel::O => "o-midashi",
        MidashiLevel::Naka => "naka-midashi",
        MidashiLevel::Ko => "ko-midashi",
    }
}

/// MidashiLevel のHTMLタグ名を取得
fn midashi_html_tag(level: MidashiLevel) -> &'static str {
    match level {
        MidashiLevel::O => "h3",
        MidashiLevel::Naka => "h4",
        MidashiLevel::Ko => "h5",
    }
}

/// HTMLレンダラー
#[derive(Debug, Clone)]
pub struct HtmlRenderer {
    options: RenderOptions,
    /// 現在のブロックスタック
    block_stack: Vec<BlockContext>,
    /// 見出しIDカウンター
    midashi_id_counter: u32,
    /// 注記を使用したかどうか
    has_notes: bool,
    /// 外字画像を使用したかどうか
    has_gaiji_images: bool,
    /// アクセント記号を使用したかどうか
    has_accent: bool,
    /// JIS X 0213文字を使用したかどうか
    has_jisx0213: bool,
}

/// ブロックコンテキスト
#[derive(Debug, Clone)]
struct BlockContext {
    block_type: BlockType,
    params: BlockParams,
}

impl HtmlRenderer {
    /// 新しいレンダラーを作成
    pub fn new(options: RenderOptions) -> Self {
        Self {
            options,
            block_stack: Vec::new(),
            midashi_id_counter: 100,
            has_notes: false,
            has_gaiji_images: false,
            has_accent: false,
            has_jisx0213: false,
        }
    }

    /// テキスト全体をHTMLに変換
    pub fn render(&mut self, input: &str) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = input.lines().collect();

        // ヘッダー情報を抽出
        let header_info = extract_header_info(&lines);

        // HTMLヘッダーとメタデータセクションを出力
        self.render_html_head(&mut output, &header_info);
        self.render_metadata_section(&mut output, &header_info);

        // main_text開始
        output.push_str(
            "<div id=\"contents\" style=\"display:none\"></div><div class=\"main_text\">",
        );

        // 本文のみ抽出してレンダリング
        let body_lines = extract_body_lines(&lines);
        for line in &body_lines {
            let line_html = self.render_line(line);

            // ぶら下げブロック内かどうかをチェック
            let burasage_ctx = self.find_burasage_context();

            if let Some((wrap_width, text_indent)) = burasage_ctx {
                // ぶら下げブロック内: 各行を個別のdivでラップ
                // ただし、ブロック要素で始まる/終わる行はラップしない
                let is_block_line = line_html.is_empty()
                    || line_html.starts_with("<div class=\"")
                    || line_html.starts_with("<h3")
                    || line_html.starts_with("<h4")
                    || line_html.starts_with("<h5")
                    || line_html.ends_with("</div>")
                    || line_html.ends_with("</h3>")
                    || line_html.ends_with("</h4>")
                    || line_html.ends_with("</h5>");

                if !is_block_line {
                    output.push_str(&format!(
                        "<div class=\"burasage\" style=\"margin-left: {wrap_width}em; text-indent: {text_indent}em;\">{line_html}</div>"
                    ));
                    output.push_str("\r\n");
                    continue;
                }
            }

            output.push_str(&line_html);

            // ブロック開始/終了だけの行（div/h3/h4/h5で終わる）には<br />を追加しない
            let needs_br = !is_block_only_line(&line_html);
            if needs_br {
                output.push_str("<br />");
            }
            output.push_str("\r\n");
        }

        // 閉じられていないブロックを閉じる
        while let Some(ctx) = self.block_stack.pop() {
            output.push_str(&self.render_block_end_tag(&ctx.block_type, &ctx.params));
        }

        // main_text終了
        output.push_str("</div>\r\n");

        // 底本情報（bibliographical_information）セクション
        let biblio_lines = extract_bibliographical_lines(&lines);
        if !biblio_lines.is_empty() {
            self.render_bibliographical_section(&mut output, &biblio_lines);
        }

        // 表記について（notation_notes）セクション
        self.render_notation_notes(&mut output);

        // 図書カードセクション
        self.render_card_section(&mut output);

        self.render_html_foot(&mut output);

        output
    }

    /// 1行をHTMLに変換
    pub fn render_line(&mut self, line: &str) -> String {
        let tokens = tokenize(line);
        let mut nodes = parse(&tokens);

        // 行内ルビを解決
        resolve_inline_ruby(&mut nodes);

        // 行の開始時点でのブロックスタックの長さを記録
        let stack_len_before = self.block_stack.len();

        let mut output = self.render_nodes(&nodes);

        // 行単位字下げ: 行の終わりで、その行で開いたブロックを閉じる
        // 「ここから」で始まるブロックは BlockParams に幅があるが、
        // 元のコマンドが「ここから」かどうかを判定するため、元の行をチェック
        let is_line_scope_block = line.starts_with("［＃")
            && !line.contains("ここから")
            && (line.contains("字下げ") || line.contains("地付き") || line.contains("地から"));

        if is_line_scope_block {
            // その行で開いたブロックを閉じる
            while self.block_stack.len() > stack_len_before {
                if let Some(ctx) = self.block_stack.pop() {
                    output.push_str(&self.render_block_end_tag(&ctx.block_type, &ctx.params));
                }
            }
        }

        output
    }

    /// ノード列をHTMLに変換
    pub fn render_nodes(&mut self, nodes: &[Node]) -> String {
        let mut output = String::new();

        for node in nodes {
            output.push_str(&self.render_node(node));
        }

        output
    }

    /// 単一ノードをHTMLに変換
    fn render_node(&mut self, node: &Node) -> String {
        match node {
            Node::Text(text) => html_escape(text),

            Node::Ruby {
                children,
                ruby,
                direction,
            } => self.render_ruby(children, ruby, *direction),

            Node::Style {
                children,
                style_type,
                class_name: _,
            } => self.render_style(children, *style_type),

            Node::Midashi {
                children,
                level,
                style,
            } => self.render_midashi(children, *level, *style),

            Node::Gaiji {
                description,
                unicode,
                jis_code,
            } => self.render_gaiji(description, unicode.as_deref(), jis_code.as_deref()),

            Node::Accent {
                code,
                name,
                unicode,
            } => {
                self.has_accent = true;
                if self.options.use_jisx0213 || self.options.use_unicode {
                    // --use-jisx0213 or --use-unicode: 数値実体参照で出力
                    if let Some(u) = unicode {
                        u.chars().map(|c| format!("&#{};", c as u32)).collect()
                    } else {
                        String::new()
                    }
                } else {
                    // デフォルト: 画像として出力（Ruby版と同じ）
                    self.has_gaiji_images = true;
                    let (folder, file) = jis_code_to_path(code);
                    format!(
                        "<img src=\"{}{}/{}.png\" alt=\"※({})\" class=\"gaiji\" />",
                        self.options.gaiji_dir,
                        folder,
                        file,
                        html_escape(name)
                    )
                }
            }

            Node::Img {
                filename,
                alt,
                css_class,
                width,
                height,
            } => self.render_img(filename, alt, css_class, *width, *height),

            Node::Tcy { children } => {
                let inner = self.render_nodes(children);
                format!("<span class=\"tcy\">{inner}</span>")
            }

            Node::Keigakomi { children } => {
                let inner = self.render_nodes(children);
                format!("<span class=\"keigakomi\">{inner}</span>")
            }

            Node::Caption { children } => {
                let inner = self.render_nodes(children);
                format!("<span class=\"caption\">{inner}</span>")
            }

            Node::Warigaki { upper, lower } => {
                let upper_html = self.render_nodes(upper);
                let lower_html = self.render_nodes(lower);
                format!(
                    "<span class=\"warichu\"><span class=\"warichu_upper\">{upper_html}</span><span class=\"warichu_lower\">{lower_html}</span></span>"
                )
            }

            Node::Kaeriten(text) => {
                format!("<sub class=\"kaeriten\">{}</sub>", html_escape(text))
            }

            Node::Okurigana(text) => {
                format!("<sup class=\"okurigana\">{}</sup>", html_escape(text))
            }

            Node::BlockStart { block_type, params } => {
                let mut output = String::new();

                // 新しいブロック開始時は、開いている同タイプまたは関連ブロックを閉じる
                if *block_type == BlockType::Jisage
                    || *block_type == BlockType::Chitsuki
                    || *block_type == BlockType::Burasage
                {
                    // 同タイプまたは関連ブロックを探して閉じる
                    while let Some(pos) = self.block_stack.iter().rposition(|c| {
                        c.block_type == *block_type
                            || c.block_type == BlockType::Burasage
                            || (*block_type == BlockType::Jisage
                                && c.block_type == BlockType::Jisage)
                    }) {
                        let ctx = self.block_stack.remove(pos);
                        // Burasageは終了タグを出力しない
                        if ctx.block_type != BlockType::Burasage {
                            output
                                .push_str(&self.render_block_end_tag(&ctx.block_type, &ctx.params));
                        }
                    }
                }

                self.block_stack.push(BlockContext {
                    block_type: *block_type,
                    params: params.clone(),
                });
                // Burasageは各行で個別にラップするため、開始タグを出力しない
                if *block_type != BlockType::Burasage {
                    output.push_str(&self.render_block_start_tag(block_type, params));
                }
                output
            }

            Node::BlockEnd { block_type } => {
                // スタックから対応するブロックを探して閉じる
                // Jisage終了でBurasageも閉じる（「ここで字下げ終わり」がBurasageを閉じる）
                let pos = self.block_stack.iter().rposition(|c| {
                    c.block_type == *block_type
                        || (*block_type == BlockType::Jisage && c.block_type == BlockType::Burasage)
                });

                if let Some(pos) = pos {
                    let ctx = self.block_stack.remove(pos);
                    // Burasageは各行で個別にラップするため、終了タグを出力しない
                    if ctx.block_type == BlockType::Burasage {
                        String::new()
                    } else {
                        self.render_block_end_tag(&ctx.block_type, &ctx.params)
                    }
                } else {
                    // 対応するブロックがない場合は空文字
                    String::new()
                }
            }

            Node::Note(text) => {
                self.has_notes = true;
                format!("<span class=\"notes\">［＃{}］</span>", html_escape(text))
            }

            Node::UnresolvedReference {
                target,
                spec,
                connector,
            } => {
                // 解決できなかった参照は注記として出力
                format!(
                    "<span class=\"notes\">［＃「{}」{}{}］</span>",
                    html_escape(target),
                    html_escape(connector),
                    html_escape(spec)
                )
            }

            Node::DakutenKatakana { num } => {
                // 濁点カタカナの出力
                match num.as_str() {
                    "2" => "ワ゛".to_string(),
                    "3" => "ヰ゛".to_string(),
                    "4" => "ヱ゛".to_string(),
                    "5" => "ヲ゛".to_string(),
                    _ => String::new(),
                }
            }
        }
    }

    /// ルビをHTMLに変換
    fn render_ruby(
        &mut self,
        children: &[Node],
        ruby: &[Node],
        direction: RubyDirection,
    ) -> String {
        let base_html = self.render_nodes(children);
        let ruby_html = self.render_nodes(ruby);

        match direction {
            RubyDirection::Right => {
                format!(
                    "<ruby><rb>{base_html}</rb><rp>（</rp><rt>{ruby_html}</rt><rp>）</rp></ruby>"
                )
            }
            RubyDirection::Left => {
                // 左ルビ（縦書き用）
                format!(
                    "<ruby class=\"leftrb\"><rb>{base_html}</rb><rp>（</rp><rt>{ruby_html}</rt><rp>）</rp></ruby>"
                )
            }
        }
    }

    /// 装飾をHTMLに変換
    fn render_style(&mut self, children: &[Node], style_type: StyleType) -> String {
        let inner = self.render_nodes(children);
        let tag = style_html_tag(style_type);
        let class = style_css_class(style_type);

        format!("<{tag} class=\"{class}\">{inner}</{tag}>")
    }

    /// 見出しをHTMLに変換
    fn render_midashi(
        &mut self,
        children: &[Node],
        level: MidashiLevel,
        style: MidashiStyle,
    ) -> String {
        let inner = self.render_nodes(children);
        let tag = midashi_html_tag(level);
        let class = midashi_css_class(level);
        let midashi_id = self.generate_midashi_id();

        match style {
            MidashiStyle::Normal => {
                // 通常見出しにもアンカーを追加
                format!(
                    "<{tag} class=\"{class}\"><a class=\"midashi_anchor\" id=\"midashi{midashi_id}\">{inner}</a></{tag}>"
                )
            }
            MidashiStyle::Dogyo => {
                // 同行見出し
                format!(
                    "<{tag} class=\"{class} dogyo-midashi\"><a class=\"midashi_anchor\" id=\"midashi{midashi_id}\">{inner}</a></{tag}>"
                )
            }
            MidashiStyle::Mado => {
                // 窓見出し
                format!(
                    "<{tag} class=\"{class} mado-midashi\"><a class=\"midashi_anchor\" id=\"midashi{midashi_id}\">{inner}</a></{tag}>"
                )
            }
        }
    }

    /// 見出しIDを生成
    fn generate_midashi_id(&mut self) -> u32 {
        let id = self.midashi_id_counter;
        self.midashi_id_counter += 10;
        id
    }

    /// ぶら下げブロック内かどうかをチェックし、パラメータを返す
    fn find_burasage_context(&self) -> Option<(u32, i32)> {
        for ctx in &self.block_stack {
            if ctx.block_type == BlockType::Burasage {
                let wrap_width = ctx.params.wrap_width.unwrap_or(1);
                let width = ctx.params.width.unwrap_or(0);
                let text_indent = width as i32 - wrap_width as i32;
                return Some((wrap_width, text_indent));
            }
        }
        None
    }

    /// 外字をHTMLに変換
    fn render_gaiji(
        &mut self,
        description: &str,
        unicode: Option<&str>,
        jis_code: Option<&str>,
    ) -> String {
        // すでにパース済みの情報がある場合はそれを使用
        match (unicode, jis_code) {
            // JisConverted: unicodeとjis_code両方がある場合
            (Some(u), Some(jis)) => {
                self.has_jisx0213 = true;
                if self.options.use_jisx0213 || self.options.use_unicode {
                    // --use-jisx0213 or --use-unicode: 数値実体参照で出力
                    return u.chars().map(|c| format!("&#{};", c as u32)).collect();
                } else {
                    // デフォルト: 画像として出力（Ruby版と同じ）
                    self.has_gaiji_images = true;
                    let (folder, file) = jis_code_to_path(jis);
                    return format!(
                        "<img src=\"{}{}/{}.png\" alt=\"※({})\" class=\"gaiji\" />",
                        self.options.gaiji_dir,
                        folder,
                        file,
                        html_escape(description)
                    );
                }
            }
            // Unicode: unicodeだけがある場合
            (Some(u), None) => {
                if self.options.use_unicode {
                    return u.chars().map(|c| format!("&#{};", c as u32)).collect();
                }
                return u.to_string();
            }
            // JisImage: jis_codeだけがある場合（変換テーブルにない）
            (None, Some(jis)) => {
                self.has_gaiji_images = true;
                let (folder, file) = jis_code_to_path(jis);
                return format!(
                    "<img src=\"{}{}/{}.png\" alt=\"※({})\" class=\"gaiji\" />",
                    self.options.gaiji_dir,
                    folder,
                    file,
                    html_escape(description)
                );
            }
            // 両方Noneの場合は下でparse_gaijiを再実行
            (None, None) => {}
        }

        // パース済み情報がない場合は再度パース
        match parse_gaiji(description) {
            GaijiResult::Unicode(s) => {
                if self.options.use_unicode {
                    s.chars().map(|c| format!("&#{};", c as u32)).collect()
                } else {
                    s
                }
            }
            GaijiResult::JisConverted {
                jis_code: jis,
                unicode: u,
            } => {
                self.has_jisx0213 = true;
                if self.options.use_jisx0213 || self.options.use_unicode {
                    // --use-jisx0213 or --use-unicode: 数値実体参照で出力
                    u.chars().map(|c| format!("&#{};", c as u32)).collect()
                } else {
                    // デフォルト: 画像として出力（Ruby版と同じ）
                    self.has_gaiji_images = true;
                    let (folder, file) = jis_code_to_path(&jis);
                    format!(
                        "<img src=\"{}{}/{}.png\" alt=\"※({})\" class=\"gaiji\" />",
                        self.options.gaiji_dir,
                        folder,
                        file,
                        html_escape(description)
                    )
                }
            }
            GaijiResult::JisImage { jis_code: jis } => {
                self.has_gaiji_images = true;
                let (folder, file) = jis_code_to_path(&jis);
                format!(
                    "<img src=\"{}{}/{}.png\" alt=\"※({})\" class=\"gaiji\" />",
                    self.options.gaiji_dir,
                    folder,
                    file,
                    html_escape(description)
                )
            }
            GaijiResult::Unconvertible => {
                format!(
                    "<span class=\"notes\">※［＃{}］</span>",
                    html_escape(description)
                )
            }
        }
    }

    /// 画像をHTMLに変換
    fn render_img(
        &self,
        filename: &str,
        alt: &str,
        css_class: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> String {
        let mut attrs = format!(
            "src=\"{}{}\" alt=\"{}\"",
            self.options.gaiji_dir,
            filename,
            html_escape(alt)
        );

        if !css_class.is_empty() {
            attrs.push_str(&format!(" class=\"{css_class}\""));
        }

        if let Some(w) = width {
            attrs.push_str(&format!(" width=\"{w}\""));
        }

        if let Some(h) = height {
            attrs.push_str(&format!(" height=\"{h}\""));
        }

        format!("<img {attrs} />")
    }

    /// ブロック開始タグを生成
    fn render_block_start_tag(&self, block_type: &BlockType, params: &BlockParams) -> String {
        match block_type {
            BlockType::Jisage => {
                if let Some(width) = params.width {
                    format!("<div class=\"jisage_{width}\" style=\"margin-left: {width}em\">")
                } else {
                    "<div class=\"jisage\">".to_string()
                }
            }
            BlockType::Chitsuki => {
                let width = params.width.unwrap_or(0);
                format!(
                    "<div class=\"chitsuki_{width}\" style=\"text-align:right; margin-right: {width}em\">"
                )
            }
            BlockType::Jizume => {
                if let Some(width) = params.width {
                    format!("<div class=\"jizume_{width}\" style=\"width: {width}em\">")
                } else {
                    "<div class=\"jizume\">".to_string()
                }
            }
            BlockType::Keigakomi => "<div class=\"keigakomi\">".to_string(),
            BlockType::Midashi => {
                if let Some(level) = params.level {
                    format!(
                        "<{} class=\"{}\">",
                        midashi_html_tag(level),
                        midashi_css_class(level)
                    )
                } else {
                    "<h3 class=\"o-midashi\">".to_string()
                }
            }
            BlockType::Yokogumi => "<div class=\"yokogumi\">".to_string(),
            BlockType::Futoji => "<div class=\"futoji\">".to_string(),
            BlockType::Shatai => "<div class=\"shatai\">".to_string(),
            BlockType::FontDai => {
                if let Some(size) = params.font_size {
                    format!("<span class=\"dai{size}\">")
                } else {
                    "<span class=\"dai\">".to_string()
                }
            }
            BlockType::FontSho => {
                if let Some(size) = params.font_size {
                    format!("<span class=\"sho{size}\">")
                } else {
                    "<span class=\"sho\">".to_string()
                }
            }
            BlockType::Tcy => "<span class=\"tcy\">".to_string(),
            BlockType::Caption => "<span class=\"caption\">".to_string(),
            BlockType::Warigaki => "<span class=\"warichu\">".to_string(),
            BlockType::Burasage => {
                // ぶら下げ: margin-left = wrap_width, text-indent = width - wrap_width
                let wrap_width = params.wrap_width.unwrap_or(1);
                let width = params.width.unwrap_or(0);
                let text_indent = width as i32 - wrap_width as i32;
                format!(
                    "<div class=\"burasage\" style=\"margin-left: {wrap_width}em; text-indent: {text_indent}em;\">"
                )
            }
        }
    }

    /// ブロック終了タグを生成
    fn render_block_end_tag(&self, block_type: &BlockType, params: &BlockParams) -> String {
        match block_type {
            BlockType::Jisage
            | BlockType::Chitsuki
            | BlockType::Jizume
            | BlockType::Keigakomi
            | BlockType::Yokogumi
            | BlockType::Futoji
            | BlockType::Shatai
            | BlockType::Burasage => "</div>".to_string(),
            BlockType::Midashi => {
                if let Some(level) = params.level {
                    format!("</{}>", midashi_html_tag(level))
                } else {
                    "</h3>".to_string()
                }
            }
            BlockType::FontDai
            | BlockType::FontSho
            | BlockType::Tcy
            | BlockType::Caption
            | BlockType::Warigaki => "</span>".to_string(),
        }
    }

    /// HTMLヘッダーを出力
    fn render_html_head(&self, output: &mut String, header_info: &HeaderInfo) {
        // XML宣言とDOCTYPE
        output.push_str("<?xml version=\"1.0\" encoding=\"Shift_JIS\"?>\r\n");
        output.push_str("<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1//EN\"\r\n");
        output.push_str("    \"http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd\">\r\n");
        output.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\" xml:lang=\"ja\" >\r\n");
        output.push_str("<head>\r\n");

        // メタ情報
        output.push_str(
            "\t<meta http-equiv=\"Content-Type\" content=\"text/html;charset=Shift_JIS\" />\r\n",
        );
        output.push_str("\t<meta http-equiv=\"content-style-type\" content=\"text/css\" />\r\n");

        // CSSリンク
        for css in &self.options.css_files {
            output.push_str(&format!(
                "\t<link rel=\"stylesheet\" type=\"text/css\" href=\"{css}\" />\r\n"
            ));
        }

        // タイトル
        let html_title = if let Some(title) = &self.options.title {
            html_escape(title)
        } else {
            header_info.html_title()
        };
        output.push_str(&format!("\t<title>{}</title>\r\n", html_title));

        // jQuery
        output.push_str(
            "\t<script type=\"text/javascript\" src=\"../../jquery-1.4.2.min.js\"></script>\r\n",
        );

        // Dublin Core メタデータ
        output
            .push_str("  <link rel=\"Schema.DC\" href=\"http://purl.org/dc/elements/1.1/\" />\r\n");

        let dc_title = header_info.title.as_deref().unwrap_or("");
        let dc_creator = header_info.author.as_deref().unwrap_or("");
        output.push_str(&format!(
            "\t<meta name=\"DC.Title\" content=\"{}\" />\r\n",
            html_escape(dc_title)
        ));
        output.push_str(&format!(
            "\t<meta name=\"DC.Creator\" content=\"{}\" />\r\n",
            html_escape(dc_creator)
        ));
        output.push_str(&format!(
            "\t<meta name=\"DC.Publisher\" content=\"{}\" />\r\n",
            AOZORA_BUNKO
        ));

        output.push_str("</head>\r\n");
        output.push_str("<body>\r\n");
    }

    /// メタデータセクションを出力
    fn render_metadata_section(&self, output: &mut String, header_info: &HeaderInfo) {
        output.push_str("<div class=\"metadata\">\r\n");

        if let Some(title) = &header_info.title {
            output.push_str(&format!(
                "<h1 class=\"title\">{}</h1>\r\n",
                html_escape(title)
            ));
        }

        if let Some(original_title) = &header_info.original_title {
            output.push_str(&format!(
                "<h2 class=\"original_title\">{}</h2>\r\n",
                html_escape(original_title)
            ));
        }

        if let Some(subtitle) = &header_info.subtitle {
            output.push_str(&format!(
                "<h2 class=\"subtitle\">{}</h2>\r\n",
                html_escape(subtitle)
            ));
        }

        if let Some(original_subtitle) = &header_info.original_subtitle {
            output.push_str(&format!(
                "<h2 class=\"original_subtitle\">{}</h2>\r\n",
                html_escape(original_subtitle)
            ));
        }

        if let Some(author) = &header_info.author {
            output.push_str(&format!(
                "<h2 class=\"author\">{}</h2>\r\n",
                html_escape(author)
            ));
        }

        if let Some(editor) = &header_info.editor {
            output.push_str(&format!(
                "<h2 class=\"editor\">{}</h2>\r\n",
                html_escape(editor)
            ));
        }

        if let Some(translator) = &header_info.translator {
            output.push_str(&format!(
                "<h2 class=\"translator\">{}</h2>\r\n",
                html_escape(translator)
            ));
        }

        if let Some(henyaku) = &header_info.henyaku {
            output.push_str(&format!(
                "<h2 class=\"editor-translator\">{}</h2>\r\n",
                html_escape(henyaku)
            ));
        }

        output.push_str("<br />\r\n<br />\r\n</div>\r\n");
    }

    /// HTMLフッターを出力
    fn render_html_foot(&self, output: &mut String) {
        output.push_str("</body>\r\n");
        output.push_str("</html>\r\n");
    }

    /// 底本情報セクションを出力
    fn render_bibliographical_section(&mut self, output: &mut String, lines: &[&str]) {
        output.push_str("<div class=\"bibliographical_information\">\r\n");
        output.push_str("<hr />\r\n");
        output.push_str("<br />\r\n");

        for line in lines {
            let line_html = self.render_line(line);
            output.push_str(&line_html);
            output.push_str("<br />\r\n");
        }

        output.push_str("</div>\r\n");
    }

    /// 表記についてセクションを出力
    fn render_notation_notes(&self, output: &mut String) {
        output.push_str("<div class=\"notation_notes\">\r\n");
        output.push_str("<hr />\r\n");
        output.push_str("<br />\r\n");
        output.push_str("●表記について<br />\r\n");
        output.push_str("<ul>\r\n");

        // XHTML1.1準拠
        output.push_str(
            "\t<li>このファイルは W3C 勧告 XHTML1.1 にそった形式で作成されています。</li>\r\n",
        );

        // 注記を使用した場合
        if self.has_notes {
            output.push_str("\t<li>［＃…］は、入力者による注を表す記号です。</li>\r\n");
        }

        // JIS X 0213文字を画像化した場合
        if self.has_jisx0213 && !self.options.use_jisx0213 {
            output.push_str("\t<li>「くの字点」をのぞくJIS X 0213にある文字は、画像化して埋め込みました。</li>\r\n");
        }

        // アクセント符号を使用した場合
        if self.has_accent && !self.options.use_jisx0213 {
            output.push_str(
                "\t<li>アクセント符号付きラテン文字は、画像化して埋め込みました。</li>\r\n",
            );
        }

        output.push_str("</ul>\r\n");
        output.push_str("</div>\r\n");
    }

    /// 図書カードセクションを出力
    fn render_card_section(&self, output: &mut String) {
        output.push_str("<div id=\"card\">\r\n");
        output.push_str("<hr />\r\n");
        output.push_str("<br />\r\n");
        output.push_str("<a href=\"JavaScript:goLibCard();\" id=\"goAZLibCard\">●図書カード</a>");
        output.push_str("<script type=\"text/javascript\" src=\"../../contents.js\"></script>\r\n");
        output
            .push_str("<script type=\"text/javascript\" src=\"../../golibcard.js\"></script>\r\n");
        output.push_str("</div>");
    }
}

/// 行がブロック要素だけかどうかを判定（<br />を追加しない）
fn is_block_only_line(html: &str) -> bool {
    // 空行
    if html.is_empty() {
        return false;
    }

    // ブロック開始タグのみで終わる（jisage, chitsuki, midashi など）
    if html.ends_with("\">") {
        // divで始まりdivで終わる場合（ブロック開始のみ）
        if html.starts_with("<div class=\"jisage")
            || html.starts_with("<div class=\"chitsuki")
            || html.starts_with("<div class=\"jizume")
        {
            return true;
        }
    }

    // 見出しで終わる（</h3>, </h4>, </h5>）
    if html.ends_with("</h3>") || html.ends_with("</h4>") || html.ends_with("</h5>") {
        return true;
    }

    // ブロック終了タグで終わる（</div>）
    if html.ends_with("</div>") {
        return true;
    }

    false
}

/// HTMLエスケープ
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// JISコードをファイルパスに変換
fn jis_code_to_path(jis_code: &str) -> (String, String) {
    // "1-02-22" → ("1-02", "1-02-22")
    let parts: Vec<&str> = jis_code.split('-').collect();
    if parts.len() == 3 {
        let folder = format!("{}-{}", parts[0], parts[1]);
        (folder, jis_code.to_string())
    } else {
        ("".to_string(), jis_code.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_text() {
        let mut renderer = HtmlRenderer::new(RenderOptions::default());
        let html = renderer.render_line("こんにちは");
        assert_eq!(html, "こんにちは");
    }

    #[test]
    fn test_render_ruby() {
        let mut renderer = HtmlRenderer::new(RenderOptions::default());
        let html = renderer.render_line("漢字《かんじ》");
        assert!(html.contains("<ruby>"));
        assert!(html.contains("<rb>漢字</rb>"));
        assert!(html.contains("<rt>かんじ</rt>"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<test>"), "&lt;test&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_jis_code_to_path() {
        let (folder, file) = jis_code_to_path("1-02-22");
        assert_eq!(folder, "1-02");
        assert_eq!(file, "1-02-22");
    }
}
