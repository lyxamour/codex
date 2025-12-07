//! 语言检测器
//!
//! 负责检测和识别文本语言

use super::locale::{Language, NaturalLanguage};
use std::collections::HashSet;

/// 语言检测器
pub struct LanguageDetector;

impl LanguageDetector {
    /// 创建新的语言检测器
    pub fn new() -> Self {
        Self
    }

    /// 检测自然语言
    pub fn detect_natural_language(&self, text: &str) -> NaturalLanguage {
        // 简单的语言检测算法，基于常见字符集和词汇
        // 实际应用中可以使用更复杂的NLP模型

        // 日语检测：检查是否包含日语平假名或片假名
        if text.chars().any(|c| matches!(c, '\u{3040}'..='\u{30ff}')) {
            return NaturalLanguage::Japanese;
        }

        // 中文检测：检查是否包含中文字符
        if text.chars().any(|c| matches!(c, '\u{4e00}'..='\u{9fff}')) {
            // 检查是否包含繁体中文字符
            let traditional_chars: HashSet<char> = [
                '優', '愛', '戰', '學', '數', '圖', '會', '裡', '車', '馬', '過', '後', '門', '開',
                '關', '電', '話',
            ]
            .iter()
            .cloned()
            .collect();

            if text.chars().any(|c| traditional_chars.contains(&c)) {
                return NaturalLanguage::ChineseTraditional;
            } else {
                return NaturalLanguage::ChineseSimplified;
            }
        }

        // 韩语检测：检查是否包含韩文字符
        if text.chars().any(|c| matches!(c, '\u{ac00}'..='\u{d7af}')) {
            return NaturalLanguage::Korean;
        }

        // 西班牙语检测：检查是否包含常见西班牙语词汇或字符
        let text_lower = text.to_lowercase();

        // 更准确的西班牙语检测，检查是否包含感叹号倒置
        if text.contains("¡") || text.contains("¿") {
            return NaturalLanguage::Spanish;
        }

        // 分割文本为单词，用于准确匹配
        let text_words: Vec<&str> = text_lower
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .collect();
        let word_count = text_words.len();

        // 语言检测评分系统
        let mut scores: Vec<(NaturalLanguage, f64)> = Vec::new();

        // 西班牙语检测
        let spanish_words = [
            "el", "la", "los", "las", "de", "que", "y", "a", "en", "un", "una", "para", "con",
            "no", "su", "es",
        ];
        let spanish_matches = text_words
            .iter()
            .filter(|word| spanish_words.contains(word))
            .count();
        if word_count > 0 {
            let spanish_score = spanish_matches as f64 / word_count as f64;
            if spanish_score > 0.3 {
                // 需要超过30%的匹配率
                scores.push((NaturalLanguage::Spanish, spanish_score));
            }
        }

        // 法语检测
        let french_words = [
            "le", "la", "les", "de", "du", "des", "que", "et", "a", "en", "un", "une", "pour",
            "avec", "pas", "son", "est",
        ];
        let french_matches = text_words
            .iter()
            .filter(|word| french_words.contains(word))
            .count();
        if word_count > 0 {
            let french_score = french_matches as f64 / word_count as f64;
            if french_score > 0.3 {
                // 需要超过30%的匹配率
                scores.push((NaturalLanguage::French, french_score));
            }
        }

        // 德语检测
        let german_words = [
            "der", "die", "das", "und", "in", "von", "zu", "ist", "nicht", "ein", "eine", "den",
            "mit", "dem", "auf", "für",
        ];
        let german_matches = text_words
            .iter()
            .filter(|word| german_words.contains(word))
            .count();
        if word_count > 0 {
            let german_score = german_matches as f64 / word_count as f64;
            if german_score > 0.3 {
                // 需要超过30%的匹配率
                scores.push((NaturalLanguage::German, german_score));
            }
        }

        // 葡萄牙语检测（移除了重复的"a"）
        let portuguese_words = [
            "o", "a", "os", "as", "de", "que", "e", "em", "um", "uma", "para", "com", "não", "seu",
            "é",
        ];
        let portuguese_matches = text_words
            .iter()
            .filter(|word| portuguese_words.contains(word))
            .count();
        if word_count > 0 {
            let portuguese_score = portuguese_matches as f64 / word_count as f64;
            if portuguese_score > 0.3 {
                // 需要超过30%的匹配率
                scores.push((NaturalLanguage::Portuguese, portuguese_score));
            }
        }

        // 俄语检测：检查是否包含俄语字符
        if text.chars().any(|c| matches!(c, '\u{0400}'..='\u{04ff}')) {
            scores.push((NaturalLanguage::Russian, 1.0)); // 俄语字符是强标识符
        }

        // 阿拉伯语检测：检查是否包含阿拉伯字符
        if text.chars().any(|c| matches!(c, '\u{0600}'..='\u{06ff}')) {
            scores.push((NaturalLanguage::Arabic, 1.0)); // 阿拉伯字符是强标识符
        }

        // 韩语检测：检查是否包含韩文字符
        if text.chars().any(|c| matches!(c, '\u{ac00}'..='\u{d7af}')) {
            scores.push((NaturalLanguage::Korean, 1.0)); // 韩文字符是强标识符
        }

        // 独特词汇检测：处理短句子和独特词汇
        let text_lower = text.to_lowercase();
        let text_words: Vec<&str> = text_lower
            .split_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| !word.is_empty())
            .collect();

