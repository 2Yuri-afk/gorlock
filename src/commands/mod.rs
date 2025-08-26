pub mod yt_dlp;

pub use yt_dlp::*;

use anyhow::Result;
use std::collections::HashMap;
use tokio::{sync::mpsc, task::JoinHandle};
use uuid::Uuid;

use crate::app_state::{
    DownloadItem,
    events::{AppEvent, DownloadAction},
};

/// Controller to manage download operations
pub struct DownloadController {
    /// Active download tasks
    tasks: HashMap<Uuid, JoinHandle<Result<()>>>,
    /// Channel to send app events
    app_tx: mpsc::UnboundedSender<AppEvent>,
}

impl DownloadController {
    pub fn new(app_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            tasks: HashMap::new(),
            app_tx,
        }
    }

    /// Handle a download action
    pub async fn handle_action(&mut self, action: DownloadAction) {
        match action {
            DownloadAction::AddUrl(url) => {
                self.add_url(url).await;
            }
            DownloadAction::StartDownload(id) => {
                self.start_download(id).await;
            }
            DownloadAction::PauseDownload(id) => {
                self.pause_download(id).await;
            }
            DownloadAction::ResumeDownload(id) => {
                self.resume_download(id).await;
            }
            DownloadAction::CancelDownload(id) => {
                self.cancel_download(id).await;
            }
            DownloadAction::RemoveItem(id) => {
                self.remove_item(id).await;
            }
            DownloadAction::FetchFormats(id) => {
                self.fetch_formats(id).await;
            }
        }
    }

    async fn add_url(&mut self, url: String) {
        // For now, we'll just trigger format fetching
        // In a real implementation, you'd first add to queue via AppEvent
        let item = DownloadItem::new(url.clone());
        let id = item.id;

        // Start format fetching in background
        let app_tx = self.app_tx.clone();
        let fetch_task = tokio::spawn(async move {
            match yt_dlp::fetch_formats(&url).await {
                Ok((formats, title, duration)) => {
                    let _ = app_tx.send(AppEvent::FormatsFetched {
                        id,
                        formats,
                        title,
                        duration,
                    });
                }
                Err(e) => {
                    let _ = app_tx.send(AppEvent::FormatsFetchFailed {
                        id,
                        error: format!("Failed to fetch formats: {}", e),
                    });
                }
            }
            Ok(())
        });

        // Store the task (though for format fetching we don't need to track it long-term)
        self.tasks.insert(id, fetch_task);
    }

    async fn start_download(&mut self, _id: Uuid) {
        // TODO: Implement actual download starting
        // This requires access to the item's format and output directory
        // For now, this is a placeholder
        println!("Start download for ID: {}", _id);
    }

    async fn pause_download(&mut self, id: Uuid) {
        // TODO: Implement pause functionality
        // This requires process management capabilities
        println!("Pause download for ID: {}", id);
    }

    async fn resume_download(&mut self, id: Uuid) {
        // TODO: Implement resume functionality
        println!("Resume download for ID: {}", id);
    }

    async fn cancel_download(&mut self, id: Uuid) {
        if let Some(handle) = self.tasks.remove(&id) {
            handle.abort();
        }

        let _ = self.app_tx.send(AppEvent::DownloadFailed {
            id,
            error: "Download cancelled by user".to_string(),
        });
    }

    async fn remove_item(&mut self, id: Uuid) {
        // Cancel any running task for this item
        if let Some(handle) = self.tasks.remove(&id) {
            handle.abort();
        }
    }

    async fn fetch_formats(&mut self, id: Uuid) {
        // This would need the URL from the item
        // For now, this is a placeholder
        println!("Fetch formats for ID: {}", id);
    }
}

/// Run the download controller task
pub async fn run_download_controller(
    mut action_rx: mpsc::UnboundedReceiver<DownloadAction>,
    app_tx: mpsc::UnboundedSender<AppEvent>,
) {
    let mut controller = DownloadController::new(app_tx);

    while let Some(action) = action_rx.recv().await {
        controller.handle_action(action).await;
    }
}
