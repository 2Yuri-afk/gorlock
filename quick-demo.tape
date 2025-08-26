# Quick Demo of Gorlock
# Run: vhs quick-demo.tape

Output quick-demo.gif

Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"

# Launch Gorlock
Type "gorlock"
Enter
Sleep 2s

# Add URL
Type "i"
Sleep 300ms
Type "https://youtube.com/watch?v=test"
Enter
Sleep 1.5s

# Fetch formats
Type "f"
Sleep 2s

# Show audio toggle
Type "t"
Sleep 1s
Type "t"
Sleep 1s

# Select format
Enter
Sleep 1.5s

# Navigate
Down
Up
Sleep 500ms

# Switch to details
Tab
Sleep 1s

# Exit
Type "q"
