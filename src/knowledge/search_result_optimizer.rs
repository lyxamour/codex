//! 搜索结果优化模块
//! 
//! 提供搜索结果的相关性排序、个性化排序、结果多样化和搜索意图理解功能

use crate::error::AppResult;
use crate::parsers::CodeElement;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// 搜索结果优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultOptimizerConfig {
    /// 相关性权重
    pub relevance_weight: f32,
    /// 个性化权重
    pub personalization_weight: f32,
    /// 多样性权重
    pub diversity_weight: f32,
    /// 时间权重
    pub time_weight: f32,
    /// 结果数量限制
    pub max_results: usize,
    /// 是否启用个性化排序
    pub enable_personalization: bool,
    /// 是否启用结果多样化
    pub enable_diversity: bool,
    /// 搜索意图理解阈值
    pub intent_understanding_threshold: f32,
}

impl Default for SearchResultOptimizerConfig {
    fn default() -> Self {
        Self {
            relevance_weight: 0.7,
            personalization_weight: 0.1,
            diversity_weight: 0.1,
            time_weight: 0.1,
            max_results: 20,
            enable_personalization: false,
            enable_diversity: true,
            intent_understanding_threshold: 0.8,
        }
    }
}

/// 搜索意图
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SearchIntent {
    /// 查找函数定义
    FunctionDefinition,
    /// 查找类定义
    ClassDefinition,
    /// 查找变量定义
    VariableDefinition,
    /// 查找特定功能
    FeatureSearch,
    /// 查找代码示例
    ExampleSearch,
    /// 查找文档
    DocumentationSearch,
    /// 查找错误修复
    ErrorFixSearch,
    /// 其他意图
    Other,
}

/// 搜索结果评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultScore {
    /// 相关性分数 (0-100)
    pub relevance: u8,
    /// 个性化分数 (0-100)
    pub personalization: u8,
    /// 多样性分数 (0-100)
    pub diversity: u8,
    /// 时间分数 (0-100)
    pub time: u8,
    /// 综合分数 (0-100)
    pub overall: u8,
    /// 匹配类型
    pub match_type: MatchType,
}

/// 匹配类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MatchType {
    /// 完全匹配
    Exact,
    /// 前缀匹配
    Prefix,
    /// 后缀匹配
    Suffix,
    /// 包含匹配
    Contains,
    /// 模糊匹配
    Fuzzy,
    /// 语义匹配
    Semantic,
}

/// 搜索结果条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// 代码元素
    pub element: CodeElement,
    /// 搜索结果评分
    pub score: SearchResultScore,
    /// 匹配的关键词
    pub matched_keywords: Vec<String>,
    /// 搜索意图
    pub intent: SearchIntent,
    /// 结果类型
    pub result_type: String,
    /// 相关性解释
    pub relevance_explanation: String,
}

/// 搜索意图理解结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIntentUnderstandingResult {
    /// 检测到的搜索意图
    pub intent: SearchIntent,
    /// 意图置信度 (0-1)
    pub confidence: f32,
    /// 提取的关键词
    pub keywords: Vec<String>,
    /// 意图相关的代码元素类型
    pub relevant_element_types: Vec<String>,
}

/// 搜索结果优化器
trait SearchResultOptimizer {
    /// 优化搜索结果
    fn optimize_results(&self, query: &str, results: Vec<CodeElement>) -> AppResult<Vec<SearchResultItem>>;
    
    /// 计算结果相关性
    fn calculate_relevance(&self, query: &str, element: &CodeElement) -> u8;
    
    /// 理解搜索意图
    fn understand_intent(&self, query: &str) -> SearchIntentUnderstandingResult;
    
    /// 个性化排序
    fn personalize_results(&self, query: &str, results: &mut Vec<SearchResultItem>);
    
    /// 结果多样化
    fn diversify_results(&self, results: &mut Vec<SearchResultItem>);
    
    /// 综合排序
    fn sort_results(&self, results: &mut Vec<SearchResultItem>);
}

/// 基于规则的搜索结果优化器实现
pub struct RuleBasedSearchResultOptimizer {
    /// 优化配置
    config: SearchResultOptimizerConfig,
    /// 用户搜索历史
    user_search_history: Vec<UserSearchHistoryItem>,
    /// 最大历史记录数量
    max_history_items: usize,
}

