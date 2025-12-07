//! Automatic update functionality for Codex
//! This module handles checking for updates, notifying users, and installing updates

use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use reqwest::Client;
use once_cell::sync::Lazy;

/// GitHub repository information for Codex
const GITHUB_OWNER: &str = "lyxamour";
const GITHUB_REPO: &str = "codex";
const GITHUB_API_URL: &str = "https://api.github.com";

/// Update check interval (24 hours)
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// HTTP client with timeout
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(format!("codex/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap()
});

/// Update status enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// No update available
    NoUpdate,
    /// Update is available
    UpdateAvailable { current_version: String, latest_version: String, download_url: String },
    /// Update check failed
    CheckFailed(String),
}

impl fmt::Display for UpdateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateStatus::NoUpdate => write!(f, "No update available"),
            UpdateStatus::UpdateAvailable { current_version, latest_version, .. } => {
                write!(f, "Update available: v{} -> v{}", current_version, latest_version)
            }
            UpdateStatus::CheckFailed(err) => write!(f, "Update check failed: {}", err),
        }
    }
}

/// GitHub release asset
#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    download_count: u32,
}

/// GitHub release information
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    prerelease: bool,
    draft: bool,
    created_at: String,
    published_at: String,
    assets: Vec<ReleaseAsset>,
}

/// Update manager struct
pub struct UpdateManager {
    current_version: String,
    last_check: Option<SystemTime>,
    update_check_url: String,
    enable_auto_check: bool,
}

impl UpdateManager {
    /// Create a new UpdateManager instance
    pub fn new(enable_auto_check: bool) -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            last_check: None,
            update_check_url: format!("{}/repos/{}/{}/releases/latest", GITHUB_API_URL, GITHUB_OWNER, GITHUB_REPO),
            enable_auto_check,
        }
    }

    /// Check if an update check is due
    pub fn is_check_due(&self) -> bool {
        if !self.enable_auto_check {
            return false;
        }

        match self.last_check {
            Some(last_check) => {
                let now = SystemTime::now();
                now.duration_since(last_check).unwrap_or(UPDATE_CHECK_INTERVAL) >= UPDATE_CHECK_INTERVAL
            }
            None => true,
        }
    }

    /// Check for updates
    pub async fn check_for_updates(&mut self) -> Result<UpdateStatus, Box<dyn Error>> {
        // Update last check time
        self.last_check = Some(SystemTime::now());

        // Get latest release from GitHub API
        let response = HTTP_CLIENT.get(&self.update_check_url).send().await?;
        
        if !response.status().is_success() {
            return Ok(UpdateStatus::CheckFailed(format!("HTTP error: {}", response.status())));
        }

        let release: GitHubRelease = response.json().await?;
        
        // Skip prerelease and draft releases
        if release.prerelease || release.draft {
            return Ok(UpdateStatus::NoUpdate);
        }

        // Parse version from tag name (remove 'v' prefix if present)
        let latest_version = release.tag_name.trim_start_matches('v').to_string();
        let current_version = self.current_version.clone();

        // Compare versions
        if latest_version > current_version {
            // Find the appropriate asset for the current platform
            let os = self.get_os();
            let arch = self.get_arch();
            
            let mut matched_asset = None;
            let mut os_only_asset = None;
            
            // Priority: exact match > same OS > any
            for asset in &release.assets {
                let name = &asset.name;
                if name.contains(os) && name.contains(arch) {
                    matched_asset = Some(asset);
                    break;
                } else if name.contains(os) {
                    os_only_asset.get_or_insert(asset);
                }
            }
            
            // Use first asset if no match found
            let asset = matched_asset.or(os_only_asset).or(release.assets.first()).ok_or("No appropriate asset found")?;
            
            Ok(UpdateStatus::UpdateAvailable {
                current_version,
                latest_version,
                download_url: asset.browser_download_url.clone(),
            })
        } else {
            Ok(UpdateStatus::NoUpdate)
        }
    }

    /// Get current operating system identifier
    fn get_os(&self) -> &str {
        #[cfg(target_os = "linux")]
        return "linux";
        
        #[cfg(target_os = "macos")]
        return "macos";
        
        #[cfg(target_os = "windows")]
        return "windows";
        
        #[cfg(target_os = "freebsd")]
        return "freebsd";
        
        #[cfg(target_os = "openbsd")]
        return "openbsd";
        
        #[cfg(target_os = "netbsd")]
        return "netbsd";
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", 
                      target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
        return "unknown";
    }

    /// Get current architecture
    fn get_arch(&self) -> &str {
        #[cfg(target_arch = "x86_64")]
        return "x86_64";
        
        #[cfg(target_arch = "aarch64")]
        return "aarch64";
        
        #[cfg(target_arch = "i686")]
        return "i686";
        
        #[cfg(target_arch = "arm")]
        return "arm";
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", 
                      target_arch = "i686", target_arch = "arm")))]
        return "unknown";
    }

    /// Download update
    pub async fn download_update(&self, download_url: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let response = HTTP_CLIENT.get(download_url).send().await?;
        let bytes = response.bytes().await?;
        
        tokio::fs::write(output_path, bytes).await?;
        Ok(())
    }

    /// Install update (platform-specific implementation needed)
    pub async fn install_update(&self, update_path: &str) -> Result<(), Box<dyn Error>> {
        // This is a placeholder - platform-specific implementation needed
        // For now, just return a success
        Ok(())
    }

    /// Show update notification
    pub fn show_update_notification(&self, status: &UpdateStatus) {
        match status {
            UpdateStatus::UpdateAvailable { current_version, latest_version, download_url } => {
                println!("\n🎉 New version available!\n");
                println!("Current version: v{}", current_version);
                println!("Latest version: v{}", latest_version);
                println!("\nTo update, run: codex update");
                println!("Or download manually from: {}", download_url);
                println!("\n");
            }
            UpdateStatus::NoUpdate => {
                // No need to notify if no update available
            }
            UpdateStatus::CheckFailed(err) => {
                // Silently fail for automatic checks, log for manual checks
                eprintln!("Update check failed: {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_manager_creation() {
        let manager = UpdateManager::new(true);
        assert_eq!(manager.current_version, env!("CARGO_PKG_VERSION"));
        assert!(manager.enable_auto_check);
    }

    #[tokio::test]
    async fn test_is_check_due() {
        // Test with no previous check
        let manager = UpdateManager::new(true);
        assert!(manager.is_check_due());
    }

    #[test]
    fn test_get_os() {
        let manager = UpdateManager::new(true);
        let os = manager.get_os();
        
        // Should return one of the known OS identifiers
        assert!(matches!(os, "linux" | "macos" | "windows" | "freebsd" | "openbsd" | "netbsd" | "unknown"));
    }

    #[test]
    fn test_get_arch() {
        let manager = UpdateManager::new(true);
        let arch = manager.get_arch();
        
        // Should return one of the known architecture identifiers
        assert!(matches!(arch, "x86_64" | "aarch64" | "i686" | "arm" | "unknown"));
    }
}
