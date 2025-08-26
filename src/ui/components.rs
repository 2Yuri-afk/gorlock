// This file contains reusable UI components for the TUI application
// Currently serves as a stub to satisfy compiler requirements

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

/// Helper function to create a centered rectangle for popups
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

/// Get validation color for URL input field
pub fn get_validation_color(input: &str, is_valid: bool) -> Color {
    if input.is_empty() {
        Color::Gray
    } else if is_valid {
        Color::Green
    } else {
        Color::Red
    }
}