/// 用户搜索历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchHistoryItem {
    /// 搜索查询
    pub query: String,
    /// 搜索时间
    pub timestamp: u64,
    /// 点击的结果
    pub clicked_results: Vec<String>,
    /// 搜索意图
    pub intent: SearchIntent,
}

impl RuleBasedSearchResultOptimizer {
    /// 创建新的搜索结果优化器
    pub fn new(config: SearchResultOptimizerConfig) -> Self {
        Self {
            config,
            user_search_history: Vec::new(),
            max_history_items: 100,
        }
    }
    
    /// 添加用户搜索历史
    pub fn add_search_history(&mut self, history_item: UserSearchHistoryItem) {
        self.user_search_history.push(history_item);
        
        // 限制历史记录数量
        if self.user_search_history.len() > self.max_history_items {
            self.user_search_history.remove(0);
        }
    }
    
    /// 获取用户搜索历史
    pub fn get_search_history(&self) -> &Vec<UserSearchHistoryItem> {
        &self.user_search_history
    }
}

impl SearchResultOptimizer for RuleBasedSearchResultOptimizer {
    fn optimize_results(&self, query: &str, results: Vec<CodeElement>) -> AppResult<Vec<SearchResultItem>> {
        // 理解搜索意图
        let intent_understanding = self.understand_intent(query);
        
        // 处理搜索结果，计算评分
        let mut result_items: Vec<SearchResultItem> = results
            .into_iter()
            .map(|element| {
                // 计算相关性分数
                let relevance = self.calculate_relevance(query, &element);
                
                // 计算个性化分数
                let personalization = if self.config.enable_personalization {
                    self.calculate_personalization(&element)
                } else {
                    50
                };
                
                // 计算多样性分数
                let diversity = 50; // 初始值，后续在多样化处理中更新
                
                // 计算时间分数
                let time = self.calculate_time_score(&element);
                
                // 计算综合分数
                let overall = self.calculate_overall_score(relevance, personalization, diversity, time);
                
                // 确定匹配类型
                let match_type = self.determine_match_type(query, &element);
                
                // 提取匹配的关键词
                let matched_keywords = self.extract_matched_keywords(query, &element);
                
                SearchResultItem {
                    element,
                    score: SearchResultScore {
                        relevance,
                        personalization,
                        diversity,
                        time,
                        overall,
                        match_type,
                    },
                    matched_keywords,
                    intent: intent_understanding.intent,
                    result_type: "code_element".to_string(),
                    relevance_explanation: self.generate_relevance_explanation(query, &element, relevance),
                }
            })
            .collect();
        
        // 个性化排序
        if self.config.enable_personalization {
            self.personalize_results(query, &mut result_items);
        }
        
        // 综合排序
        self.sort_results(&mut result_items);
        
        // 结果多样化
        if self.config.enable_diversity {
            self.diversify_results(&mut result_items);
        }
        
        // 限制结果数量
        if result_items.len() > self.config.max_results {
            result_items.truncate(self.config.max_results);
        }
        
        Ok(result_items)
    }
    
    fn calculate_relevance(&self, query: &str, element: &CodeElement) -> u8 {
        // 实现基于规则的相关性计算
        let mut score = 0;
        
        // 1. 完全匹配名称
        if element.name == query {
            score += 40;
        }
        
        // 2. 名称包含关键词
        if element.name.contains(query) {
            score += 20;
        }
        
        // 3. 文档包含关键词
        if let Some(doc) = &element.doc {
            if doc.contains(query) {
                score += 15;
            }
        }
        
        // 4. 代码包含关键词
        if element.code.contains(query) {
            score += 10;
        }
        
        // 5. 基于元素类型的权重
        let type_weight = match element.element_type {
            crate::parsers::CodeElementType::Function => 15,
            crate::parsers::CodeElementType::Class => 15,
            crate::parsers::CodeElementType::Method => 10,
            crate::parsers::CodeElementType::Variable => 5,
            crate::parsers::CodeElementType::Constant => 5,
            crate::parsers::CodeElementType::Interface => 12,
            crate::parsers::CodeElementType::Struct => 12,
            crate::parsers::CodeElementType::Enum => 10,
            crate::parsers::CodeElementType::Module => 8,
            _ => 5,
        };
        score += type_weight;
        
        // 确保分数在0-100之间
        score.clamp(0, 100) as u8
    }
    
