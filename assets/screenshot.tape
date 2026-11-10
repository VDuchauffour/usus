# Regenerate with: vhs assets/screenshot.tape (run from the repo root, needs a configured provider)
Output target/screenshot.gif

Set Shell bash
Set FontSize 16
Set Width 640
Set Height 240
Set Padding 20
Set Theme "Catppuccin Mocha"

Hide
Type "export PATH=$PWD/target/release:$PATH PS1='$ ' && clear"
Enter
Show

Type "usus claude"
Sleep 300ms
Enter
Sleep 4s
Screenshot assets/screenshot.png
