# VHS Tape file for Gorlock Showcase
# Generate with: vhs showcase.tape

# Configuration
Output showcase.gif
Set Shell "bash"
Set FontSize 15
Set Width 1200
Set Height 750
Set Padding 12
Set Theme "nord"
Set TypingSpeed 45ms
Set LineHeight 1.15

# Clear screen and show Gorlock
Hide
Type "clear" Enter
Show

# Start Gorlock
Type "gorlock" Enter
Sleep 3s

# Showcase 1: Input Rick Astley URL
Type "i"
Sleep 400ms
Type "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
Sleep 800ms
Screenshot input-mode.png
Enter
Sleep 2.5s

# Showcase 2: Navigate queue
Down
Sleep 500ms
Up
Sleep 500ms

# Showcase 3: Show format selection
Type "f"
Sleep 3s
Screenshot format-selection.png
Sleep 1s

# Navigate through video formats
Down
Sleep 350ms
Down
Sleep 350ms
Down
Sleep 350ms

# Toggle to audio-only formats
Type "t"
Sleep 1.5s
Screenshot audio-only.png
Down
Sleep 400ms

# Select format to download
Enter
Sleep 3s
Screenshot downloading.png
Sleep 1s

# Showcase 4: Switch panels
Tab
Sleep 800ms
Screenshot details-panel.png
Tab
Sleep 800ms

# Showcase 5: Delete from queue
Type "d"
Sleep 1s

# Add another URL for variety
Type "i"
Sleep 500ms
Type "https://vimeo.com/example456"
Enter
Sleep 2s

# Final screenshot
Screenshot final-state.png
Sleep 2s

# Exit gracefully
Type "q"
Sleep 500ms
