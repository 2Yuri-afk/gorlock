# VHS Tape file for Gorlock Showcase
# Generate with: vhs showcase.tape

# Configuration
Output showcase.gif
Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 10
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

# Clear screen and show Gorlock
Hide
Type "clear" Enter
Show

# Start Gorlock
Type "gorlock" Enter
Sleep 3s

# Showcase 1: Input a URL
Type "i"
Sleep 500ms
Type "https://youtu.be/example123"
Sleep 1s
Screenshot input-mode.png
Enter
Sleep 2s

# Showcase 2: Navigate queue
Down
Sleep 500ms
Up
Sleep 500ms

# Showcase 3: Show format selection
Type "f"
Sleep 2s
Screenshot format-selection.png
Sleep 1s

# Navigate formats
Down
Sleep 400ms
Down
Sleep 400ms

# Toggle audio filter
Type "t"
Sleep 1s
Screenshot audio-only.png

# Cancel format selection
Escape
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
