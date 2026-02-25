Output demo.gif
Set Width 900
Set Height 600
Set FontSize 14
Set Padding 20
Set Theme { "name": "Catppuccin Mocha", "black": "#45475a", "red": "#f38ba8", "green": "#a6e3a1", "yellow": "#f9e2af", "blue": "#89b4fa", "magenta": "#f5c2e7", "cyan": "#94e2d5", "white": "#bac2de", "brightBlack": "#585b70", "brightRed": "#f38ba8", "brightGreen": "#a6e3a1", "brightYellow": "#f9e2af", "brightBlue": "#89b4fa", "brightMagenta": "#f5c2e7", "brightCyan": "#94e2d5", "brightWhite": "#a6adc8", "background": "#1e1e2e", "foreground": "#cdd6f4", "selection": "#585b70", "cursor": "#f5e0dc" }

Sleep 500ms

# 1. Colored help text — the hero shot
Type "cargo run --example full_demo --features color -- --help"
Enter
Sleep 15s

# 2. Successful parse with flags, options, and defaults
Type "cargo run --example full_demo --features color -- --file data.csv --json -v --port 3000"
Enter
Sleep 4s

# 3. Conflict violation — mutually exclusive formats
Type "cargo run --example full_demo --features color -- --json --csv --file x"
Enter
Sleep 4s

# 4. Validation error — port out of range
Type "cargo run --example full_demo --features color -- --file x --port 99999"
Enter
Sleep 4s

# 5. Version flag
Type "cargo run --example full_demo --features color -- --version"
Enter
Sleep 3s