    fn understand_intent(&self, query: &str) -> SearchIntentUnderstandingResult {
        // 实现基于规则的搜索意图理解
        let query_lower = query.to_lowercase();
        let mut keywords = Vec::new();
        let mut relevant_element_types = Vec::new();
        
        // 提取关键词
        keywords = self.extract_keywords(&query_lower);
        
        // 检测搜索意图
        let (intent, confidence) = self.detect_intent(&query_lower, &keywords);
        
        // 根据意图确定相关的代码元素类型
        match intent {
            SearchIntent::FunctionDefinition => {
                relevant_element_types.push("function".to_string());
                relevant_element_types.push("method".to_string());
            },
            SearchIntent::ClassDefinition => {
                relevant_element_types.push("class".to_string());
                relevant_element_types.push("struct".to_string());
                relevant_element_types.push("interface".to_string());
            },
            SearchIntent::VariableDefinition => {
                relevant_element_types.push("variable".to_string());
                relevant_element_types.push("constant".to_string());
            },
            SearchIntent::FeatureSearch => {
                relevant_element_types.push("function".to_string());
                relevant_element_types.push("class".to_string());
                relevant_element_types.push("method".to_string());
            },
            SearchIntent::ExampleSearch => {
                relevant_element_types.push("function".to_string());
                relevant_element_types.push("class".to_string());
            },
            SearchIntent::DocumentationSearch => {
                relevant_element_types.push("class".to_string());
                relevant_element_types.push("function".to_string());
                relevant_element_types.push("interface".to_string());
            },
            SearchIntent::ErrorFixSearch => {
                relevant_element_types.push("function".to_string());
                relevant_element_types.push("method".to_string());
            },
            SearchIntent::Other => {
                // 所有类型都相关
                relevant_element_types.push("all".to_string());
            },
        }
        
        SearchIntentUnderstandingResult {
            intent,
            confidence,
            keywords,
            relevant_element_types,
        }
    }
    
    fn personalize_results(&self, query: &str, results: &mut Vec<SearchResultItem>) {
        // 实现基于用户历史的个性化排序
        // TODO: 主人~ 这里需要实现基于用户搜索历史的个性化排序逻辑
        // 提示：根据用户历史搜索和点击记录，调整结果的个性化分数
        
        // 简单实现：如果用户历史中搜索过类似的查询，提高相关结果的个性化分数
        for result in results {
            // 检查用户历史中是否有类似的查询
            for history_item in &self.user_search_history {
                if self.is_similar_query(query, &history_item.query) {
                    // 提高个性化分数
                    result.score.personalization = std::cmp::min(100, result.score.personalization + 10);
                    break;
                }
            }
        }
    }
    
    fn diversify_results(&self, results: &mut Vec<SearchResultItem>) {
        // 实现结果多样化，避免结果过于集中
        let mut diversified_results = Vec::new();
        let mut seen_types = HashSet::new();
        let mut seen_files = HashSet::new();
        
        // 第一轮：选择不同类型的结果
        for result in results.iter() {
            let element_type = format!("{:?}", result.element.element_type);
            if !seen_types.contains(&element_type) {
                diversified_results.push(result.clone());
                seen_types.insert(element_type);
                seen_files.insert(result.element.file_path.clone());
            }
        }
        
        // 第二轮：选择不同文件的结果
        for result in results.iter() {
            if diversified_results.len() >= self.config.max_results {
                break;
            }
            let file_path = result.element.file_path.clone();
            if !seen_files.contains(&file_path) {
                diversified_results.push(result.clone());
                seen_files.insert(file_path);
            }
        }
        
        // 第三轮：填充剩余结果
        for result in results.iter() {
            if diversified_results.len() >= self.config.max_results {
                break;
            }
            if !diversified_results.contains(result) {
                diversified_results.push(result.clone());
            }
        }
        
        // 更新结果列表
        *results = diversified_results;
    }
    
