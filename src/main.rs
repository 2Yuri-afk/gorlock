use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, time};

mod app_state;
mod commands;
mod ui;

use app_state::{AppState, events::*};
use ui::{App, handle_input};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup application state
    let mut app_state = AppState::default();
    let mut app = App::default();

    // Create communication channels
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<InputEvent>();
    let (app_tx, mut app_rx) = mpsc::unbounded_channel::<AppEvent>();
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<DownloadAction>();

    // Spawn input handling task
    let input_task = {
        let input_tx = input_tx.clone();
        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            let tick_rate = Duration::from_millis(250);

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if crossterm::event::poll(timeout).unwrap_or(false) {
                    match event::read().unwrap() {
                        Event::Key(key) => {
                            if input_tx.send(InputEvent::Key(key)).is_err() {
                                break;
                            }
                        }
                        Event::Mouse(mouse) => {
                            if input_tx.send(InputEvent::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Event::Resize(w, h) => {
                            if input_tx.send(InputEvent::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    last_tick = Instant::now();
                }
            }
        })
    };

    // Download controller is now handled directly in the main event loop

    // Main event loop
    let mut last_render = Instant::now();
    let render_rate = Duration::from_millis(16); // ~60 FPS

    let result = loop {
        tokio::select! {
            // Handle input events
            input_event = input_rx.recv() => {
                if let Some(event) = input_event {
                    handle_input(event, &mut app_state, &action_tx).await;
                    if app_state.should_quit {
                        break Ok(());
                    }
                }
            }

            // Handle download actions
            action = action_rx.recv() => {
                if let Some(action) = action {
                    handle_download_action(action, &mut app_state, &app_tx).await;
                }
            }

            // Handle application events
            app_event = app_rx.recv() => {
                if let Some(event) = app_event {
                    handle_app_event(event, &mut app_state).await;
                    if app_state.should_quit {
                        break Ok(());
                    }
                }
            }

            // Render UI at ~60 FPS
            _ = time::sleep_until(time::Instant::from_std(last_render + render_rate)) => {
                terminal.draw(|f| app.render(f, &app_state))?;
                last_render = Instant::now();
            }
        }
    };

    // Cleanup
    input_task.abort();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Handle download actions in the main event loop
async fn handle_download_action(
    action: DownloadAction,
    state: &mut AppState,
    app_tx: &mpsc::UnboundedSender<AppEvent>,
) {
    match action {
        DownloadAction::AddUrl(url) => {
            // First check if this might be a playlist by trying to get entries
            let app_tx_clone = app_tx.clone();
            let url_clone = url.clone();
            
            tokio::spawn(async move {
                match commands::yt_dlp::fetch_playlist_entries(&url_clone).await {
                    Ok(entries) => {
                        if entries.len() > 1 {
                            // It's a playlist with multiple entries - queue them all
                            let _ = app_tx_clone.send(AppEvent::PlaylistDetected {
                                entries,
                            });
                        } else if let Some((entry_url, title, duration)) = entries.first() {
                            // Single entry - treat as regular video
                            let _ = app_tx_clone.send(AppEvent::SingleVideoDetected {
                                url: entry_url.clone(),
                                title: title.clone(),
                                duration: duration.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        let _ = app_tx_clone.send(AppEvent::PlaylistFetchFailed {
                            error: format!("Failed to process URL: {}", e),
                        });
                    }
                }
            });
        }
        DownloadAction::StartDownload(id) => {
            if let Some(item) = state.queue.iter_mut().find(|i| i.id == id) {
                if let Some(format) = &item.format {
                    let format_id = format.format_id.clone();
                    let url = item.url.clone();
                    let output_dir = state.output_dir.clone();
                    let app_tx_clone = app_tx.clone();

                    item.status = app_state::DownloadStatus::Downloading;

                    // Start download in background
                    let download_task = tokio::spawn(async move {
                        let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();

                        // Spawn progress forwarding task
                        let progress_forward_task = {
                            let app_tx = app_tx_clone.clone();
                            tokio::spawn(async move {
                                while let Some(progress) = progress_rx.recv().await {
                                    let _ = app_tx.send(AppEvent::ProgressUpdate { id, progress });
                                }
                            })
                        };

                        // Start actual download
                        match commands::yt_dlp::start_download(
                            &url,
                            &format_id,
                            &output_dir,
                            progress_tx,
                        )
                        .await
                        {
                            Ok(()) => {
                                let _ = app_tx_clone.send(AppEvent::DownloadCompleted { id });
                            }
                            Err(e) => {
                                let _ = app_tx_clone.send(AppEvent::DownloadFailed {
                                    id,
                                    error: format!("Download failed: {}", e),
                                });
                            }
                        }

                        progress_forward_task.abort();
                        Ok(())
                    });

                    // Store the task handle for potential cancellation
                    state.running_tasks.insert(id, download_task);
                }
            }
        }
        DownloadAction::CancelDownload(id) => {
            if let Some(handle) = state.running_tasks.remove(&id) {
                handle.abort();
            }

            if let Some(item) = state.queue.iter_mut().find(|i| i.id == id) {
                item.status = app_state::DownloadStatus::Cancelled;
            }
        }
        DownloadAction::RemoveItem(id) => {
            // Cancel any running task
            if let Some(handle) = state.running_tasks.remove(&id) {
                handle.abort();
            }

            // Remove from queue - this is already handled in the input handler
            // for immediate UI responsiveness
        }
        DownloadAction::FetchFormats(id) => {
            if let Some(item) = state.queue.iter_mut().find(|i| i.id == id) {
                let url = item.url.clone();
                item.status = app_state::DownloadStatus::FetchingInfo;

                let app_tx_clone = app_tx.clone();
                tokio::spawn(async move {
                    match commands::yt_dlp::fetch_formats(&url).await {
                        Ok((formats, title, duration)) => {
                            let _ = app_tx_clone.send(AppEvent::FormatsFetched {
                                id,
                                formats,
                                title,
                                duration,
                            });
                        }
                        Err(e) => {
                            let _ = app_tx_clone.send(AppEvent::FormatsFetchFailed {
                                id,
                                error: format!("Failed to fetch formats: {}", e),
                            });
                        }
                    }
                });
            }
        }
        // TODO: Implement pause/resume functionality
        DownloadAction::PauseDownload(_id) => {
            // Placeholder - requires process management
        }
        DownloadAction::ResumeDownload(_id) => {
            // Placeholder - requires process management
        }
    }
}

/// Handle application events from background tasks
async fn handle_app_event(event: AppEvent, state: &mut AppState) {
    match event {
        AppEvent::Quit => {
            state.should_quit = true;
        }
        AppEvent::ProgressUpdate { id, progress } => {
            if let Some(item) = state.queue.iter_mut().find(|item| item.id == id) {
                item.progress = progress;
                if item.status != app_state::DownloadStatus::Downloading {
                    item.status = app_state::DownloadStatus::Downloading;
                }
            }
        }
        AppEvent::DownloadCompleted { id } => {
            if let Some(item) = state.queue.iter_mut().find(|item| item.id == id) {
                item.status = app_state::DownloadStatus::Completed;
            }
            state.running_tasks.remove(&id);
        }
        AppEvent::DownloadFailed { id, error } => {
            if let Some(item) = state.queue.iter_mut().find(|item| item.id == id) {
                item.status = app_state::DownloadStatus::Failed;
                item.error = Some(error);
            }
            state.running_tasks.remove(&id);
        }
        AppEvent::FormatsFetched {
            id,
            formats,
            title,
            duration,
        } => {
            if let Some(item) = state.queue.iter_mut().find(|item| item.id == id) {
                item.title = Some(title);
                item.duration = duration;
                item.status = app_state::DownloadStatus::Ready;

                // Show format selection popup
                state.format_popup = Some(app_state::FormatPopup {
                    item_id: id,
                    formats,
                    selected_index: 0,
                    audio_only_filter: false,
                });
            }
        }
        AppEvent::FormatsFetchFailed { id, error } => {
            if let Some(item) = state.queue.iter_mut().find(|item| item.id == id) {
                item.status = app_state::DownloadStatus::Failed;
                item.error = Some(error.clone());
            }
            state.error_message = Some(error);
        }
        AppEvent::UrlValidated {
            url: _,
            is_valid: _,
            error,
        } => {
            if let Some(error) = error {
                state.error_message = Some(error);
            }
        }
        AppEvent::PlaylistDetected { entries } => {
            // Clear loading state
            state.is_loading = false;
            state.loading_message = None;
            
            // Calculate total duration
            let mut total_seconds = 0u64;
            let playlist_entries: Vec<app_state::PlaylistEntry> = entries
                .into_iter()
                .map(|(url, title, duration)| {
                    // Add to total duration if parseable
                    if let Some(dur) = &duration {
                        if let Some(seconds) = app_state::parse_duration_to_seconds(dur) {
                            total_seconds += seconds;
                        }
                    }
                    app_state::PlaylistEntry {
                        url,
                        title,
                        duration,
                    }
                })
                .collect();
            
            // Format total duration
            let total_duration = if total_seconds > 0 {
                Some(app_state::format_duration_from_seconds(total_seconds))
            } else {
                None
            };
            
            // Show playlist preview popup
            state.playlist_preview = Some(app_state::PlaylistPreviewPopup {
                entries: playlist_entries,
                selected_index: 0,
                total_duration,
            });
        }
        AppEvent::SingleVideoDetected { url, title, duration } => {
            // Clear loading state
            state.is_loading = false;
            state.loading_message = None;
            
            // Add single video to queue and trigger format fetching
            let mut item = app_state::DownloadItem::new(url.clone());
            let id = item.id;
            item.title = Some(title);
            item.duration = duration;
            item.status = app_state::DownloadStatus::FetchingInfo;
            state.queue.push(item);
            
            // Trigger format fetching for this single video
            // This will be handled by the existing FormatsFetched event
            // We need to add a DownloadAction for this
            // For now, we'll set it to Ready and let user manually trigger format fetch
            if let Some(item) = state.queue.iter_mut().find(|i| i.id == id) {
                item.status = app_state::DownloadStatus::Ready;
            }
        }
        AppEvent::PlaylistFetchFailed { error } => {
            // Clear loading state
            state.is_loading = false;
            state.loading_message = None;
            state.error_message = Some(error);
        }
    }
}
