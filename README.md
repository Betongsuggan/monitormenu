# monitormenu

A launcher-driven monitor manager for Hyprland, allowing you to manage monitor configurations through your favorite application launcher.

## Features

- List all connected monitors with their current status
- View and apply all supported resolutions and refresh rates
- Enable/disable monitors
- Integration with popular launchers (walker, rofi, dmenu, fuzzel)
- Clean two-stage menu interface

## Requirements

- Hyprland compositor
- One of the supported launchers:
  - walker (default)
  - rofi
  - dmenu
  - fuzzel

## Installation

### Using Nix Flakes

```bash
# Build the project
nix build

# Run directly
nix run

# Install to your profile
nix profile install
```

### Using Cargo

```bash
cargo build --release
cargo install --path .
```

## Usage

```bash
# Use with default launcher (walker)
monitormenu

# Use with specific launcher
monitormenu --launcher rofi
monitormenu --launcher dmenu
monitormenu --launcher fuzzel
```

### Workflow

1. Run `monitormenu`
2. Select a monitor from the first menu
3. Choose an action from the second menu:
   - Apply a resolution and refresh rate
   - Enable or disable the monitor

## Monitor Display Format

Monitors are displayed with the following information:
- Status icon: ✓ (enabled) or ✗ (disabled)
- Focus indicator: ● (focused) or space
- Monitor description
- Current resolution and refresh rate
- Monitor name (identifier)

Example:
```
✓ ● Samsung Odyssey G7 - 2560x1440@165.00Hz (DP-1)
✓   LG UltraWide - 3440x1440@100.00Hz (DP-2)
```

## Resolution Menu Format

Available modes are displayed with:
- Current mode indicator: ●
- Resolution format: widthxheight @ refresh_rate Hz

Example:
```
● Set: 2560x1440 @ 165.00 Hz
  Set: 2560x1440 @ 144.00 Hz
  Set: 1920x1080 @ 60.00 Hz
```

## Architecture

The project follows a modular architecture similar to [audiomenu](https://github.com/yourusername/audiomenu):

- `backend/hyprland.rs` - Hyprland integration via hyprctl
- `launcher/mod.rs` - Launcher abstraction layer
- `cli/mod.rs` - Command-line interface
- `main.rs` - Two-stage menu orchestration

## Future Enhancements

- Support for additional Wayland compositors (sway, river, etc.)
- Monitor positioning and arrangement
- Custom scaling configuration
- Monitor profiles and presets

## License

GPL-3.0

## Contributing

Contributions are welcome! This project is designed to be easily extensible to support additional Wayland compositors.
