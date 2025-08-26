use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::app_state::{AppState, Panel, DownloadStatus, events::*};

/// Handle input events and update application state
pub async fn handle_input(
    event: InputEvent,
    state: &mut AppState,
    action_tx: &mpsc::Sender<DownloadAction>,
) {
    match event {
        InputEvent::Key(key) => {
            handle_key_event(key, state, action_tx).await;
        }
        InputEvent::Mouse(_) => {
            // Mouse handling can be implemented later if needed
        }
        InputEvent::Resize(_width, _height) => {
            // Terminal resize is handled automatically by ratatui
        }
    }
}

/// Handle keyboard events
async fn handle_key_event(
    key: KeyEvent,
    state: &mut AppState,
    action_tx: &mpsc::Sender<DownloadAction>,
) {
    // Handle Ctrl-C to quit
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        state.should_quit = true;
        return;
    }

    // Clear any error message on key press
    if state.error_message.is_some() {
        state.error_message = None;
        return;
    }

    // Handle format popup if active
    if state.format_popup.is_some() {
        handle_format_popup_input(key, state, action_tx).await;
        return;
    }

    // Handle playlist preview popup if active
    if state.playlist_preview.is_some() {
        handle_playlist_preview_input(key, state, action_tx).await;
        return;
    }

    // Handle input mode
    if state.input_mode {
        handle_input_mode(key, state, action_tx).await;
        return;
    }

    // Handle normal navigation mode
    handle_navigation_mode(key, state, action_tx).await;
}

/// Handle input when in URL input mode
async fn handle_input_mode(
    key: KeyEvent,
    state: &mut AppState,
    action_tx: &mpsc::Sender<DownloadAction>,
) {
    match key.code {
        KeyCode::Enter => {
            if !state.url_input.trim().is_empty() {
                let url = state.url_input.trim().to_string();
                // Set loading state before processing URL
                state.is_loading = true;
                state.loading_message = Some("Fetching video information...".to_string());
                let _ = action_tx.send(DownloadAction::AddUrl(url)).await;
                state.url_input.clear();
            }
            state.input_mode = false;
            state.current_panel = Panel::Queue;
        }
        KeyCode::Esc => {
            state.input_mode = false;
            state.current_panel = Panel::Queue;
        }
        KeyCode::Char(c) => {
            state.url_input.push(c);
        }
        KeyCode::Backspace => {
            state.url_input.pop();
        }
        KeyCode::Delete => {
            state.url_input.clear();
        }
        _ => {}
    }
}

