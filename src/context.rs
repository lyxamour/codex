use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Context item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    /// Item ID
    id: String,
    /// Content of the context item
    content: String,
    /// Item type
    item_type: ContextItemType,
    /// Importance score (0-100)
    importance: u8,
    /// Creation timestamp (Unix seconds)
    created_at: i64,
    /// Last accessed timestamp (Unix seconds)
    last_accessed: i64,
    /// Reference count
    ref_count: u32,
    /// Token count (approximate)
    token_count: usize,
    /// Tags for categorization
    tags: Vec<String>,
}

/// Context item type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextItemType {
    UserMessage,
    AIMessage,
    CodeSnippet,
    KnowledgeBaseEntry,
    SystemPrompt,
    ToolResult,
    Other,
}

impl From<ContextItemType> for &'static str {
    fn from(item_type: ContextItemType) -> Self {
        match item_type {
            ContextItemType::UserMessage => "user",
            ContextItemType::AIMessage => "ai",
            ContextItemType::CodeSnippet => "code",
            ContextItemType::KnowledgeBaseEntry => "kb",
            ContextItemType::SystemPrompt => "system",
            ContextItemType::ToolResult => "tool",
            ContextItemType::Other => "other",
        }
    }
}

/// Context compression strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionStrategy {
    /// Keep most recent items first
    RecentFirst,
    /// Keep most important items first
    ImportanceFirst,
    /// Hybrid of recent and importance
    Hybrid,
    /// Compress based on token count
    TokenBased,
    /// No compression (keep all items)
    None,
}

/// Context manager for handling context auto-compression
pub struct ContextManager {
    context: VecDeque<ContextItem>,
    max_tokens: usize,
    strategy: CompressionStrategy,
    tokenizer: Arc<dyn Tokenizer>,
    importance_weights: HashMap<ContextItemType, f32>,
}

/// Tokenizer trait for estimating token counts
pub trait Tokenizer: Sync + Send {
    fn count_tokens(&self, text: &str) -> usize;
}

/// Simple tokenizer implementation (word-based)
pub struct SimpleTokenizer;

impl Tokenizer for SimpleTokenizer {
    fn count_tokens(&self, text: &str) -> usize {
        // Simple word-based tokenization for estimation
        text.split_whitespace().count()
    }
}

/// Default tokenizer implementation (token-based)
pub struct DefaultTokenizer;

impl Tokenizer for DefaultTokenizer {
    fn count_tokens(&self, text: &str) -> usize {
        // Estimate tokens: ~4 chars per token for English
        (text.len() as f32 / 4.0).ceil() as usize
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(8192, CompressionStrategy::Hybrid)
    }
}

impl ContextManager {
    /// Create a new context manager with specified max tokens and strategy
    pub fn new(max_tokens: usize, strategy: CompressionStrategy) -> Self {
        let mut importance_weights = HashMap::new();
        importance_weights.insert(ContextItemType::SystemPrompt, 1.0);
        importance_weights.insert(ContextItemType::UserMessage, 0.9);
        importance_weights.insert(ContextItemType::AIMessage, 0.8);
        importance_weights.insert(ContextItemType::CodeSnippet, 0.7);
        importance_weights.insert(ContextItemType::KnowledgeBaseEntry, 0.6);
        importance_weights.insert(ContextItemType::ToolResult, 0.5);
        importance_weights.insert(ContextItemType::Other, 0.4);
        
        Self {
            context: VecDeque::new(),
            max_tokens,
            strategy,
            tokenizer: Arc::new(DefaultTokenizer),
            importance_weights,
        }
    }
    
    /// Add a new item to the context
    pub fn add_item(&mut self, item: ContextItem) {
        self.context.push_back(item);
        self.compress()
    }
    
    /// Add a user message to the context
    pub fn add_user_message(&mut self, content: &str) {
        let item = self.create_context_item(
            content,
            ContextItemType::UserMessage,
            90,
            vec!["user".to_string()]
        );
        self.add_item(item);
    }
    
    /// Add an AI message to the context
    pub fn add_ai_message(&mut self, content: &str) {
        let item = self.create_context_item(
            content,
            ContextItemType::AIMessage,
            80,
            vec!["ai".to_string()]
        );
        self.add_item(item);
    }
    