    fn sort_results(&self, results: &mut Vec<SearchResultItem>) {
        // 实现基于综合分数的排序
        results.sort_by(|a, b| {
            // 首先按综合分数降序排序
            let score_cmp = b.score.overall.cmp(&a.score.overall);
            if score_cmp != std::cmp::Ordering::Equal {
                return score_cmp;
            }
            
            // 其次按相关性分数降序排序
            let relevance_cmp = b.score.relevance.cmp(&a.score.relevance);
            if relevance_cmp != std::cmp::Ordering::Equal {
                return relevance_cmp;
            }
            
            // 最后按时间分数降序排序
            b.score.time.cmp(&a.score.time)
        });
    }
}

impl RuleBasedSearchResultOptimizer {
    /// 计算综合分数
    fn calculate_overall_score(&self, relevance: u8, personalization: u8, diversity: u8, time: u8) -> u8 {
        let overall = (
            relevance as f32 * self.config.relevance_weight +
            personalization as f32 * self.config.personalization_weight +
            diversity as f32 * self.config.diversity_weight +
            time as f32 * self.config.time_weight
        ) as u8;
        
        overall.clamp(0, 100)
    }
    
    /// 计算个性化分数
    fn calculate_personalization(&self, element: &CodeElement) -> u8 {
        // TODO: 主人~ 这里需要实现基于用户历史的个性化分数计算
        // 提示：根据用户历史搜索和点击记录，计算元素的个性化分数
        50 // 默认值
    }
    
    /// 计算时间分数
    fn calculate_time_score(&self, element: &CodeElement) -> u8 {
        // TODO: 主人~ 这里需要实现基于时间的分数计算
        // 提示：根据文件修改时间，计算元素的时间分数
        50 // 默认值
    }
    
    /// 确定匹配类型
    fn determine_match_type(&self, query: &str, element: &CodeElement) -> MatchType {
        let query_lower = query.to_lowercase();
        let name_lower = element.name.to_lowercase();
        let doc_lower = element.doc.as_ref().map(|d| d.to_lowercase()).unwrap_or_default();
        let code_lower = element.code.to_lowercase();
        
        // 完全匹配
        if name_lower == query_lower {
            return MatchType::Exact;
        }
        
        // 前缀匹配
        if name_lower.starts_with(&query_lower) {
            return MatchType::Prefix;
        }
        
        // 后缀匹配
        if name_lower.ends_with(&query_lower) {
            return MatchType::Suffix;
        }
        
        // 包含匹配
        if name_lower.contains(&query_lower) || doc_lower.contains(&query_lower) {
            return MatchType::Contains;
        }
        
        // 模糊匹配（简单实现：检查关键词是否有重叠）
        if self.has_keyword_overlap(&query_lower, &name_lower) {
            return MatchType::Fuzzy;
        }
        
        // 默认语义匹配
        MatchType::Semantic
    }
    
    /// 提取匹配的关键词
    fn extract_matched_keywords(&self, query: &str, element: &CodeElement) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let keywords = self.extract_keywords(&query_lower);
        let name_lower = element.name.to_lowercase();
        let doc_lower = element.doc.as_ref().map(|d| d.to_lowercase()).unwrap_or_default();
        let code_lower = element.code.to_lowercase();
        