/// Handle input in normal navigation mode
async fn handle_navigation_mode(
    key: KeyEvent,
    state: &mut AppState,
    action_tx: &mpsc::Sender<DownloadAction>,
) {
    match key.code {
        KeyCode::Char('q') => {
            state.should_quit = true;
        }
        KeyCode::Char('i') => {
            state.input_mode = true;
            state.current_panel = Panel::Input;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !state.queue.is_empty() && state.selected_index > 0 {
                state.selected_index -= 1;
                // Prefetch formats for the newly selected item if not already fetched
                prefetch_formats_for_selected_item(state, action_tx).await;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.queue.is_empty() && state.selected_index < state.queue.len() - 1 {
                state.selected_index += 1;
                // Prefetch formats for the newly selected item if not already fetched
                prefetch_formats_for_selected_item(state, action_tx).await;
            }
        }
        KeyCode::Tab => {
            state.current_panel = match state.current_panel {
                Panel::Queue => Panel::Details,
                Panel::Details => Panel::Input,
                Panel::Input => Panel::Queue,
            };
        }
        KeyCode::Char('f') => {
            if let Some(item) = state.queue.get(state.selected_index) {
                if matches!(
                    item.status,
                    crate::app_state::DownloadStatus::Pending
                        | crate::app_state::DownloadStatus::Ready
                        | crate::app_state::DownloadStatus::Failed
                ) {
                    let _ = action_tx.send(DownloadAction::FetchFormats(item.id)).await;
                }
            }
        }
        KeyCode::Char('d') => {
            if !state.queue.is_empty() {
                let item = &state.queue[state.selected_index];
                let _ = action_tx.send(DownloadAction::RemoveItem(item.id)).await;

                // Remove from queue immediately for UI responsiveness
                state.queue.remove(state.selected_index);
                if state.selected_index >= state.queue.len() && !state.queue.is_empty() {
                    state.selected_index = state.queue.len() - 1;
                }
            }
        }
        KeyCode::Char('p') => {
            if let Some(item) = state.queue.get(state.selected_index) {
                match item.status {
                    crate::app_state::DownloadStatus::Downloading => {
                        let _ = action_tx.send(DownloadAction::PauseDownload(item.id)).await;
                    }
                    crate::app_state::DownloadStatus::Paused => {
                        let _ = action_tx.send(DownloadAction::ResumeDownload(item.id)).await;
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Char('c') => {
            if let Some(item) = state.queue.get(state.selected_index) {
                if matches!(
                    item.status,
                    crate::app_state::DownloadStatus::Downloading
                        | crate::app_state::DownloadStatus::Paused
                ) {
                    let _ = action_tx.send(DownloadAction::CancelDownload(item.id)).await;
                }
            }
        }
        _ => {}
    }
}

/// Handle input when format popup is active
async fn handle_format_popup_input(
    key: KeyEvent,
    state: &mut AppState,
    action_tx: &mpsc::Sender<DownloadAction>,
) {
    if let Some(popup) = &mut state.format_popup {
        // Get filtered formats for navigation
        let filtered_formats: Vec<&crate::app_state::FormatInfo> = popup
            .formats
            .iter()
            .filter(|format| {
                if popup.audio_only_filter {
                    format.is_audio_only
                } else {
                    true // Show all formats when filter is off
                }
            })
            .collect();
            
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if popup.selected_index > 0 {
                    popup.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if popup.selected_index < filtered_formats.len().saturating_sub(1) {
                    popup.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(selected_format) = filtered_formats.get(popup.selected_index).cloned() {
                    let item_id = popup.item_id;
                    let selected_format = selected_format.clone();
                    
                    // Close popup first
                    state.format_popup = None;
                    
                    // Update the item with selected format
                    if let Some(item) = state.queue.iter_mut().find(|item| item.id == item_id) {
                        item.format = Some(selected_format);
                        item.status = crate::app_state::DownloadStatus::Ready;
                    }
                    
                    // Start download
                    let _ = action_tx.send(DownloadAction::StartDownload(item_id)).await;
                }
            }
            KeyCode::Char('t') => {
                // Toggle audio-only filter
                popup.audio_only_filter = !popup.audio_only_filter;
                popup.selected_index = 0; // Reset selection when filtering
            }
            KeyCode::Esc => {
                state.format_popup = None;
            }
            _ => {}
        }
    }
}

/// Handle input when playlist preview popup is active
async fn handle_playlist_preview_input(
    key: KeyEvent,
    state: &mut AppState,
    _action_tx: &mpsc::Sender<DownloadAction>,
) {
    if let Some(preview) = &mut state.playlist_preview {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if preview.selected_index > 0 {
                    preview.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if preview.selected_index + 1 < preview.entries.len() {
                    preview.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                // Confirm: add all entries to queue
                let entries = std::mem::take(&mut preview.entries);
                state.playlist_preview = None;

                for e in entries {
                    let mut item = crate::app_state::DownloadItem::new(e.url);
                    item.title = Some(e.title);
                    item.duration = e.duration;
                    item.status = crate::app_state::DownloadStatus::Pending;
                    state.queue.push(item);
                }
            }
            KeyCode::Esc => {
                // Cancel
                state.playlist_preview = None;
            }
            _ => {}
        }
    }
}

/// Prefetch formats for the selected item if needed
async fn prefetch_formats_for_selected_item(
    state: &AppState,
    action_tx: &mpsc::Sender<DownloadAction>,
) {
    if let Some(item) = state.queue.get(state.selected_index) {
        // Only prefetch if the item hasn't been fetched yet
        if matches!(item.status, DownloadStatus::Pending) {
            // Send a fetch formats action in background (non-blocking)
            let _ = action_tx.try_send(DownloadAction::FetchFormats(item.id));
        }
    }
}