    /// Add a code snippet to the context
    pub fn add_code_snippet(&mut self, content: &str, language: &str) {
        let item = self.create_context_item(
            content,
            ContextItemType::CodeSnippet,
            70,
            vec!["code".to_string(), language.to_string()]
        );
        self.add_item(item);
    }
    
    /// Add a knowledge base entry to the context
    pub fn add_knowledge_entry(&mut self, content: &str, source: &str) {
        let item = self.create_context_item(
            content,
            ContextItemType::KnowledgeBaseEntry,
            60,
            vec!["kb".to_string(), source.to_string()]
        );
        self.add_item(item);
    }
    
    /// Create a context item with default values
    fn create_context_item(
        &self,
        content: &str,
        item_type: ContextItemType,
        importance: u8,
        tags: Vec<String>
    ) -> ContextItem {
        let now = chrono::Utc::now().timestamp();
        let token_count = self.tokenizer.count_tokens(content);
        
        ContextItem {
            id: format!("{}-{}", now, uuid::Uuid::new_v4()),
            content: content.to_string(),
            item_type,
            importance,
            created_at: now,
            last_accessed: now,
            ref_count: 0,
            token_count,
            tags,
        }
    }
    
    /// Get the current total token count
    pub fn total_tokens(&self) -> usize {
        self.context.iter()
            .map(|item| item.token_count)
            .sum()
    }
    
    /// Get the current context as a vector
    pub fn get_context(&self) -> Vec<ContextItem> {
        self.context.clone().into_iter().collect()
    }
    
    /// Compress the context based on the selected strategy
    pub fn compress(&mut self) {
        if self.strategy == CompressionStrategy::None {
            return;
        }
        
        let total_tokens = self.total_tokens();
        if total_tokens <= self.max_tokens {
            return;
        }
        
        // Calculate target token count (80% of max to leave room for new items)
        let target_tokens = (self.max_tokens as f32 * 0.8) as usize;
        
        println!("Compressing context: {} tokens -> {} tokens", total_tokens, target_tokens);
        
        match self.strategy {
            CompressionStrategy::RecentFirst => {
                self.compress_recent_first(target_tokens);
            }
            CompressionStrategy::ImportanceFirst => {
                self.compress_importance_first(target_tokens);
            }
            CompressionStrategy::Hybrid => {
                self.compress_hybrid(target_tokens);
            }
            CompressionStrategy::TokenBased => {
                self.compress_token_based(target_tokens);
            }
            CompressionStrategy::None => {
                // Do nothing
            }
        }
        
        println!("Compressed to {} tokens", self.total_tokens());
    }
    
    /// Compress by keeping most recent items first
    fn compress_recent_first(&mut self, target_tokens: usize) {
        let mut total = 0;
        let mut compressed = VecDeque::new();
        
        // Iterate from most recent to oldest
        for item in self.context.iter().rev() {
            total += item.token_count;
            if total > target_tokens {
                break;
            }
            compressed.push_front(item.clone());
        }
        
        self.context = compressed;
    }
    
    /// Compress by keeping most important items first
    fn compress_importance_first(&mut self, target_tokens: usize) {
        // Sort items by importance (descending)
        let mut items: Vec<_> = self.context.iter().cloned().collect();
        items.sort_by(|a, b| {
            // First sort by importance
            let importance_cmp = b.importance.cmp(&a.importance);
            if importance_cmp != std::cmp::Ordering::Equal {
                return importance_cmp;
            }
            
            // Then by recency
            b.created_at.cmp(&a.created_at)
        });
        
        // Select top items up to target tokens
        let mut total = 0;
        let mut compressed = VecDeque::new();
        
        for item in items {
            total += item.token_count;
            if total > target_tokens {
                break;
            }
            compressed.push_back(item);
        }
        
        // Sort back by created_at for chronological order
        let mut sorted_compressed: Vec<_> = compressed.into_iter().collect();
        sorted_compressed.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        self.context = sorted_compressed.into_iter().collect();
    }
    
