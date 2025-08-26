use crate::app_state::{DownloadProgress, FormatInfo};
use uuid::Uuid;

/// Events that can be sent to the main application
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Quit the application
    Quit,
    /// Update download progress
    ProgressUpdate {
        id: Uuid,
        progress: DownloadProgress,
    },
    /// Download completed successfully
    DownloadCompleted { id: Uuid },
    /// Download failed
    DownloadFailed { id: Uuid, error: String },
    /// Format information fetched for a URL
    FormatsFetched {
        id: Uuid,
        formats: Vec<FormatInfo>,
        title: String,
        duration: Option<String>,
    },
    /// Failed to fetch formats
    FormatsFetchFailed { id: Uuid, error: String },
    /// URL validation completed
    UrlValidated {
        url: String,
        is_valid: bool,
        error: Option<String>,
    },
    /// Playlist detected with multiple entries
    PlaylistDetected {
        entries: Vec<(String, String, Option<String>)>, // (url, title, duration)
    },
    /// Single video detected (from playlist check)
    SingleVideoDetected {
        url: String,
        title: String,
        duration: Option<String>,
    },
    /// Failed to fetch playlist information
    PlaylistFetchFailed {
        error: String,
    },
}

/// Input events from the terminal
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Key was pressed
    Key(crossterm::event::KeyEvent),
    /// Mouse event
    Mouse(crossterm::event::MouseEvent),
    /// Terminal was resized
    Resize(u16, u16),
}

/// Actions that can be performed on downloads
#[derive(Debug, Clone)]
pub enum DownloadAction {
    /// Add a new URL to the queue
    AddUrl(String),
    /// Start downloading an item
    StartDownload(Uuid),
    /// Pause a download
    PauseDownload(Uuid),
    /// Resume a paused download
    ResumeDownload(Uuid),
    /// Cancel a download
    CancelDownload(Uuid),
    /// Remove an item from the queue
    RemoveItem(Uuid),
    /// Fetch available formats for a URL
    FetchFormats(Uuid),
}
