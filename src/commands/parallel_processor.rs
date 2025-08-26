use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::app_state::{events::AppEvent, FormatInfo};
use super::yt_dlp;

/// Process playlist entries in parallel with controlled concurrency
pub struct ParallelPlaylistProcessor {
    /// Maximum concurrent format fetches
    max_concurrency: usize,
    /// Semaphore to control concurrency
    semaphore: Arc<Semaphore>,
}

impl ParallelPlaylistProcessor {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    /// Process multiple playlist entries concurrently
    pub async fn process_playlist_formats(
        &self,
        entries: Vec<(Uuid, String)>, // (item_id, url)
        app_tx: mpsc::Sender<AppEvent>,
    ) -> Result<()> {
        let mut tasks = JoinSet::new();
        
        for (id, url) in entries {
            let semaphore = self.semaphore.clone();
            let app_tx_clone = app_tx.clone();
            
            tasks.spawn(async move {
                // Acquire permit for concurrency control
                let _permit = semaphore.acquire().await.unwrap();
                
                // Fetch formats for this video
                match yt_dlp::fetch_formats(&url).await {
                    Ok((formats, title, duration)) => {
                        let _ = app_tx_clone.send(AppEvent::FormatsFetched {
                            id,
                            formats,
                            title,
                            duration,
                        }).await;
                    }
                    Err(e) => {
                        let _ = app_tx_clone.send(AppEvent::FormatsFetchFailed {
                            id,
                            error: format!("Failed to fetch formats: {}", e),
                        }).await;
                    }
                }
            });
        }
        
        // Wait for all tasks to complete
        while let Some(_) = tasks.join_next().await {}
        
        Ok(())
    }
    
    /// Prefetch formats for a single item (for hover prefetching)
    pub async fn prefetch_single(
        &self,
        id: Uuid,
        url: String,
        app_tx: mpsc::Sender<AppEvent>,
    ) {
        let semaphore = self.semaphore.clone();
        
        tokio::spawn(async move {
            // Try to acquire permit, but don't wait if busy
            if let Ok(_permit) = semaphore.try_acquire() {
                // Silently fetch formats in background
                if let Ok((formats, title, duration)) = yt_dlp::fetch_formats(&url).await {
                    let _ = app_tx.send(AppEvent::FormatsFetched {
                        id,
                        formats,
                        title,
                        duration,
                    }).await;
                }
            }
        });
    }
}

/// Global instance for parallel processing
lazy_static::lazy_static! {
    pub static ref PARALLEL_PROCESSOR: ParallelPlaylistProcessor = 
        ParallelPlaylistProcessor::new(8); // Max 8 concurrent fetches
}
