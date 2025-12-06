
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
        
        // 中文检测：检查是否包含中文字符
        if text.chars().any(|c| matches!(c, '\u4e00'..='\u9fff')) {
            // 检查是否包含繁体中文字符
            let traditional_chars: HashSet<char> = [
                '優', '愛', '戰', '學', '數', '圖', '會', '裡', '車', '馬', '過', '後', '門', '開', '關', '電', '話'
            ].iter().cloned().collect();
            
            if text.chars().any(|c| traditional_chars.contains(&c)) {
                return NaturalLanguage::ChineseTraditional;
            } else {
                return NaturalLanguage::ChineseSimplified;
            }
        }
        
        // 日语检测：检查是否包含日语平假名或片假名
        if text.chars().any(|c| matches!(c, '\u3040'..='\u30ff')) {
            return NaturalLanguage::Japanese;
        }
        
        // 韩语检测：检查是否包含韩文字符
        if text.chars().any(|c| matches!(c, '\uac00'..='\ud7af')) {
            return NaturalLanguage::Korean;
        }
        
        // 西班牙语检测：检查是否包含常见西班牙语词汇或字符
        let spanish_words = ["el", "la", "los", "las", "de", "que", "y", "a", "en", "un", "una", "para", "con", "no", "su", "es"];
        let text_lower = text.to_lowercase();
        let spanish_count = spanish_words.iter().filter(|word| text_lower.contains(*word)).count();
        if spanish_count > text_lower.split_whitespace().count() / 10 {
            return NaturalLanguage::Spanish;
        }
        
        // 法语检测：检查是否包含常见法语词汇或字符
        let french_words = ["le", "la", "les", "de", "du", "des", "que", "et", "a", "en", "un", "une", "pour", "avec", "pas", "son", "est"];
        let french_count = french_words.iter().filter(|word| text_lower.contains(*word)).count();
        if french_count > text_lower.split_whitespace().count() / 10 {
            return NaturalLanguage::French;
        }
        
        // 德语检测：检查是否包含常见德语词汇或字符
        let german_words = ["der", "die", "das", "und", "in", "von", "zu", "ist", "nicht", "ein", "eine", "den", "mit", "dem", "auf", "für"];
        let german_count = german_words.iter().filter(|word| text_lower.contains(*word)).count();
        if german_count > text_lower.split_whitespace().count() / 10 {
            return NaturalLanguage::German;
        }
        
        // 俄语检测：检查是否包含俄语字符
        if text.chars().any(|c| matches!(c, '\u0400'..='\u04ff')) {
            return NaturalLanguage::Russian;
        }
        
        // 阿拉伯语检测：检查是否包含阿拉伯字符
        if text.chars().any(|c| matches!(c, '\u0600'..='\u06ff')) {
            return NaturalLanguage::Arabic;
        }
        
        // 葡萄牙语检测：检查是否包含常见葡萄牙语词汇或字符
        let portuguese_words = ["o", "a", "os", "as", "de", "que", "e", "a", "em", "um", "uma", "para", "com", "não", "seu", "é"];
        let portuguese_count = portuguese_words.iter().filter(|word| text_lower.contains(*word)).count();
        if portuguese_count > text_lower.split_whitespace().count() / 10 {
            return NaturalLanguage::Portuguese;
        }
        
        // 默认返回英语
        NaturalLanguage::English
    }
    
    /// 检测编程语言
    pub fn detect_programming_language(&self, text: &str) -> Language {
        // 基于代码特征的编程语言检测
        let text_lower = text.to_lowercase();
        
        // 检查各种编程语言的特征
        if text_lower.contains("fn ") && text_lower.contains("let ") && text_lower.contains("impl ") {
            return Language::Rust;
        }
        if text_lower.contains("def ") && text_lower.contains("import ") || text_lower.contains("from ") {
            return Language::Python;
        }
        if text_lower.contains("function ") && text_lower.contains("var ") || text_lower.contains("const ") {
            return Language::JavaScript;
        }
        if text_lower.contains("function ") && text_lower.contains("interface ") {
            return Language::TypeScript;
        }
        if text_lower.contains("public class ") && text_lower.contains("import ") {
            return Language::Java;
        }
        if text_lower.contains("package main") && text_lower.contains("func main()") {
            return Language::Go;
        }
        if text_lower.contains("#include <") && text_lower.contains("int main(") {
            return Language::Cpp;
        }
        if text_lower.contains("#include <stdio.h>") && text_lower.contains("int main(") {
            return Language::C;
        }
        if text_lower.contains("class ") && text_lower.contains("def initialize") {
            return Language::Ruby;
        }
        if text_lower.contains("import SwiftUI") || text_lower.contains("struct ") && text_lower.contains("var ") {
            return Language::Swift;
        }
        if text_lower.contains("fun main()") && text_lower.contains("val ") {
            return Language::Kotlin;
        }
        if text_lower.contains("<?php") || text_lower.contains("function ") && text_lower.contains("echo ") {
            return Language::Php;
        }
        if text_lower.contains("<!DOCTYPE html") || text_lower.contains("<html") {
            return Language::Html;
        }
        if text_lower.contains("body {") || text_lower.contains("div {") {
            return Language::Css;
        }
        if text_lower.starts_with("# ") || text_lower.contains("## ") {
            return Language::Markdown;
        }
        if text_lower.starts_with("{") && text_lower.contains(":") {
            return Language::Json;
        }
        if text_lower.contains(": ") && text_lower.contains("---") {
            return Language::Yaml;
        }
        if text_lower.contains("[package]") || text_lower.contains("name = ") {
            return Language::Toml;
        }
        if text_lower.starts_with("<") && text_lower.contains("></") {
            return Language::Xml;
        }
        if text_lower.contains("#!/bin/bash") || text_lower.contains("#!/usr/bin/env bash") {
            return Language::Shell;
        }
        
        // 默认返回英语（作为文本处理）
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
        assert_eq!(detector.detect_natural_language("你好，世界！"), NaturalLanguage::ChineseSimplified);
        assert_eq!(detector.detect_natural_language("優秀的程式設計師"), NaturalLanguage::ChineseTraditional);
        
        // 测试英语检测
        assert_eq!(detector.detect_natural_language("Hello, world!"), NaturalLanguage::English);
        
        // 测试日语检测
        assert_eq!(detector.detect_natural_language("こんにちは、世界！"), NaturalLanguage::Japanese);
        
        // 测试西班牙语检测
        assert_eq!(detector.detect_natural_language("¡Hola, mundo!"), NaturalLanguage::Spanish);
        
        // 测试法语检测
        assert_eq!(detector.detect_natural_language("Bonjour, monde!"), NaturalLanguage::French);
    }
    
    #[test]
    fn test_detect_programming_language() {
        let detector = LanguageDetector::new();
        
        // 测试Rust检测
        assert_eq!(detector.detect_programming_language("fn main() { println!(\"Hello, world!\"); }"), Language::Rust);
        
        // 测试Python检测
        assert_eq!(detector.detect_programming_language("def main():\n    print('Hello, world!')"), Language::Python);
        
        // 测试JavaScript检测
        assert_eq!(detector.detect_programming_language("function main() { console.log('Hello, world!'); }"), Language::JavaScript);
    }