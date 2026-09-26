# monitormenu

A launcher-driven monitor manager for Hyprland, allowing you to manage monitor configurations through your favorite application launcher.

## Features

- List all connected monitors with their current status
- Per monitor, change and see the current value of:
  - resolution and refresh rate
  - scale (1, 1.25, 1.5, 1.6, 2)
  - rotation
  - placement: extend right of / left of / above / below another monitor, or mirror it (Hyprland)
  - VRR
  - HDR (Hyprland; switches to 10-bit colour)
  - enable / disable (the last lit monitor can't be disabled)
- Every change keeps the monitor's other settings: the whole output state is read back and applied, never a partial rule
- After each change the menu returns with the new state, so several changes need no restart
- "Show as Nix" copies the monitor's current state as a nix-home `my.window-manager.monitors.<name>` entry (and shows it in a notification), to make a runtime setup permanent
- Hyprland and niri backends; walker, rofi, dmenu, fuzzel and vicinae launchers

Changes are runtime-only: a compositor reload goes back to its config.

## Requirements

- Hyprland or niri
- `wl-copy` and `notify-send` for "Show as Nix" (bundled by the Nix package)
- One of the supported launchers:
  - walker (default)
  - rofi
  - dmenu
  - fuzzel
  - vicinae

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
monitormenu --launcher vicinae
```

### Workflow

1. Run `monitormenu`
2. Select a monitor
3. Pick a setting (entries ending in ▸ open a list, with ● on the current value); toggles apply at once
4. The settings menu comes back with the new state; press Escape to finish

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