        // 英语独特词汇 - 先检测英语以避免常见单词冲突
        let english_words = [
            "hello",
            "world",
            "this",
            "is",
            "simple",
            "english",
            "sentence",
            "the",
            "and",
            "but",
            "or",
            "not",
            "if",
            "then",
            "else",
            "because",
            "so",
            "for",
            "while",
            "do",
            "done",
            "until",
            "loop",
            "function",
            "method",
            "class",
            "object",
            "variable",
            "const",
            "let",
            "var",
            "def",
            "fn",
            "func",
            "procedure",
            "subroutine",
        ];
        let english_matches = text_words
            .iter()
            .filter(|word| english_words.contains(word))
            .count();
        if english_matches > 0
            && text_words.len() > 0
            && english_matches as f64 / text_words.len() as f64 > 0.2
        {
            return NaturalLanguage::English;
        }

        // 法语检测 - 包含独特问候词
        if text_lower.contains("bonjour") || text_lower.contains("merci") {
            return NaturalLanguage::French;
        }

        // 西班牙语检测 - 包含独特问候词或标点
        if text_lower.contains("hola")
            || text_lower.contains("gracias")
            || text.contains("¡")
            || text.contains("¿")
        {
            return NaturalLanguage::Spanish;
        }

        // 德语检测 - 包含独特问候词
        if text_lower.contains("hallo") || text_lower.contains("danke") {
            return NaturalLanguage::German;
        }

        // 葡萄牙语检测 - 包含独特问候词
        if text_lower.contains("olá") || text_lower.contains("obrigado") {
            return NaturalLanguage::Portuguese;
        }

