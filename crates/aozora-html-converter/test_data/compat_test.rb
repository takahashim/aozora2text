#!/usr/bin/env ruby
# frozen_string_literal: true

# 互換性テストスクリプト
# aozora2html (Ruby) と aozora-html-converter (Rust) の出力を比較

require 'bundler/setup'
require 'aozora2html'
require 'stringio'
require 'open3'

using Aozora2Html::StringRefinements

RUST_CONVERTER = File.expand_path('../target/release/aozora-html-converter', __dir__)

# テストケース: [入力, 期待される出力パターン（Rubyの出力）]
TEST_CASES = [
  # 基本テキスト
  ["こんにちは\r\n", /こんにちは/],

  # ルビ（自動親文字抽出）
  ["青空文庫《あおぞらぶんこ》\r\n", /<ruby><rb>青空文庫<\/rb><rp>（<\/rp><rt>あおぞらぶんこ<\/rt><rp>）<\/rp><\/ruby>/],

  # ルビ（親文字指定）
  ["｜東京《とうきょう》\r\n", /<ruby><rb>東京<\/rb><rp>（<\/rp><rt>とうきょう<\/rt><rp>）<\/rp><\/ruby>/],

  # ルビ（混合テキスト）
  ["私の東京《とうきょう》\r\n", /私の<ruby><rb>東京<\/rb>/],

  # 傍点
  ["重要なこと［＃「重要」に傍点］\r\n", /<em class="sesame_dot">重要<\/em>なこと/],

  # 太字
  ["大切だ［＃「大切」に太字］\r\n", /<span class="futoji">大切<\/span>だ/],

  # 見出し
  ["第一章［＃「第一章」は大見出し］\r\n", /<h3 class="o-midashi".*>第一章<\/h3>/],

  # 中見出し
  ["第一節［＃「第一節」は中見出し］\r\n", /<h4 class="naka-midashi".*>第一節<\/h4>/],

  # 小見出し
  ["概要［＃「概要」は小見出し］\r\n", /<h5 class="ko-midashi".*>概要<\/h5>/],

  # 斜体
  ["これは斜め［＃「斜め」に斜体］\r\n", /<span class="shatai">斜め<\/span>/],

  # 傍線
  ["これは重要だ［＃「重要」に傍線］\r\n", /<em class="underline_solid">重要<\/em>だ/],

  # 二重傍線
  ["強調語句［＃「強調」に二重傍線］\r\n", /<em class="underline_double">強調<\/em>語句/],

  # 波線
  ["大事な部分［＃「大事」に波線］\r\n", /<em class="underline_wave">大事<\/em>な部分/],

  # 字下げブロック
  ["［＃ここから２字下げ］\r\n", /<div class="jisage_2">/],

  # 字下げ終わり（ブロック開始とセットでテスト）
  ["［＃ここから２字下げ］\r\nテスト\r\n［＃ここで字下げ終わり］\r\n", /<div class="jisage_2"[^>]*>.*<\/div>/m],

  # 行字下げ
  ["［＃３字下げ］\r\n", /<div class="jisage_3">/],

  # 複合ルビ（複数漢字 + ルビ）
  ["山田太郎《やまだたろう》\r\n", /<ruby><rb>山田太郎<\/rb><rp>（<\/rp><rt>やまだたろう<\/rt><rp>）<\/rp><\/ruby>/],

  # ひらがなのルビ
  ["すずめ《・・・》\r\n", /<ruby><rb>すずめ<\/rb><rp>（<\/rp><rt>・・・<\/rt><rp>）<\/rp><\/ruby>/],

  # カタカナのルビ
  ["スズメ《すずめ》\r\n", /<ruby><rb>スズメ<\/rb><rp>（<\/rp><rt>すずめ<\/rt><rp>）<\/rp><\/ruby>/],

  # 縦中横
  ["［＃縦中横］12［＃縦中横終わり］\r\n", /<span class="tcy">12<\/span>/],
]

def parse_ruby(input_text)
  input = StringIO.new(input_text.to_sjis)
  output = StringIO.new
  parser = Aozora2Html.new(input, output)
  parser.instance_eval do
    @section = :tail
    start_document_section(:body)
  end
  catch(:terminate) do
    loop do
      parser.__send__(:parse)
    end
  end
  document = parser.instance_variable_get(:@document)
  Aozora2Html::HtmlRenderer.render_document(document, output)
  output.string.to_utf8
end

def parse_rust(input_text)
  # Rustコンバーターを呼び出し
  unless File.exist?(RUST_CONVERTER)
    # releaseビルドがなければdebugを使う
    debug_converter = File.expand_path('../target/debug/aozora-html-converter', __dir__)
    if File.exist?(debug_converter)
      converter = debug_converter
    else
      return nil
    end
  else
    converter = RUST_CONVERTER
  end

  stdout, stderr, status = Open3.capture3(converter, stdin_data: input_text.gsub("\r\n", "\n"))
  stdout
end

def run_tests
  passed = 0
  failed = 0
  errors = []

  puts "=" * 60
  puts "aozora2html 互換性テスト"
  puts "=" * 60
  puts

  TEST_CASES.each_with_index do |(input, expected_pattern), i|
    print "Test #{i + 1}: "

    begin
      ruby_output = parse_ruby(input)
      rust_output = parse_rust(input)

      if rust_output.nil?
        puts "SKIP (Rust converter not found)"
        next
      end

      # 行末の<br />タグを除去して比較（Rubyは<br />を追加する）
      ruby_clean = ruby_output.gsub(/<br \/>/, '').strip
      rust_clean = rust_output.strip

      if expected_pattern.match?(rust_clean)
        puts "PASS"
        passed += 1
      else
        puts "FAIL"
        failed += 1
        errors << {
          test: i + 1,
          input: input.inspect,
          ruby: ruby_clean,
          rust: rust_clean,
          expected: expected_pattern.inspect
        }
      end
    rescue => e
      puts "ERROR: #{e.message}"
      failed += 1
      errors << {
        test: i + 1,
        input: input.inspect,
        error: e.message
      }
    end
  end

  puts
  puts "=" * 60
  puts "結果: #{passed} passed, #{failed} failed"
  puts "=" * 60

  unless errors.empty?
    puts
    puts "失敗したテスト:"
    errors.each do |err|
      puts "-" * 40
      puts "Test #{err[:test]}"
      puts "Input: #{err[:input]}"
      if err[:error]
        puts "Error: #{err[:error]}"
      else
        puts "Expected pattern: #{err[:expected]}"
        puts "Ruby output: #{err[:ruby]}"
        puts "Rust output: #{err[:rust]}"
      end
    end
  end

  failed == 0
end

exit(run_tests ? 0 : 1)
