use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app_state::{AppState, DownloadStatus, Panel, format_bytes};
use crate::commands::is_valid_url;

const GORLOCK_ASCII: &str = r#"┌────────────────────────────────────────────────────┐
│      _____ ____  ____  _     ____  ____  _  __     │
│     /  __//  _ \/  __\/ \   /  _ \/   _\/ |/ /     │
│     | |  _| / \||  \/|| |   | / \||  /  |   /      │
│     | |_//| \_/||    /| |_/\| \_/||  \_ |   \      │
│     \____\\____/\_/\_\\____/\____/\____/\_|\_\     │
└────────────────────────────────────────────────────┘"#;

pub struct App {
    pub list_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self { list_state }
    }
}

impl App {
    /// Render the ASCII art header
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new(GORLOCK_ASCII)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default());
        
        f.render_widget(header, area);
    }

    /// Render the complete UI
    pub fn render(&mut self, f: &mut Frame, state: &AppState) {
        let size = f.size();

        // Main layout with ASCII header at top
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // ASCII art header
                Constraint::Min(10),    // Main area
                Constraint::Length(3),  // Input area
                Constraint::Length(1),  // Status bar
            ])
            .split(size);
        
        // Render ASCII header
        self.render_header(f, chunks[0]);

        // Split main area into queue (left) and details (right)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // Render queue list
        self.render_queue(f, main_chunks[0], state);

        // Render details panel
        self.render_details(f, main_chunks[1], state);

        // Render input area
        self.render_input(f, chunks[2], state);

        // Render status bar
        self.render_status_bar(f, chunks[3], state);

        // Render popups if any
        if state.format_popup.is_some() {
            self.render_format_popup(f, size, state);
        }

        if state.error_message.is_some() {
            self.render_error_popup(f, size, state);
        }
        
        // Render loading indicator
        if state.is_loading {
            self.render_loading_indicator(f, size, state);
        }
        
        // Render playlist preview popup
        if state.playlist_preview.is_some() {
            self.render_playlist_preview(f, size, state);
        }
    }

    /// Render the download queue list
    fn render_queue(&mut self, f: &mut Frame, area: Rect, state: &AppState) {
        let items: Vec<ListItem> = state
            .queue
            .iter()
            .enumerate()
            .map(|(i, item)| {
                // Use clean title, fallback to shortened URL if no title
                let title = if let Some(title) = &item.title {
                    // Clean up the title - remove common prefixes/suffixes
                    title.trim().to_string()
                } else {
                    // Show shortened URL
                    if item.url.len() > 50 {
                        format!("{}...", &item.url[..47])
                    } else {
                        item.url.clone()
                    }
                };
                
                let status_style = match item.status {
                    DownloadStatus::Completed => Style::default().fg(Color::Green),
                    DownloadStatus::Failed => Style::default().fg(Color::Red),
                    DownloadStatus::Downloading => Style::default().fg(Color::Yellow),
                    DownloadStatus::Paused => Style::default().fg(Color::Cyan),
                    _ => Style::default(),
                };

                let progress_bar = if item.progress.percent > 0.0 {
                    format!(" [{:.1}%]", item.progress.percent)
                } else {
                    String::new()
                };

                let line = Line::from(vec![
                    Span::styled(format!("{}. {}", i + 1, title), Style::default()),
                    Span::styled(progress_bar, Style::default().fg(Color::Blue)),
                    Span::styled(format!(" ({})", item.status), status_style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Download Queue")
                    .borders(Borders::ALL)
                    .border_style(if state.current_panel == Panel::Queue {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .bg(Color::DarkGray),
            )
            .highlight_symbol(">> ");

        // Update list state selection
        if !state.queue.is_empty() {
            self.list_state
                .select(Some(state.selected_index.min(state.queue.len() - 1)));
        } else {
            self.list_state.select(None);
        }

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Render the details panel
    fn render_details(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let selected_item = state.queue.get(state.selected_index);

        let content = if let Some(item) = selected_item {
            let mut lines = vec![];
            
            // Title (only if different from URL)
            if let Some(title) = &item.title {
                if title != &item.url && !item.url.contains(title) {
                    lines.push(Line::from(vec![
                        Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                        Span::raw(title),
                    ]));
                }
            }
            
            // Duration
            if let Some(duration) = &item.duration {
                lines.push(Line::from(vec![
                    Span::styled("Duration: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                    Span::raw(duration),
                ]));
            }
            
            // Format details
            if let Some(format) = &item.format {
                lines.push(Line::from(vec![
                    Span::styled("Format: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                    Span::raw(format.display_name()),
                ]));
                
                // Resolution and FPS on separate line if available
                if let Some(resolution) = &format.resolution {
                    let resolution_info = if let Some(fps) = format.fps {
                        format!("{} @ {}fps", resolution, fps)
                    } else {
                        resolution.clone()
                    };
                    lines.push(Line::from(vec![
                        Span::styled("Quality: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                        Span::raw(resolution_info),
                    ]));
                }
                
                // File size if available
                if let Some(size) = format.filesize {
                    lines.push(Line::from(vec![
                        Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                        Span::raw(format_bytes(size)),
                    ]));
                }
            }
            
            // Status with color coding
            let status_color = match item.status {
                crate::app_state::DownloadStatus::Completed => Color::Green,
                crate::app_state::DownloadStatus::Failed => Color::Red,
                crate::app_state::DownloadStatus::Downloading => Color::Yellow,
                crate::app_state::DownloadStatus::Paused => Color::Cyan,
                _ => Color::White,
            };
            
            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                Span::styled(item.status.to_string(), Style::default().fg(status_color)),
            ]));
            
            // Created at
            let created_time = item.created_at.format("%H:%M:%S").to_string();
            lines.push(Line::from(vec![
                Span::styled("Added: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                Span::raw(created_time),
            ]));
            
            
            // Output directory
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Output: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
                Span::raw(&state.output_dir),
            ]));
            
            // Error message if any
            if let Some(error) = &item.error {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                ]));
                lines.push(Line::from(Span::styled(error, Style::default().fg(Color::Red))));
            }
            
            // Progress info for downloading items
            if item.status == crate::app_state::DownloadStatus::Downloading {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Progress:",
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
                )]));
                
                if let Some(speed) = &item.progress.speed {
                    lines.push(Line::from(vec![
                        Span::styled("  Speed: ", Style::default().fg(Color::Gray)),
                        Span::raw(speed),
                    ]));
                }
                
                if let Some(eta) = &item.progress.eta {
                    lines.push(Line::from(vec![
                        Span::styled("  ETA: ", Style::default().fg(Color::Gray)),
                        Span::raw(eta),
                    ]));
                }
                
                if let Some(total_size) = &item.progress.total_size {
                    lines.push(Line::from(vec![
                        Span::styled("  Total: ", Style::default().fg(Color::Gray)),
                        Span::raw(total_size),
                    ]));
                }
            }
            
            lines
        } else {
            vec![
                Line::from(Span::styled(
                    "No item selected",
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Select an item from the queue to see details",
                    Style::default().fg(Color::Cyan)
                )),
            ]
        };

        let details = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Details")
                    .borders(Borders::ALL)
                    .border_style(if state.current_panel == Panel::Details {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(details, area);

        // Render progress bar if item is downloading
        if let Some(item) = selected_item {
            if item.status == DownloadStatus::Downloading && item.progress.percent > 0.0 {
                let progress_area = Rect {
                    x: area.x + 1,
                    y: area.y + area.height - 3,
                    width: area.width - 2,
                    height: 1,
                };

                let progress_label = format!(
                    "{:.1}%{}{}",
                    item.progress.percent,
                    item.progress
                        .speed
                        .as_ref()
                        .map(|s| format!(" @ {}", s))
                        .unwrap_or_default(),
                    item.progress
                        .eta
                        .as_ref()
                        .map(|s| format!(" ETA {}", s))
                        .unwrap_or_default()
                );

                let gauge = Gauge::default()
                    .block(Block::default())
                    .gauge_style(Style::default().fg(Color::Blue))
                    .percent(item.progress.percent as u16)
                    .label(progress_label);

                f.render_widget(gauge, progress_area);
            }
        }
    }

    /// Render the URL input area
    fn render_input(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let input_style = if state.input_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let is_valid = is_valid_url(&state.url_input);
        let validation_color = if state.url_input.is_empty() {
            Color::Gray
        } else if is_valid {
            Color::Green
        } else {
            Color::Red
        };

        let input = Paragraph::new(state.url_input.as_str())
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Enter URL (Press 'i' to input, Enter to add)")
                    .border_style(if state.current_panel == Panel::Input {
                        Style::default().fg(validation_color)
                    } else {
                        Style::default()
                    }),
            );

        f.render_widget(input, area);

        // Set cursor position when in input mode
        if state.input_mode {
            f.set_cursor(
                area.x + state.url_input.chars().count() as u16 + 1,
                area.y + 1,
            );
        }
    }

    /// Render the status bar
    fn render_status_bar(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let help_text = if state.input_mode {
            "ESC: exit input | Enter: add URL | Ctrl+C: quit"
        } else if state.format_popup.is_some() {
            "↑/↓: navigate formats | Enter: select & download | t: toggle audio-only | ESC: cancel"
        } else {
            "i: input URL | f: fetch formats | d: delete | q: quit | ↑/↓: navigate"
        };

        let status_info = format!(
            " {} items | Output: {} ",
            state.queue.len(),
            state.output_dir
        );

        let status = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left);

        let info_width = status_info.len() as u16;
        let info = Paragraph::new(status_info)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Right);

        f.render_widget(status, area);

        // Render the info text on the right side
        let info_area = Rect {
            x: area.width.saturating_sub(info_width),
            y: area.y,
            width: info_width,
            height: area.height,
        };
        f.render_widget(info, info_area);
    }

    /// Render format selection popup
    fn render_format_popup(&self, f: &mut Frame, area: Rect, state: &AppState) {
        if let Some(popup) = &state.format_popup {
            let popup_area = self.centered_rect(80, 60, area);

            // Clear background
            f.render_widget(Clear, popup_area);

            // Filter formats based on audio_only_filter
            let filtered_formats: Vec<(usize, &crate::app_state::FormatInfo)> = popup
                .formats
                .iter()
                .enumerate()
                .filter(|(_, format)| {
                    if popup.audio_only_filter {
                        format.is_audio_only
                    } else {
                        true // Show all formats when filter is off
                    }
                })
                .collect();

            let items: Vec<ListItem> = filtered_formats
                .iter()
                .enumerate()
                .map(|(display_idx, (_, format))| {
                    let is_selected = display_idx == popup.selected_index;
                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(format.display_name(), style),
                    ]))
                })
                .collect();

            let title = if popup.audio_only_filter {
                "Select Format (Audio Only)"
            } else {
                "Select Format (All)"
            };
            
            let list = List::new(items)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .highlight_style(Style::default().bg(Color::DarkGray));

            f.render_widget(list, popup_area);

            // Help text at bottom
            let help_area = Rect {
                x: popup_area.x + 1,
                y: popup_area.y + popup_area.height - 2,
                width: popup_area.width - 2,
                height: 1,
            };

            let help = Paragraph::new(
                "↑/↓: navigate | Enter: select | ESC: cancel | t: toggle audio-only",
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

            f.render_widget(help, help_area);
        }
    }

    /// Render error popup
    fn render_error_popup(&self, f: &mut Frame, area: Rect, state: &AppState) {
        if let Some(error) = &state.error_message {
            let popup_area = self.centered_rect(60, 20, area);

            // Clear background
            f.render_widget(Clear, popup_area);

            let error_text = Paragraph::new(error.as_str())
                .block(
                    Block::default()
                        .title("Error")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red)),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);

            f.render_widget(error_text, popup_area);

            // Help text
            let help_area = Rect {
                x: popup_area.x + 1,
                y: popup_area.y + popup_area.height - 2,
                width: popup_area.width - 2,
                height: 1,
            };

            let help = Paragraph::new("Press any key to close")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            f.render_widget(help, help_area);
        }
    }

    /// Render loading indicator
    fn render_loading_indicator(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let popup_area = self.centered_rect(50, 15, area);

        // Clear background
        f.render_widget(Clear, popup_area);

        // Create loading message with spinner animation
        let loading_frames = vec!["⣷", "⣯", "⣟", "⡿", "⢿", "⣻", "⣽", "⣾"];
        let frame_index = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            / 100) as usize
            % loading_frames.len();
        let spinner = loading_frames[frame_index];

        let message = state.loading_message.as_deref().unwrap_or("Processing URL...");
        let loading_text = format!("{} {}", spinner, message);

        let loading_widget = Paragraph::new(loading_text)
            .block(
                Block::default()
                    .title("Loading")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));

        f.render_widget(loading_widget, popup_area);

        // Subtitle with helpful message
        let subtitle_area = Rect {
            x: popup_area.x + 2,
            y: popup_area.y + 3,
            width: popup_area.width - 4,
            height: 1,
        };

        let subtitle = Paragraph::new("Please wait while fetching video information...")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(subtitle, subtitle_area);
    }

    /// Render playlist preview popup
    fn render_playlist_preview(&self, f: &mut Frame, area: Rect, state: &AppState) {
        if let Some(preview) = &state.playlist_preview {
            let popup_area = self.centered_rect(70, 70, area);

            // Clear background
            f.render_widget(Clear, popup_area);

            // Create title with count and total duration
            let title = format!(
                "Playlist Preview - {} items{}",
                preview.entries.len(),
                preview.total_duration
                    .as_ref()
                    .map(|d| format!(" • {}", d))
                    .unwrap_or_default()
            );

            // Create list items
            let items: Vec<ListItem> = preview
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    let is_selected = i == preview.selected_index;
                    let style = if is_selected {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };

                    let duration_str = entry.duration.as_deref().unwrap_or("");
                    let line = if !duration_str.is_empty() {
                        format!("{}. {} ({})", i + 1, entry.title, duration_str)
                    } else {
                        format!("{}. {}", i + 1, entry.title)
                    };

                    ListItem::new(Line::from(vec![Span::styled(line, style)]))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Magenta)),
                )
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol(">> ");

            // Create a ListState for scrolling
            let mut list_state = ListState::default();
            list_state.select(Some(preview.selected_index));

            f.render_stateful_widget(list, popup_area, &mut list_state);

            // Help text at bottom
            let help_area = Rect {
                x: popup_area.x + 1,
                y: popup_area.y + popup_area.height - 2,
                width: popup_area.width - 2,
                height: 1,
            };

            let help = Paragraph::new(
                "↑/↓: navigate | Enter: add all to queue | ESC: cancel",
            )
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);

            f.render_widget(help, help_area);
        }
    }

    /// Helper function to create a centered rectangle
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
