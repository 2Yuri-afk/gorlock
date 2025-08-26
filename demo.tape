# VHS Tape file for Gorlock Demo
# To generate: vhs demo.tape

# Configuration
Output demo.gif
Set Shell "bash"
Set FontSize 18
Set Width 1600
Set Height 900
Set Padding 20
Set Theme "Dracula"
Set TypingSpeed 75ms
Set PlaybackSpeed 1

# Start the demo
Type "gorlock" Enter
Sleep 2s

# Show the beautiful ASCII header
Screenshot gorlock-header.png
Sleep 1s

# Demo 1: Add a single video URL
Type "i"
Sleep 500ms
Type "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
Sleep 500ms
Enter
Sleep 2s

# Navigate and fetch formats
Type "f"
Sleep 3s

# Navigate through formats
Down
Sleep 300ms
Down
Sleep 300ms
Up
Sleep 300ms

# Toggle audio-only filter
Type "t"
Sleep 1s
Down
Sleep 300ms

# Select a format and download
Enter
Sleep 2s

# Demo 2: Queue management - add another URL
Type "i"
Sleep 500ms
Type "https://www.youtube.com/watch?v=9bZkp7q19f0"
Enter
Sleep 2s

# Navigate through queue
Up
Sleep 500ms
Down
Sleep 500ms

# Show details panel
Tab
Sleep 1s
Tab
Sleep 1s

# Demo 3: Delete item from queue
Type "d"
Sleep 1s

# Demo 4: Playlist preview (simulated)
Type "i"
Sleep 500ms
Type "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"
Enter
Sleep 3s

# Navigate through playlist preview
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms

# Accept playlist
Enter
Sleep 2s

# Show final state
Screenshot gorlock-final.png
Sleep 2s

# Exit
Type "q"
Sleep 500ms
