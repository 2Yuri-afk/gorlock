# Gorlock

A TUI wrapper for `yt-dlp` written in Rust. Download videos and audio from YouTube and 1000+ other sites with style.

![Gorlock Demo](demo-preview.gif)

## Features

- **Cool TUI** - Clean, modern interface with ASCII art branding
- **Smart Format Selection** - Interactive format picker with audio-only toggle
- **Queue Management** - Add multiple downloads and track their progress
- **Playlist Support** - Preview and confirm playlists before downloading
- **Real-time Progress** - Live download stats with speed, ETA, and progress bars
- **Kinda Fast** - Built with Rust for optimal performance
- **1000+ Sites** - Powered by yt-dlp, supports YouTube, Vimeo, Twitter, and more

## Quick Start

### Prerequisites

- `yt-dlp` installed and in your PATH
- Rust toolchain (for building from source)

### Installation

```bash
# Clone and install
git clone https://github.com/yourusername/gorlock.git
cd gorlock
cargo install --path .

# Run with either command
gorlock
# or the short alias
gl
```

## ⌨️ Keyboard Shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `i` | Enter URL input mode | Normal |
| `Enter` | Add URL to queue | Input mode |
| `f` | Fetch formats | Queue item selected |
| `Enter` | Download with selected format | Format popup |
| `t` | Toggle audio-only filter | Format popup |
| `d` | Delete from queue | Queue item selected |
| `↑/↓` or `j/k` | Navigate | Any list |
| `Tab` | Switch panels | Normal |
| `Esc` | Cancel/Back | Any popup |
| `q` | Quit | Normal |
| `Ctrl+C` | Force quit | Any time |

## Usage Examples

### Download a single video
1. Launch with `gl`
2. Press `i` to enter input mode
3. Paste URL and press `Enter`
4. Press `f` to see formats
5. Select format and press `Enter`

### Download audio only
1. Add URL as above
2. Press `f` for formats
3. Press `t` to filter audio-only
4. Select format and download

### Handle playlists
- When you paste a playlist URL, Gorlock shows a preview
- Navigate through videos with `↑/↓`
- Press `Enter` to add all to queue
- Press `Esc` to cancel

## Building from Source

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run development version
cargo run

# Install globally
cargo install --path .
```

## Project Structure

```
src/
├── main.rs              # Application entry & event loop
├── app_state/           # State management
│   ├── mod.rs          # Core state structures
│   └── events.rs       # Event definitions
├── ui/                  # User interface
│   ├── app.rs          # Main UI rendering
│   ├── events.rs       # Input handling
│   └── components.rs   # Reusable UI parts
└── commands/            # External commands
    ├── mod.rs          # Command orchestration
    └── yt_dlp.rs       # yt-dlp integration
```

## Contributing

Contributions are welcome! Feel free to:

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## TODO / Roadmap

- [ ] Configuration file support
- [ ] Custom output directory per download
- [ ] Download history/log
- [ ] Concurrent downloads
- [ ] Resume interrupted downloads
- [ ] Subtitle download options
- [ ] Authentication support
- [ ] Bandwidth limiting
- [ ] Post-download actions

## License

MIT License - see [LICENSE](LICENSE) file for details

## 🙏 Credits

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) - Amazing TUI framework
- Powered by [yt-dlp](https://github.com/yt-dlp/yt-dlp) - The downloading engine
- Inspired by the need for a better downloading experience


