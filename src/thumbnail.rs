use anyhow::Result;
use image::{io::Reader, DynamicImage, GenericImageView};
use std::io::Cursor;

/// Convert an image URL to ASCII art
pub async fn fetch_and_convert_thumbnail(url: &str, width: u32, height: u32) -> Result<Vec<String>> {
    // Download the image
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    
    // Load and process the image
    let img = Reader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
        
    // Convert to ASCII
    let ascii_art = image_to_ascii(&img, width, height);
    Ok(ascii_art)
}

/// Convert a DynamicImage to ASCII art
fn image_to_ascii(img: &DynamicImage, target_width: u32, target_height: u32) -> Vec<String> {
    // Resize image to fit terminal dimensions
    let resized = img.resize(target_width, target_height, image::imageops::FilterType::Lanczos3);
    let grayscale = resized.grayscale();
    
    // ASCII characters from darkest to lightest
    let ascii_chars = " .:-=+*#%@";
    let char_array: Vec<char> = ascii_chars.chars().collect();
    
    let mut result = Vec::new();
    
    for y in 0..target_height {
        let mut line = String::new();
        for x in 0..target_width {
            let pixel = grayscale.get_pixel(x, y);
            let intensity = pixel[0] as f32 / 255.0;
            let char_index = (intensity * (char_array.len() - 1) as f32).round() as usize;
            line.push(char_array[char_index]);
        }
        result.push(line);
    }
    
    result
}

/// Generate a sad face ASCII art when no thumbnail is available
pub fn get_sad_face_ascii() -> Vec<String> {
    vec![
        "    ╭─────────╮".to_string(),
        "   ╱           ╲".to_string(),
        "  │   ◉     ◉   │".to_string(),
        "  │             │".to_string(),
        "  │      ◡      │".to_string(),
        "  │   ╲     ╱   │".to_string(),
        "  │    ╲___╱    │".to_string(),
        "   ╲           ╱".to_string(),
        "    ╲_________╱".to_string(),
        "".to_string(),
        "   No thumbnail".to_string(),
        "    available".to_string(),
    ]
}

/// Generate a simple loading placeholder
pub fn get_loading_ascii() -> Vec<String> {
    vec![
        "    ╭─────────╮".to_string(),
        "   ╱           ╲".to_string(),
        "  │             │".to_string(),
        "  │   Loading   │".to_string(),
        "  │      ...    │".to_string(),
        "  │             │".to_string(),
        "   ╲           ╱".to_string(),
        "    ╲_________╱".to_string(),
    ]
}
