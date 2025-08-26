use anyhow::{Result, anyhow};
use regex::Regex;
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::app_state::{DownloadProgress, FormatInfo};

/// Validate if a URL is potentially supported by yt-dlp
pub fn is_valid_url(url: &str) -> bool {
    let url_regex = Regex::new(
        r"^https?://(www\.)?(youtube\.com|youtu\.be|vimeo\.com|dailymotion\.com|twitch\.tv|instagram\.com|twitter\.com|x\.com|tiktok\.com|facebook\.com|soundcloud\.com|spotify\.com|bandcamp\.com|archive\.org)/.*"
    ).unwrap();

    url_regex.is_match(url) || url.starts_with("http://") || url.starts_with("https://")
}

/// Fetch available formats for a given URL - handles both single videos and playlists
pub async fn fetch_formats(url: &str) -> Result<(Vec<FormatInfo>, String, Option<String>)> {
    let output = Command::new("yt-dlp")
        .args(["--dump-single-json", "--no-warnings", url])
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to fetch formats: {}", error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse JSON output directly
    let video_info: Value = serde_json::from_str(&stdout)?;
    
    // Extract title and duration from JSON
    let title = video_info["title"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
        
    let duration = video_info["duration_string"]
        .as_str()
        .filter(|s| !s.is_empty() && *s != "NA")
        .map(|s| s.to_string());

    let mut formats = Vec::new();

    if let Some(format_list) = video_info["formats"].as_array() {
        for format in format_list {
            if let Some(format_info) = parse_format_json(format) {
                formats.push(format_info);
            }
        }
    }

    // Sort formats: video formats first (by resolution), then audio formats
    formats.sort_by(|a, b| {
        match (a.is_audio_only, b.is_audio_only) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => {
                // Both are video, sort by resolution (higher first)
                let a_height = parse_height(&a.resolution);
                let b_height = parse_height(&b.resolution);
                b_height.cmp(&a_height)
            }
            (true, true) => {
                // Both are audio, sort by filesize or quality
                b.filesize.unwrap_or(0).cmp(&a.filesize.unwrap_or(0))
            }
        }
    });

    Ok((formats, title, duration))
}

