use codex::config::app::KnowledgeConfig;
use codex::knowledge::indexer::CodeIndexer;
use codex::parsers::initialize_parsers;
use std::path::Path;
use std::time::Instant;

/// 测试索引器性能
#[cfg(test)]
#[test]
fn test_indexer_performance() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 创建默认配置
    let config = KnowledgeConfig::default();

    // 创建索引器
    let mut indexer = CodeIndexer::new(config).unwrap();

    // 开始计时
    let start = Instant::now();

    // 索引当前目录下的Rust文件
    let result = indexer.index_directory(Path::new("."));

    // 结束计时
    let duration = start.elapsed();

    assert!(result.is_ok(), "索引失败");
    println!("索引当前目录下的Rust文件耗时: {:?}", duration);
    println!("索引的文件数量: {}", indexer.file_count());
}

/// 测试搜索性能
#[cfg(test)]
#[test]
fn test_search_performance() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 创建默认配置
    let config = KnowledgeConfig::default();

    // 创建索引器
    let mut indexer = CodeIndexer::new(config).unwrap();

    // 索引当前目录下的Rust文件
    indexer.index_directory(Path::new(".")).unwrap();

    // 开始计时
    let start = Instant::now();

    // 执行搜索
    let result = indexer.search("fn main");

    // 结束计时
    let duration = start.elapsed();

    assert!(result.is_ok(), "搜索失败");
    let results = result.unwrap();
    println!("搜索 'fn main' 耗时: {:?}", duration);
    println!("搜索结果数量: {}", results.len());
}

/// 测试内存使用情况
#[cfg(test)]
#[test]
fn test_memory_usage() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 创建默认配置
    let config = KnowledgeConfig::default();

    // 创建索引器
    let mut indexer = CodeIndexer::new(config).unwrap();

    // 索引当前目录下的Rust文件
    indexer.index_directory(Path::new(".")).unwrap();

    // 打印内存使用情况
    println!("索引器内存使用情况:");
    println!("- 索引的文件数量: {}", indexer.file_count());
    println!("- 索引的代码元素数量: {}", indexer.total_elements());
}

/// 测试并发搜索性能
#[cfg(test)]
#[test]
fn test_concurrent_search() {
    // 初始化解析器
    initialize_parsers().unwrap();

    // 并发搜索测试 - 每个线程使用自己的索引器实例
    let search_queries = ["fn main", "struct", "enum", "impl", "use"];
    let mut handles = vec![];

    // 开始计时
    let start = Instant::now();

    // 启动多个搜索线程
    for query in search_queries.iter() {
        let query_clone = query.to_string();
        let handle = std::thread::spawn(move || {
            // 每个线程创建自己的索引器实例
            let config = KnowledgeConfig::default();
            let mut indexer = CodeIndexer::new(config).unwrap();
            // 索引当前目录
            indexer.index_directory(Path::new(".")).unwrap();
            // 执行搜索
            indexer.search(&query_clone)
        });
        handles.push(handle);
    }

    // 等待所有搜索完成
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "并发搜索失败");
    }

    // 结束计时
    let duration = start.elapsed();

    println!(
        "并发搜索 {} 个查询耗时: {:?}",
        search_queries.len(),
        duration
    );
}