        keywords
            .into_iter()
            .filter(|keyword| {
                name_lower.contains(keyword) || doc_lower.contains(keyword) || code_lower.contains(keyword)
            })
            .collect()
    }
    
    /// 提取关键词
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        // 简单的关键词提取：分割为单词，去停用词
        let stop_words = self.get_stop_words();
        query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty() && !stop_words.contains(word))
            .map(|word| word.to_string())
            .collect()
    }
    
    /// 获取停用词列表
    fn get_stop_words(&self) -> HashSet<&str> {
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "with", "by",
            "of", "from", "about", "into", "through", "after", "before", "during", "above",
            "below", "up", "down", "out", "over", "under", "again", "further", "then", "once",
            "here", "there", "when", "where", "why", "how", "all", "any", "both", "each",
            "few", "more", "most", "other", "some", "such", "no", "nor", "not", "only",
            "own", "same", "so", "than", "too", "very", "s", "t", "can", "will", "just",
            "don", "should", "now", "的", "了", "和", "是", "就", "都", "而", "及", "与",
            "着", "或", "要", "在", "有", "来", "去", "你", "我", "他", "她", "它", "们"
        ];
        HashSet::from_iter(stop_words)
    }
    
    /// 检测搜索意图
    fn detect_intent(&self, query: &str, keywords: &[String]) -> (SearchIntent, f32) {
        // 简单的意图检测规则
        let query_lower = query.to_lowercase();
        
        // 检测函数定义意图
        if query_lower.contains("function") || query_lower.contains("def") || query_lower.contains("func") {
            return (SearchIntent::FunctionDefinition, 0.9);
        }
        
        // 检测类定义意图
        if query_lower.contains("class") || query_lower.contains("struct") || query_lower.contains("interface") {
            return (SearchIntent::ClassDefinition, 0.9);
        }
        
        // 检测变量定义意图
        if query_lower.contains("variable") || query_lower.contains("var") || query_lower.contains("const") {
            return (SearchIntent::VariableDefinition, 0.8);
        }
        
        // 检测文档搜索意图
        if query_lower.contains("doc") || query_lower.contains("documentation") || query_lower.contains("help") {
            return (SearchIntent::DocumentationSearch, 0.8);
        }
        
        // 检测示例搜索意图
        if query_lower.contains("example") || query_lower.contains("demo") || query_lower.contains("sample") {
            return (SearchIntent::ExampleSearch, 0.8);
        }
        
        // 检测错误修复意图
        if query_lower.contains("fix") || query_lower.contains("bug") || query_lower.contains("error") {
            return (SearchIntent::ErrorFixSearch, 0.8);
        }
        
        // 默认意图
        (SearchIntent::FeatureSearch, 0.7)
    }
    
    /// 生成相关性解释
    fn generate_relevance_explanation(&self, query: &str, element: &CodeElement, relevance: u8) -> String {
        let mut explanation = Vec::new();
        
        if element.name.to_lowercase().contains(&query.to_lowercase()) {
            explanation.push("名称包含关键词".to_string());
        }
        
        if element.doc.as_ref().map(|d| d.to_lowercase().contains(&query.to_lowercase())).unwrap_or(false) {
            explanation.push("文档包含关键词".to_string());
        }
        
        if element.code.to_lowercase().contains(&query.to_lowercase()) {
            explanation.push("代码包含关键词".to_string());
        }
        
        if explanation.is_empty() {
            explanation.push("基于语义相关性匹配".to_string());
        }
        
        explanation.join("，")
    }
    
    /// 检查关键词重叠
    fn has_keyword_overlap(&self, query: &str, text: &str) -> bool {
        let query_keywords = self.extract_keywords(query);
        let text_keywords = self.extract_keywords(text);
        
        // 检查是否有共同关键词
        for query_keyword in &query_keywords {
            for text_keyword in &text_keywords {
                if query_keyword == text_keyword {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// 检查查询是否相似
    fn is_similar_query(&self, query1: &str, query2: &str) -> bool {
        // 简单实现：检查关键词重叠
        let keywords1 = self.extract_keywords(query1);
        let keywords2 = self.extract_keywords(query2);
        
        // 计算关键词重叠比例
        if keywords1.is_empty() || keywords2.is_empty() {
            return false;
        }
        
        let common_keywords = keywords1
            .iter()
            .filter(|k| keywords2.contains(k))
            .count();
        
        let overlap_ratio = common_keywords as f32 / std::cmp::min(keywords1.len(), keywords2.len()) as f32;
        
        overlap_ratio >= 0.5
    }
}

/// 搜索结果优化器工厂
pub struct SearchResultOptimizerFactory {
    /// 默认配置
    default_config: SearchResultOptimizerConfig,
}

impl SearchResultOptimizerFactory {
    /// 创建新的搜索结果优化器工厂
    pub fn new() -> Self {
        Self {
            default_config: SearchResultOptimizerConfig::default(),
        }
    }
    
    /// 创建搜索结果优化器实例
    pub fn create_optimizer(&self, config: Option<SearchResultOptimizerConfig>) -> Box<dyn SearchResultOptimizer> {
        let config = config.unwrap_or(self.default_config.clone());
        Box::new(RuleBasedSearchResultOptimizer::new(config))
    }
    
    /// 创建默认配置的搜索结果优化器
    pub fn create_default_optimizer(&self) -> Box<dyn SearchResultOptimizer> {
        self.create_optimizer(None)
    }
}