/// Fetch playlist entries for a given URL
pub async fn fetch_playlist_entries(url: &str) -> Result<Vec<(String, String, Option<String>)>> {
    let output = Command::new("yt-dlp")
        .args(["--dump-single-json", "--no-warnings", url])
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to fetch playlist info: {}", error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let playlist_info: Value = serde_json::from_str(&stdout)?;
    
    let mut entries = Vec::new();
    
    // Check if this is a playlist
    if let Some(entry_list) = playlist_info["entries"].as_array() {
        for entry in entry_list {
            // Use webpage_url for proper video URL, fallback to url if not available
            let entry_url = entry["webpage_url"]
                .as_str()
                .or_else(|| entry["url"].as_str());
                
            if let Some(url) = entry_url {
                let title = entry["title"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string();
                let duration = entry["duration_string"]
                    .as_str()
                    .filter(|s| !s.is_empty() && *s != "NA")
                    .map(|s| s.to_string());
                    
                entries.push((url.to_string(), title, duration));
            }
        }
    } else {
        // Not a playlist, return the single video
        let title = playlist_info["title"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let duration = playlist_info["duration_string"]
            .as_str()
            .filter(|s| !s.is_empty() && *s != "NA")
            .map(|s| s.to_string());
            
        entries.push((url.to_string(), title, duration));
    }
    
    Ok(entries)
}

/// Parse formats from yt-dlp --list-formats output
fn parse_formats(output: &str) -> Vec<FormatInfo> {
    let mut formats = Vec::new();
    
    for line in output.lines() {
        if let Some(format_info) = parse_format_line(line) {
            formats.push(format_info);
        }
    }
    
    formats
}

/// Parse a single line from --list-formats output
fn parse_format_line(line: &str) -> Option<FormatInfo> {
    // Skip header lines and empty lines
    if line.is_empty() || line.starts_with('[') || line.contains("ID") && line.contains("EXT") {
        return None;
    }
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    
    let format_id = parts[0].to_string();
    if !format_id.chars().all(|c| c.is_ascii_digit() || c == '+') {
        return None;
    }
    
    let ext = parts.get(1).unwrap_or(&"unknown").to_string();
    
    // Try to extract resolution
    let resolution = parts.iter()
        .find(|part| part.contains('x') && part.chars().any(|c| c.is_ascii_digit()))
        .map(|s| s.to_string());
    
    // Determine if audio only
    let is_audio_only = line.contains("audio only") || 
                       ext == "m4a" || ext == "mp3" || ext == "wav" || ext == "flac";
    
    Some(FormatInfo {
        format_id,
        ext,
        resolution,
        fps: None,
        vcodec: None,
        acodec: None,
        filesize: None,
        quality: None,
        is_audio_only,
    })
}

/// Extract title from --list-formats output
fn extract_title_from_output(output: &str) -> Option<String> {
    for line in output.lines().take(5) {
        if line.starts_with('[') && line.contains(']') {
            // Look for pattern like "[youtube] dQw4w9WgXcQ: Rick Astley - Never Gonna Give You Up"
            if let Some(colon_pos) = line.find(':') {
                let title = line[colon_pos + 1..].trim();
                if !title.is_empty() && !title.contains("Downloading") && !title.contains("Extracting") {
                    return Some(title.to_string());
                }
            }
        }
    }
    None
}

/// Shorten a URL for display
fn shorten_url(url: &str) -> String {
    if url.len() > 50 {
        format!("{}...", &url[..47])
    } else {
        url.to_string()
    }
}

/// Parse a format from yt-dlp JSON output
fn parse_format_json(format: &Value) -> Option<FormatInfo> {
    let format_id = format["format_id"].as_str()?.to_string();
    let ext = format["ext"].as_str().unwrap_or("unknown").to_string();

    let resolution = match (format["width"].as_u64(), format["height"].as_u64()) {
        (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
        _ => format["resolution"].as_str().map(|s| s.to_string()),
    };

    let fps = format["fps"].as_f64();
    let vcodec = format["vcodec"].as_str().map(|s| s.to_string());
    let acodec = format["acodec"].as_str().map(|s| s.to_string());
    let filesize = format["filesize"]
        .as_u64()
        .or_else(|| format["filesize_approx"].as_u64());

    let quality = format["quality"].as_str().map(|s| s.to_string());

    // Determine if this is audio-only
    let is_audio_only = vcodec.as_deref() == Some("none") || (vcodec.is_none() && acodec.is_some());

    Some(FormatInfo {
        format_id,
        ext,
        resolution,
        fps,
        vcodec,
        acodec,
        filesize,
        quality,
        is_audio_only,
    })
}

/// Parse height from resolution string for sorting
fn parse_height(resolution: &Option<String>) -> u32 {
    if let Some(res) = resolution {
        if let Some(captures) = Regex::new(r"(\d+)x(\d+)").unwrap().captures(res) {
            if let Ok(height) = captures.get(2).unwrap().as_str().parse::<u32>() {
                return height;
            }
        }
        // Try to parse just height (like "720p")
        if let Some(captures) = Regex::new(r"(\d+)p?").unwrap().captures(res) {
            if let Ok(height) = captures.get(1).unwrap().as_str().parse::<u32>() {
                return height;
            }
        }
    }
    0
}

/// Fetch complete video information including thumbnail
pub async fn fetch_video_info(url: &str) -> Result<(String, Option<String>, Option<String>)> {
    let output = Command::new("yt-dlp")
        .args(["--dump-single-json", "--no-warnings", url])
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to fetch video info: {}", error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let video_info: Value = serde_json::from_str(&stdout)?;
    
    let title = video_info["title"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
        
    let duration = video_info["duration_string"]
        .as_str()
        .filter(|s| !s.is_empty() && *s != "NA")
        .map(|s| s.to_string());
        
    let thumbnail_url = video_info["thumbnail"]
        .as_str()
        .map(|s| s.to_string());
    
    Ok((title, duration, thumbnail_url))
}


/// Start a download with progress updates
pub async fn start_download(
    url: &str,
    format_id: &str,
    output_dir: &str,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
) -> Result<()> {
    // Determine the actual format string to use
    let format_string = if format_id.contains("audio_only") {
        // For audio-only downloads, use the format as-is
        format_id.replace("audio_only_", "")
    } else {
        // For video downloads, ensure we get both video and audio
        // Use format+bestaudio to merge video with best audio
        format!("{}+bestaudio/best", format_id)
    };

    let mut cmd = Command::new("yt-dlp")
        .args([
            "--format", &format_string,
            "--output", &format!("{}/%(title)s.%(ext)s", output_dir),
            "--merge-output-format", "mp4", // Ensure merged output is mp4
            "--newline",
            "--progress",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = cmd
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let mut reader = BufReader::new(stdout).lines();

    // Read progress lines
    while let Some(line) = reader.next_line().await? {
        if let Some(progress) = parse_progress_line(&line) {
            if progress_tx.send(progress).is_err() {
                break; // Channel closed, download cancelled
            }
        }
    }

    let status = cmd.wait().await?;
    if !status.success() {
        return Err(anyhow!("Download failed with exit code: {}", status));
    }

    Ok(())
}

/// Parse a progress line from yt-dlp output
fn parse_progress_line(line: &str) -> Option<DownloadProgress> {
    // yt-dlp progress format: [download]  12.3% of 45.67MiB at 1.23MiB/s ETA 00:34
    if !line.starts_with("[download]") {
        return None;
    }

    let progress_regex = Regex::new(
        r"\[download\]\s+(?P<percent>\d+\.?\d*)%(?:\s+of\s+(?P<total>\S+))?(?:\s+at\s+(?P<speed>\S+))?(?:\s+ETA\s+(?P<eta>\S+))?"
    ).unwrap();

    if let Some(captures) = progress_regex.captures(line) {
        let percent = captures.name("percent")?.as_str().parse().ok()?;
        let total_size = captures.name("total").map(|m| m.as_str().to_string());
        let speed = captures.name("speed").map(|m| m.as_str().to_string());
        let eta = captures.name("eta").map(|m| m.as_str().to_string());

        return Some(DownloadProgress {
            percent,
            speed,
            eta,
            downloaded: None, // Could be calculated from percent and total
            total_size,
        });
    }

    None
}

/// Validate a URL by attempting to extract info without downloading
pub async fn validate_url(url: &str) -> Result<bool> {
    let output = Command::new("yt-dlp")
        .args(["--simulate", "--quiet", "--no-warnings", url])
        .output()
        .await?;

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(is_valid_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(is_valid_url("https://vimeo.com/123456789"));
        assert!(!is_valid_url("not a url"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_progress_parsing() {
        let line = "[download]  45.6% of 123.45MiB at 2.34MiB/s ETA 01:23";
        let progress = parse_progress_line(line).unwrap();

        assert_eq!(progress.percent, 45.6);
        assert_eq!(progress.total_size, Some("123.45MiB".to_string()));
        assert_eq!(progress.speed, Some("2.34MiB/s".to_string()));
        assert_eq!(progress.eta, Some("01:23".to_string()));
    }

}
