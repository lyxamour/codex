use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use url::Url;

/// Web scraper configuration
#[derive(Debug, Clone)]
pub struct ScraperConfig {
    /// Maximum depth for recursive scraping
    max_depth: u32,
    /// User agent string
    user_agent: String,
    /// Whether to follow redirects
    follow_redirects: bool,
    /// Maximum concurrent requests
    max_concurrent: usize,
    /// URL filters (allowed domains)
    allowed_domains: Option<Vec<String>>,
    /// URL patterns to exclude
    exclude_patterns: Option<Vec<String>>,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            user_agent: "Codex-Scraper/1.0".to_string(),
            follow_redirects: true,
            max_concurrent: 10,
            allowed_domains: None,
            exclude_patterns: None,
        }
    }
}

/// Scraped content structure
#[derive(Debug, Clone)]
pub struct ScrapedContent {
    /// URL of the scraped page
    url: String,
    /// Title of the page
    title: String,
    /// Main content of the page
    content: String,
    /// Depth at which this page was scraped
    depth: u32,
}

/// Web scraper implementation
pub struct WebScraper {
    config: ScraperConfig,
    client: Arc<Client>,
    visited_urls: HashSet<String>,
}

impl WebScraper {
    /// Create a new web scraper with default configuration and specified depth
    pub fn new(max_depth: u32) -> Result<Self, Box<dyn Error>> {
        let config = ScraperConfig {
            max_depth,
            ..Default::default()
        };

        let client = Arc::new(
            Client::builder()
                .user_agent(&config.user_agent)
                .redirect(if config.follow_redirects {
                    reqwest::redirect::Policy::default()
                } else {
                    reqwest::redirect::Policy::none()
                })
                .build()?,
        );

        Ok(Self {
            config,
            client,
            visited_urls: HashSet::new(),
        })
    }

    /// Create a new web scraper with custom configuration
    pub fn with_config(config: ScraperConfig) -> Result<Self, Box<dyn Error>> {
        let client = Arc::new(
            Client::builder()
                .user_agent(&config.user_agent)
                .redirect(if config.follow_redirects {
                    reqwest::redirect::Policy::default()
                } else {
                    reqwest::redirect::Policy::none()
                })
                .build()?,
        );

        Ok(Self {
            config,
            client,
            visited_urls: HashSet::new(),
        })
    }

    /// Scrape multiple URLs
    pub async fn scrape(&mut self, urls: &[String]) -> Result<Vec<ScrapedContent>, Box<dyn Error>> {
        let mut results = Vec::new();

        for url in urls {
            let scraped = self.scrape_single(url, 0).await?;
            results.extend(scraped);
        }

        Ok(results)
    }

    /// Scrape a single URL recursively
    fn scrape_single<'a>(
        &'a mut self,
        url: &str,
        depth: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ScrapedContent>, Box<dyn Error>>> + 'a>> {
        let url = url.to_string();
        Box::pin(async move {
            let mut results = Vec::new();

            // Check if we've already visited this URL
            if self.visited_urls.contains(&url) {
                return Ok(results);
            }

            // Check if we've reached maximum depth
            if depth > self.config.max_depth {
                return Ok(results);
            }

            // Parse URL
            let parsed_url = Url::parse(&url)?;

            // Check domain restrictions
            if let Some(allowed_domains) = &self.config.allowed_domains {
                if let Some(domain) = parsed_url.domain() {
                    if !allowed_domains.contains(&domain.to_string()) {
                        return Ok(results);
                    }
                }
            }

            // Add to visited URLs
            self.visited_urls.insert(url.clone());

            println!("Scraping: {} (depth: {})\n", url, depth);

            // Fetch page content
            let response = self.client.get(&url).send().await?;

            if !response.status().is_success() {
                println!("Failed to scrape {}: {}\n", url, response.status());
                return Ok(results);
            }

            // Get content type
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("");

            // Only process HTML content
            if !content_type.starts_with("text/html") {
                return Ok(results);
            }

            // Read response body
            let body = response.text().await?;

            // Parse HTML
            let document = Html::parse_document(&body);

            // Extract title
            let title_selector = Selector::parse("title").unwrap();
            let title = document
                .select(&title_selector)
                .next()
                .and_then(|element| element.text().next())
                .unwrap_or("")
                .trim()
                .to_string();

            // Extract main content (simplified)
            let content = extract_main_content(&document);

            // Create scraped content
            results.push(ScrapedContent {
                url: url.clone(),
                title,
                content,
                depth,
            });

            // If we're not at maximum depth, find and scrape links
            if depth < self.config.max_depth {
                let links = extract_links(&document, &parsed_url);

                for link in links {
                    // Recursively scrape links
                    let link_results = self.scrape_single(&link, depth + 1).await?;
                    results.extend(link_results);
                }
            }

            Ok(results)
        })
    }

    /// Clear visited URLs cache
    pub fn clear_cache(&mut self) {
        self.visited_urls.clear();
    }

    /// Set allowed domains for scraping
    pub fn set_allowed_domains(&mut self, domains: Vec<String>) {
        self.config.allowed_domains = Some(domains);
    }

    /// Set exclude patterns
    pub fn set_exclude_patterns(&mut self, patterns: Vec<String>) {
        self.config.exclude_patterns = Some(patterns);
    }
}

