# Regenerate from the repo root (needs a configured provider):
#   cargo build --release && vhs assets/demo.tape
Output assets/demo.gif

Set Shell bash
Set FontSize 16
Set Width 640
Set Height 260
Set Padding 20
# Catppuccin Mocha with the darker "crust" background
Set Theme { "name": "Catppuccin Mocha (crust bg)", "background": "#11111b", "foreground": "#cdd6f4", "cursor": "#f5e0dc", "selection": "#585b70", "black": "#45475a", "red": "#f38ba8", "green": "#a6e3a1", "yellow": "#f9e2af", "blue": "#89b4fa", "magenta": "#f5c2e7", "cyan": "#94e2d5", "white": "#bac2de", "brightBlack": "#585b70", "brightRed": "#f38ba8", "brightGreen": "#a6e3a1", "brightYellow": "#f9e2af", "brightBlue": "#89b4fa", "brightMagenta": "#f5c2e7", "brightCyan": "#94e2d5", "brightWhite": "#a6adc8" }
Set TypingSpeed 60ms

Hide
Type "export PATH=$PWD/target/release:$PATH PS1='$ ' && clear"
Enter
Show

Sleep 500ms
Type "usus claude"
Sleep 500ms
Enter
Sleep 5s