    /// Hybrid compression: combine importance and recency
    fn compress_hybrid(&mut self, target_tokens: usize) {
        // Calculate hybrid score for each item
        let now = chrono::Utc::now().timestamp();
        let mut scored_items: Vec<(f32, ContextItem)> = Vec::new();
        
        for item in self.context.iter().cloned() {
            // Time decay factor (older items have lower score)
            let age_seconds = now - item.created_at;
            let time_decay = if age_seconds < 3600 { // 1 hour
                1.0
            } else if age_seconds < 86400 { // 1 day
                0.7
            } else if age_seconds < 604800 { // 1 week
                0.4
            } else {
                0.1
            };
            
            // Importance weight based on item type
            let type_weight = *self.importance_weights.get(&item.item_type).unwrap_or(&0.5);
            
            // Reference count factor
            let ref_factor = (item.ref_count as f32 + 1.0).ln() / 10.0;
            
            // Calculate hybrid score
            let hybrid_score = (
                (item.importance as f32 / 100.0) * type_weight * 0.6 +
                time_decay * 0.3 +
                ref_factor * 0.1
            );
            
            scored_items.push((hybrid_score, item));
        }
        
        // Sort by hybrid score (descending)
        scored_items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Select top items up to target tokens
        let mut total = 0;
        let mut compressed = VecDeque::new();
        
        for (_, item) in scored_items {
            total += item.token_count;
            if total > target_tokens {
                break;
            }
            compressed.push_back(item);
        }
        
        // Sort back by created_at for chronological order
        let mut sorted_compressed: Vec<_> = compressed.into_iter().collect();
        sorted_compressed.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        self.context = sorted_compressed.into_iter().collect();
    }
    
    /// Token-based compression: prioritize smaller items
    fn compress_token_based(&mut self, target_tokens: usize) {
        // Sort items by token count (ascending) + importance (descending)
        let mut items: Vec<_> = self.context.iter().cloned().collect();
        items.sort_by(|a, b| {
            // First sort by token count (smaller first)
            let token_cmp = a.token_count.cmp(&b.token_count);
            if token_cmp != std::cmp::Ordering::Equal {
                return token_cmp;
            }
            
            // Then by importance (higher first)
            b.importance.cmp(&a.importance)
        });
        
        // Select items up to target tokens
        let mut total = 0;
        let mut compressed = VecDeque::new();
        
        for item in items {
            total += item.token_count;
            if total > target_tokens {
                break;
            }
            compressed.push_back(item);
        }
        
        // Sort back by created_at for chronological order
        let mut sorted_compressed: Vec<_> = compressed.into_iter().collect();
        sorted_compressed.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        self.context = sorted_compressed.into_iter().collect();
    }
    
    /// Clear the entire context
    pub fn clear(&mut self) {
        self.context.clear();
    }
    
    /// Remove specific item by ID
    pub fn remove_item(&mut self, id: &str) {
        self.context.retain(|item| item.id != id);
    }
    
    /// Update importance of an item
    pub fn update_importance(&mut self, id: &str, importance: u8) {
        for item in self.context.iter_mut() {
            if item.id == id {
                item.importance = importance;
                break;
            }
        }
    }
    
    /// Increment reference count for an item
    pub fn increment_ref_count(&mut self, id: &str) {
        for item in self.context.iter_mut() {
            if item.id == id {
                item.ref_count += 1;
                item.last_accessed = chrono::Utc::now().timestamp();
                break;
            }
        }
    }
    
    /// Export context to a string
    pub fn export(&self, format: ContextExportFormat) -> Result<String, Box<dyn Error>> {
        match format {
            ContextExportFormat::Text => {
                let mut output = String::new();
                for item in &self.context {
                    let item_type: &str = item.item_type.into();
                    output.push_str(&format!("[{}] {}\n{}\n\n", 
                        item_type,
                        item.created_at,
                        item.content
                    ));
                }
                Ok(output)
            }
            ContextExportFormat::Json => {
                serde_json::to_string_pretty(&self.context)
                    .map_err(|e| e.into())
            }
            ContextExportFormat::Yaml => {
                serde_yaml::to_string(&self.context)
                    .map_err(|e| e.into())
            }
        }
    }
    
    /// Import context from a string
    pub fn import(&mut self, data: &str, format: ContextExportFormat) -> Result<(), Box<dyn Error>> {
        match format {
            ContextExportFormat::Json => {
                let items: Vec<ContextItem> = serde_json::from_str(data)?;
                self.context.extend(items);
                self.compress();
                Ok(())
            }
            ContextExportFormat::Yaml => {
                let items: Vec<ContextItem> = serde_yaml::from_str(data)?;
                self.context.extend(items);
                self.compress();
                Ok(())
            }
            ContextExportFormat::Text => {
                // Text format import not implemented yet
                Err("Text format import not implemented".into())
            }
        }
    }
    
