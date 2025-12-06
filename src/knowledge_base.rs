use clap::Subcommand;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use sled::Db;
use tantivy::collector::{Count, TopDocs};
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use walkdir::WalkDir;

/// Static schema definition for the knowledge base
static SCHEMA: Lazy<Schema> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("content", TEXT | STORED);
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("language", TEXT | STORED);
    schema_builder.add_i64_field("timestamp", FAST);
    schema_builder.build()
});

/// Static field accessors
static CONTENT_FIELD: Lazy<tantivy::schema::Field> =
    Lazy::new(|| SCHEMA.get_field("content").unwrap());
static PATH_FIELD: Lazy<tantivy::schema::Field> = Lazy::new(|| SCHEMA.get_field("path").unwrap());
static LANGUAGE_FIELD: Lazy<tantivy::schema::Field> =
    Lazy::new(|| SCHEMA.get_field("language").unwrap());
static TIMESTAMP_FIELD: Lazy<tantivy::schema::Field> =
    Lazy::new(|| SCHEMA.get_field("timestamp").unwrap());

/// Knowledge base structure
pub struct KnowledgeBase {
    /// Sled database for metadata storage
    db: Db,
    /// Tantivy index for full-text search
    index: Index,
    /// Index writer (wrapped in Mutex for thread safety)
    index_writer: Mutex<IndexWriter>,
    /// Searcher
    searcher: tantivy::Searcher,
}

/// Knowledge base actions (same as in main.rs for now)
#[derive(Debug, Clone, Subcommand)]
pub enum KnowledgeActions {
    Add { paths: Vec<String>, recursive: bool },
    Search { query: String, limit: usize },
    List { details: bool },
    Clear { confirm: bool },
}

impl KnowledgeBase {
    /// Create a new knowledge base instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Create or open Sled database
        let db = sled::open(".codex_kb")?;

        // Create or open Tantivy index
        let index_path = Path::new(".codex_index");
        let directory = MmapDirectory::open(index_path)?;
        let index = Index::open_or_create(directory, SCHEMA.clone())?;

        // Create index writer
        let index_writer = index.writer(50_000_000)?; // 50MB buffer

        // Create searcher
        let searcher = index.reader()?.searcher();

        Ok(Self {
            db,
            index,
            index_writer: Mutex::new(index_writer),
            searcher,
        })
    }

    /// Add files to the knowledge base
    pub fn add_files(&mut self, paths: Vec<String>, recursive: bool) -> Result<(), Box<dyn Error>> {
        for path_str in paths {
            let path = Path::new(&path_str);

            if path.is_dir() {
                if recursive {
                    // Recursively add all files in directory
                    for entry in WalkDir::new(path) {
                        let entry = entry?;
                        if entry.file_type().is_file() {
                            self.add_file(entry.path())?;
                        }
                    }
                } else {
                    // Add only files in current directory
                    for entry in fs::read_dir(path)? {
                        let entry = entry?;
                        if entry.file_type()?.is_file() {
                            self.add_file(&entry.path())?
                        }
                    }
                }
            } else if path.is_file() {
                // Add single file
                self.add_file(path)?;
            }
        }

        // Commit changes to index
        self.index_writer.lock().unwrap().commit()?;

        Ok(())
    }

    /// Add a single file to the knowledge base
    fn add_file(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        // Determine file language based on extension
        let language = match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => "rust",
            Some("py") => "python",
            Some("js") => "javascript",
            Some("ts") => "typescript",
            Some("java") => "java",
            Some("c") => "c",
            Some("cpp") => "cpp",
            Some("go") => "go",
            Some("rb") => "ruby",
            Some("php") => "php",
            Some("html") => "html",
            Some("css") => "css",
            Some("json") => "json",
            Some("yaml") => "yaml",
            Some("yml") => "yaml",
            Some("toml") => "toml",
            Some("md") => "markdown",
            _ => "text",
        };

        // Read file content
        let content = fs::read_to_string(path)?;
        let path_str = path.to_str().ok_or("Invalid path")?;

        // Add to Tantivy index
        let mut document = Document::default();
        document.add_text(*CONTENT_FIELD, &content);
        document.add_text(*PATH_FIELD, path_str);
        document.add_text(*LANGUAGE_FIELD, language);
        document.add_i64(*TIMESTAMP_FIELD, chrono::Utc::now().timestamp());

        self.index_writer.lock().unwrap().add_document(document)?;

        // Add to Sled database for metadata
        self.db.insert(path_str.as_bytes(), content.as_bytes())?;

        Ok(())
    }

    /// Search the knowledge base
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error>> {
        // Create query parser
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![*CONTENT_FIELD, *PATH_FIELD, *LANGUAGE_FIELD],
        );

        // Parse query
        let query = query_parser.parse_query(query)?;

        // Search index
        let top_docs = self
            .searcher
            .search(&query, &(TopDocs::with_limit(limit), Count))?;

        // Extract results
        let mut results = Vec::new();
        for (score, doc_address) in top_docs.0 {
            let doc = self.searcher.doc(doc_address)?;
            let content = doc.get_first(*CONTENT_FIELD).unwrap().as_text().unwrap();
            let path = doc.get_first(*PATH_FIELD).unwrap().as_text().unwrap();
            let language = doc.get_first(*LANGUAGE_FIELD).unwrap().as_text().unwrap();

            results.push(format!(
                "Score: {:.2} | Path: {} | Language: {}\nContent: {:.100}...",
                score, path, language, content
            ));
        }

        Ok(results)
    }

    /// List knowledge base contents
    pub fn list(&self, details: bool) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results = Vec::new();

        // Iterate through Sled database
        for entry in self.db.iter() {
            let (key, value) = entry?;
            let path = String::from_utf8(key.to_vec())?;
            let content = String::from_utf8(value.to_vec())?;

            if details {
                results.push(format!(
                    "Path: {}\nSize: {} bytes\nLines: {}\n",
                    path,
                    content.len(),
                    content.lines().count()
                ));
            } else {
                results.push(path);
            }
        }

        Ok(results)
    }

    /// Clear the knowledge base
    pub fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        // Clear Tantivy index
        let mut writer = self.index_writer.lock().unwrap();
        writer.delete_all_documents()?;
        writer.commit()?;

        // Clear Sled database
        self.db.clear()?;

        Ok(())
    }

    /// Add scraped content to the knowledge base
    pub fn add_scraped_content(&mut self, url: &str, content: &str) -> Result<(), Box<dyn Error>> {
        // Add to Tantivy index
        let mut document = Document::default();
        document.add_text(*CONTENT_FIELD, content);
        document.add_text(*PATH_FIELD, url);
        document.add_text(*LANGUAGE_FIELD, "html");
        document.add_i64(*TIMESTAMP_FIELD, chrono::Utc::now().timestamp());

        self.index_writer.lock().unwrap().add_document(document)?;

        // Add to Sled database for metadata
        self.db.insert(url.as_bytes(), content.as_bytes())?;

        // Commit changes
        self.index_writer.lock().unwrap().commit()?;

        Ok(())
    }
}
