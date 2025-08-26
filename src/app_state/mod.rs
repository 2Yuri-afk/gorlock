use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub mod events;

/// The main application state
#[derive(Debug)]
pub struct AppState {
    /// Download queue
    pub queue: Vec<DownloadItem>,
    /// Currently selected item in queue
    pub selected_index: usize,
    /// Current active panel
    pub current_panel: Panel,
    /// Output directory for downloads
    pub output_dir: String,
    /// Current input buffer for URL entry
    pub url_input: String,
    /// Whether we're in input mode
    pub input_mode: bool,
    /// Error messages to display
    pub error_message: Option<String>,
    /// Running download tasks
    pub running_tasks: HashMap<Uuid, JoinHandle<Result<()>>>,
    /// Application should exit
    pub should_quit: bool,
    /// Format selection popup state
    pub format_popup: Option<FormatPopup>,
    /// Loading indicator for URL processing
    pub is_loading: bool,
    /// Loading message to display
    pub loading_message: Option<String>,
    /// Playlist preview popup state
    pub playlist_preview: Option<PlaylistPreviewPopup>,
}

/// Different panels in the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Queue,
    Details,
    Input,
}

/// Download item in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub duration: Option<String>,
    pub format: Option<FormatInfo>,
    pub status: DownloadStatus,
    pub progress: DownloadProgress,
    pub created_at: DateTime<Utc>,
    pub error: Option<String>,
}

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    FetchingInfo,
    Ready,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub percent: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub downloaded: Option<String>,
    pub total_size: Option<String>,
}

/// Format information from yt-dlp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub fps: Option<f64>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub quality: Option<String>,
    pub is_audio_only: bool,
}

/// Format selection popup state
#[derive(Debug, Clone)]
pub struct FormatPopup {
    pub item_id: Uuid,
    pub formats: Vec<FormatInfo>,
    pub selected_index: usize,
    pub audio_only_filter: bool,
}

/// Playlist entry for preview popup
#[derive(Debug, Clone)]
pub struct PlaylistEntry {
    pub url: String,
    pub title: String,
    pub duration: Option<String>,
}

/// Playlist preview popup state
#[derive(Debug, Clone)]
pub struct PlaylistPreviewPopup {
    pub entries: Vec<PlaylistEntry>,
    pub selected_index: usize,
    pub total_duration: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            selected_index: 0,
            current_panel: Panel::Input,
            output_dir: dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap().join("Downloads"))
                .to_string_lossy()
                .to_string(),
            url_input: String::new(),
            input_mode: false,
            error_message: None,
            running_tasks: HashMap::new(),
            should_quit: false,
            format_popup: None,
            is_loading: false,
            loading_message: None,
            playlist_preview: None,
        }
    }
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            percent: 0.0,
            speed: None,
            eta: None,
            downloaded: None,
            total_size: None,
        }
    }
}

impl DownloadItem {
    pub fn new(url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            title: None,
            duration: None,
            format: None,
            status: DownloadStatus::Pending,
            progress: DownloadProgress::default(),
            created_at: Utc::now(),
            error: None,
        }
    }
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadStatus::Pending => write!(f, "Pending"),
            DownloadStatus::FetchingInfo => write!(f, "Fetching info..."),
            DownloadStatus::Ready => write!(f, "Ready"),
            DownloadStatus::Downloading => write!(f, "Downloading"),
            DownloadStatus::Paused => write!(f, "Paused"),
            DownloadStatus::Completed => write!(f, "Completed"),
            DownloadStatus::Failed => write!(f, "Failed"),
            DownloadStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl FormatInfo {
    pub fn display_name(&self) -> String {
        let mut parts = vec![];
        
        if self.is_audio_only {
            parts.push("Audio Only".to_string());
        } else {
            if let Some(resolution) = &self.resolution {
                parts.push(format!("Video {}", resolution));
            } else {
                parts.push("Video".to_string());
            }
        }
        
        if let Some(fps) = self.fps {
            parts.push(format!("{}fps", fps));
        }
        
        parts.push(self.ext.clone());
        
        if let Some(size) = self.filesize {
            parts.push(format_bytes(size));
        }
        
        // Add note about audio merging for video formats
        if !self.is_audio_only {
            parts.push("(+audio)".to_string());
        }
        
        parts.join(" • ")
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_index])
}

/// Parse duration string (e.g., "03:45" or "1:23:45") to seconds
pub fn parse_duration_to_seconds(duration: &str) -> Option<u64> {
    let parts: Vec<&str> = duration.split(':').collect();
    match parts.len() {
        1 => {
            // Just seconds
            parts[0].parse::<u64>().ok()
        }
        2 => {
            // MM:SS
            let minutes = parts[0].parse::<u64>().ok()?;
            let seconds = parts[1].parse::<u64>().ok()?;
            Some(minutes * 60 + seconds)
        }
        3 => {
            // HH:MM:SS
            let hours = parts[0].parse::<u64>().ok()?;
            let minutes = parts[1].parse::<u64>().ok()?;
            let seconds = parts[2].parse::<u64>().ok()?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        _ => None,
    }
}

/// Format seconds to duration string
pub fn format_duration_from_seconds(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
