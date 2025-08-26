use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::app_state::FormatInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    pub url: String,
    pub title: String,
    pub duration: Option<String>,
    pub formats: Option<Vec<FormatInfo>>,
    pub playlist_entries: Option<Vec<(String, String, Option<String>)>>,
    pub timestamp: u64,
}

pub struct CacheStore {
    entries: Arc<RwLock<HashMap<String, CachedEntry>>>,
    cache_file: PathBuf,
    ttl: Duration,
}

impl CacheStore {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
            .join("gorlock");
        
        std::fs::create_dir_all(&cache_dir)?;
        let cache_file = cache_dir.join("metadata_cache.json");
        
        // Load existing cache if available
        let entries = if cache_file.exists() {
            let contents = std::fs::read_to_string(&cache_file)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            entries: Arc::new(RwLock::new(entries)),
            cache_file,
            ttl: Duration::from_secs(24 * 3600), // 24 hour TTL
        })
    }
    
    pub async fn get(&self, url: &str) -> Option<CachedEntry> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(url) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Check if entry is still valid
            if now - entry.timestamp < self.ttl.as_secs() {
                return Some(entry.clone());
            }
        }
        None
    }
    
    pub async fn set(&self, url: String, entry: CachedEntry) -> Result<()> {
        {
            let mut entries = self.entries.write().await;
            entries.insert(url, entry);
        }
        
        // Save to disk asynchronously
        let entries = self.entries.clone();
        let cache_file = self.cache_file.clone();
        
        tokio::task::spawn_blocking(move || {
            let entries = entries.blocking_read();
            if let Ok(json) = serde_json::to_string_pretty(&*entries) {
                let _ = std::fs::write(cache_file, json);
            }
        });
        
        Ok(())
    }
    
    pub async fn invalidate(&self, url: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(url);
        Ok(())
    }
    
    pub async fn clear(&self) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.clear();
        let _ = std::fs::remove_file(&self.cache_file);
        Ok(())
    }
}

// Helper to create a cached entry with current timestamp
impl CachedEntry {
    pub fn new(url: String, title: String, duration: Option<String>) -> Self {
        Self {
            url,
            title,
            duration,
            formats: None,
            playlist_entries: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

// Global cache instance
lazy_static::lazy_static! {
    pub static ref CACHE: tokio::sync::OnceCell<CacheStore> = tokio::sync::OnceCell::new();
}

pub async fn get_cache() -> &'static CacheStore {
    CACHE.get_or_init(|| async {
        CacheStore::new().expect("Failed to initialize cache")
    }).await
}