    /// Set a custom tokenizer
    pub fn set_tokenizer(&mut self, tokenizer: Arc<dyn Tokenizer>) {
        self.tokenizer = tokenizer;
        // Recalculate token counts
        for item in self.context.iter_mut() {
            item.token_count = self.tokenizer.count_tokens(&item.content);
        }
    }
    
    /// Set importance weights for different item types
    pub fn set_importance_weights(&mut self, weights: HashMap<ContextItemType, f32>) {
        self.importance_weights = weights;
    }
    
    /// Set maximum token limit
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.max_tokens = max_tokens;
        self.compress();
    }
    
    /// Set compression strategy
    pub fn set_strategy(&mut self, strategy: CompressionStrategy) {
        self.strategy = strategy;
        self.compress();
    }
}

/// Context export format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextExportFormat {
    Text,
    Json,
    Yaml,
}

/// Context summary structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    /// Total items in context
    pub total_items: usize,
    /// Total token count
    pub total_tokens: usize,
    /// Item type distribution
    pub type_distribution: HashMap<String, usize>,
    /// Average importance score
    pub avg_importance: f32,
    /// Oldest item timestamp
    pub oldest_item: i64,
    /// Newest item timestamp
    pub newest_item: i64,
}

impl ContextManager {
    /// Generate a summary of the current context
    pub fn get_summary(&self) -> ContextSummary {
        let mut type_distribution = HashMap::new();
        let mut total_importance = 0;
        let mut oldest = i64::MAX;
        let mut newest = i64::MIN;
        
        for item in &self.context {
            let item_type: &str = item.item_type.into();
            *type_distribution.entry(item_type.to_string()).or_insert(0) += 1;
            total_importance += item.importance as u32;
            oldest = oldest.min(item.created_at);
            newest = newest.max(item.created_at);
        }
        
        let avg_importance = if self.context.is_empty() {
            0.0
        } else {
            total_importance as f32 / self.context.len() as f32
        };
        
        ContextSummary {
            total_items: self.context.len(),
            total_tokens: self.total_tokens(),
            type_distribution,
            avg_importance,
            oldest_item: oldest,
            newest_item: newest,
        }
    }
    
    /// Print context summary
    pub fn print_summary(&self) {
        let summary = self.get_summary();
        println!("Context Summary:");
        println!("  Total Items: {}", summary.total_items);
        println!("  Total Tokens: {}", summary.total_tokens);
        println!("  Average Importance: {:.2}", summary.avg_importance);
        println!("  Item Type Distribution:");
        for (item_type, count) in summary.type_distribution {
            println!("    - {}: {}", item_type, count);
        }
        println!("  Time Range: {} to {}", 
            chrono::NaiveDateTime::from_timestamp(summary.oldest_item, 0)
                .format("%Y-%m-%d %H:%M:%S"),
            chrono::NaiveDateTime::from_timestamp(summary.newest_item, 0)
                .format("%Y-%m-%d %H:%M:%S")
        );
    }
}

/// Context manager builder for easy configuration
pub struct ContextManagerBuilder {
    max_tokens: usize,
    strategy: CompressionStrategy,
    tokenizer: Option<Arc<dyn Tokenizer>>,
    importance_weights: Option<HashMap<ContextItemType, f32>>,
}

impl ContextManagerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            max_tokens: 8192,
            strategy: CompressionStrategy::Hybrid,
            tokenizer: None,
            importance_weights: None,
        }
    }
    
    /// Set maximum token limit
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }
    
    /// Set compression strategy
    pub fn strategy(mut self, strategy: CompressionStrategy) -> Self {
        self.strategy = strategy;
        self
    }
    
    /// Set custom tokenizer
    pub fn tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self {
        self.tokenizer = Some(tokenizer);
        self
    }
    
    /// Set importance weights
    pub fn importance_weights(mut self, weights: HashMap<ContextItemType, f32>) -> Self {
        self.importance_weights = Some(weights);
        self
    }
    
    /// Build the context manager
    pub fn build(self) -> ContextManager {
        let mut manager = ContextManager::new(self.max_tokens, self.strategy);
        
        if let Some(tokenizer) = self.tokenizer {
            manager.set_tokenizer(tokenizer);
        }
        
        if let Some(weights) = self.importance_weights {
            manager.set_importance_weights(weights);
        }
        
        manager
    }
}
