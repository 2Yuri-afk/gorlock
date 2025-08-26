# Perfect Demo for Gorlock README
# Showcases all major features with Rick Astley
# Run: vhs perfect-demo.tape

Output perfect-demo.gif

# Beautiful Nord theme settings
Set FontSize 14
Set Width 1100
Set Height 700
Set Theme "nord"
Set Padding 20
Set TypingSpeed 60ms
Set LineHeight 1.1
Set CursorBlink false

# Start fresh
Hide
Type "clear"
Enter
Show

# Launch Gorlock
Type "# Gorlock - Beautiful TUI for yt-dlp"
Sleep 800ms
Enter
Type "gorlock"
Enter
Sleep 3s

# === Feature 1: Add Rick Astley's Never Gonna Give You Up ===
Type "i"  # Enter input mode
Sleep 400ms
Type "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
Sleep 800ms
Enter
Sleep 3s  # Wait for URL to be processed

# === Feature 2: Fetch and Browse Formats ===
Type "f"  # Fetch formats
Sleep 4s  # Wait for formats to load

# Navigate through video formats
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Up
Sleep 300ms

# === Feature 3: Audio-Only Toggle ===
Type "t"  # Switch to audio-only
Sleep 1.5s
Down
Sleep 400ms
Down
Sleep 400ms

# Switch back to all formats
Type "t"
Sleep 1s

# === Feature 4: Select Format and Start Download ===
Up
Up
Sleep 300ms
Enter  # Select format and start download
Sleep 4s  # Show download progress

# === Feature 5: Add Another Video to Queue ===
Type "i"
Sleep 300ms
Type "https://www.youtube.com/watch?v=oHg5SJYRHA0"
Enter
Sleep 2.5s

# === Feature 6: Queue Navigation ===
Up  # Navigate up in queue
Sleep 500ms
Down  # Navigate down
Sleep 500ms

# === Feature 7: Switch to Details Panel ===
Tab
Sleep 1.2s

# === Feature 8: Clean Exit ===
Sleep 1.5s
Type "q"
Sleep 500ms