/// Extract main content from HTML document (simplified approach)
fn extract_main_content(document: &Html) -> String {
    // Try different selectors for main content
    let content_selectors = [
        Selector::parse("main").unwrap(),
        Selector::parse(".main-content").unwrap(),
        Selector::parse(".content").unwrap(),
        Selector::parse("article").unwrap(),
        Selector::parse(".article").unwrap(),
    ];

    for selector in content_selectors.iter() {
        if let Some(content_element) = document.select(selector).next() {
            return content_element
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
        }
    }

    // Fallback: extract all text from body
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = document.select(&body_selector).next() {
        return body.text().collect::<Vec<_>>().join(" ").trim().to_string();
    }

    "".to_string()
}

/// Extract links from HTML document
fn extract_links(document: &Html, base_url: &Url) -> Vec<String> {
    let mut links = Vec::new();
    let link_selector = Selector::parse("a").unwrap();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Parse and resolve URL
            match base_url.join(href) {
                Ok(resolved_url) => {
                    // Only keep HTTP/HTTPS links
                    if resolved_url.scheme() == "http" || resolved_url.scheme() == "https" {
                        // Remove fragment
                        let mut resolved_url = resolved_url;
                        resolved_url.set_fragment(None);
                        links.push(resolved_url.to_string());
                    }
                }
                Err(_) => {
                    // Skip invalid URLs
                    continue;
                }
            }
        }
    }

    // Remove duplicates
    let mut unique_links = HashSet::new();
    unique_links.extend(links);
    unique_links.into_iter().collect()
}

/// Filter URLs based on exclude patterns
fn filter_urls(urls: Vec<String>, patterns: &[String]) -> Vec<String> {
    urls.into_iter()
        .filter(|url| !patterns.iter().any(|pattern| url.contains(pattern)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scrape_single() {
        let mut scraper = WebScraper::new(1).unwrap();
        let results = scraper.scrape_single("https://example.com", 0).await;
        assert!(results.is_ok());
        assert!(!results.unwrap().is_empty());
    }

    #[test]
    fn test_extract_links() {
        let html = r#"<html><body><a href="/test">Test</a><a href="https://example.com/external">External</a></body></html>"#;
        let document = Html::parse_document(html);
        let base_url = Url::parse("https://example.com").unwrap();
        let links = extract_links(&document, &base_url);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com/test".to_string()));
        assert!(links.contains(&"https://example.com/external".to_string()));
    }

    #[test]
    fn test_filter_urls() {
        let urls = vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
            "https://example.com/admin".to_string(),
        ];
        let patterns = vec!["admin".to_string()];
        let filtered = filter_urls(urls, &patterns);
        assert_eq!(filtered.len(), 2);
        assert!(!filtered.contains(&"https://example.com/admin".to_string()));
    }
}
