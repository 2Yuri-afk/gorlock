# README Demo for Gorlock
# This creates a clean demo for the README
# Run: vhs readme-demo.tape && mv readme-demo.gif demo-preview.gif

Output readme-demo.gif

# Clean, professional settings with Nord theme
Set FontSize 14
Set Width 1000
Set Height 650
Set Theme "nord"
Set Padding 15
Set TypingSpeed 50ms
Set LineHeight 1.2

# Start clean
Hide
Type "clear"
Enter
Show

# Launch Gorlock with style
Type "# Launch Gorlock"
Sleep 500ms
Enter
Type "gorlock"
Enter
Sleep 2.5s

# Feature 1: Adding Rick Astley - Never Gonna Give You Up
Type "i"
Sleep 300ms
Type "https://www.youtube.com/watch?v=dQw4w9WgXcQ"  
Sleep 500ms
Type "  # Rick Astley!"
Backspace 14  # Remove the comment
Enter
Sleep 2.5s

# Feature 2: Fetch available formats
Type "f"
Sleep 3.5s

# Navigate through video formats
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Up
Sleep 300ms

# Feature 3: Toggle to audio-only formats
Type "t"
Sleep 1.5s
Down
Sleep 300ms

# Go back to video formats
Type "t"
Sleep 1s

# Select best quality format and start download
Enter
Sleep 3s  # Show download starting

# Feature 4: Queue management
Type "i"
Sleep 200ms
Type "https://vimeo.com/example"
Enter
Sleep 1.5s

# Navigate queue
Up
Sleep 300ms
Down
Sleep 300ms

# Feature 5: Panel switching
Tab
Sleep 800ms

# Show the clean exit
Sleep 1s
Type "q"
Sleep 300ms