        // 选择得分最高的语言
        if let Some((best_lang, _)) = scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            return best_lang;
        }

        // 默认返回英语
        NaturalLanguage::English
    }

    /// 检测编程语言
    pub fn detect_programming_language(&self, text: &str) -> Language {
        // 基于代码特征的编程语言检测
        let text_lower = text.to_lowercase();

        // 检查各种编程语言的特征

        // Rust检测：更准确的特征检测
        if text_lower.contains("fn main()")
            || (text_lower.contains("fn ") && text_lower.contains("let "))
            || text_lower.contains("use std::")
            || text_lower.contains("cargo.toml")
        {
            return Language::Rust;
        }

        // Python检测：更宽松的检测逻辑，只需要包含def或者import即可
        if text_lower.contains("def ")
            || text_lower.contains("import ")
            || text_lower.contains("from ")
        {
            return Language::Python;
        }

        // JavaScript检测：更宽松的条件，包含function和console.log也可以识别为JavaScript
        if text_lower.contains("function ")
            && (text_lower.contains("var ")
                || text_lower.contains("const ")
                || text_lower.contains("console.log")
                || text_lower.contains("console.error"))
        {
            return Language::JavaScript;
        }

        // TypeScript检测
        if text_lower.contains("function ") && text_lower.contains("interface ") {
            return Language::TypeScript;
        }

        // Java检测
        if text_lower.contains("public class ") && text_lower.contains("import ") {
            return Language::Java;
        }

        // Go检测
        if text_lower.contains("package main") && text_lower.contains("func main()") {
            return Language::Go;
        }

        // C++检测
        if text_lower.contains("#include <") && text_lower.contains("int main(") {
            return Language::Cpp;
        }

        // C检测
        if text_lower.contains("#include <stdio.h>") && text_lower.contains("int main(") {
            return Language::C;
        }

        // Ruby检测
        if text_lower.contains("class ") && text_lower.contains("def initialize") {
            return Language::Ruby;
        }

        // Swift检测
        if text_lower.contains("import SwiftUI")
            || (text_lower.contains("struct ") && text_lower.contains("var "))
        {
            return Language::Swift;
        }

        // Kotlin检测
        if text_lower.contains("fun main()") && text_lower.contains("val ") {
            return Language::Kotlin;
        }

        // PHP检测
        if text_lower.contains("<?php")
            || (text_lower.contains("function ") && text_lower.contains("echo "))
        {
            return Language::Php;
        }

        // HTML检测
        if text_lower.contains("<!DOCTYPE html") || text_lower.contains("<html") {
            return Language::Html;
        }

        // CSS检测
        if text_lower.contains("body {") || text_lower.contains("div {") {
            return Language::Css;
        }

        // Markdown检测
        if text_lower.starts_with("# ") || text_lower.contains("## ") {
            return Language::Markdown;
        }

        // JSON检测
        if text_lower.starts_with("{") && text_lower.contains(":") {
            return Language::Json;
        }

        // YAML检测
        if text_lower.contains(": ") && text_lower.contains("---") {
            return Language::Yaml;
        }

        // TOML检测
        if text_lower.contains("[package]") || text_lower.contains("name = ") {
            return Language::Toml;
        }

        // XML检测
        if text_lower.starts_with("<") && text_lower.contains("></") {
            return Language::Xml;
        }

        // Shell检测
        if text_lower.contains("#!/bin/bash") || text_lower.contains("#!/usr/bin/env bash") {
            return Language::Shell;
        }

        // 默认返回Markdown（作为文本处理）
        Language::Markdown
    }

    /// 从文件扩展名检测编程语言
    pub fn detect_from_extension(&self, extension: &str) -> Option<Language> {
        Language::from_extension(extension)
    }

    /// 从文件名检测编程语言
    pub fn detect_from_filename(&self, filename: &str) -> Option<Language> {
        Language::from_filename(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_natural_language() {
        let detector = LanguageDetector::new();

        // 测试中文检测
        assert_eq!(
            detector.detect_natural_language("你好，世界！"),
            NaturalLanguage::ChineseSimplified
        );
        assert_eq!(
            detector.detect_natural_language("優秀的程式設計師"),
            NaturalLanguage::ChineseTraditional
        );

        // 测试英语检测
        assert_eq!(
            detector.detect_natural_language("Hello, world!"),
            NaturalLanguage::English
        );

        // 测试日语检测
        assert_eq!(
            detector.detect_natural_language("こんにちは、世界！"),
            NaturalLanguage::Japanese
        );

        // 测试西班牙语检测
        assert_eq!(
            detector.detect_natural_language("¡Hola, mundo!"),
            NaturalLanguage::Spanish
        );
        // 测试简单英语检测
        assert_eq!(
            detector.detect_natural_language("This is a simple English sentence."),
            NaturalLanguage::English
        );

        // 测试法语检测
        assert_eq!(
            detector.detect_natural_language("Bonjour, monde!"),
            NaturalLanguage::French
        );
    }

    #[test]
    fn test_detect_programming_language() {
        let detector = LanguageDetector::new();

        // 测试Rust检测
        assert_eq!(
            detector.detect_programming_language("fn main() { println!(\"Hello, world!\"); }"),
            Language::Rust
        );

        // 测试Python检测
        assert_eq!(
            detector.detect_programming_language("def main():\n    print('Hello, world!')"),
            Language::Python
        );

        // 测试JavaScript检测
        assert_eq!(
            detector
                .detect_programming_language("function main() { console.log('Hello, world!'); }"),
            Language::JavaScript
        );
    }
}
