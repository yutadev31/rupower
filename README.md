# rupower

`rupower` is a small graphical power menu for Linux desktops. It provides buttons for power actions and can run as a Wayland layer surface or an X11 window.

## Features

- Power off, reboot, suspend, log out, and lock the current session.
- Mouse and keyboard interaction.
- Optional action buttons and layout customization through TOML.
- Compositor and window-manager aware logout support for Hyprland, Sway, and i3.
- Reproducible builds through Nix.

## Requirements

The build requires Rust, `pkg-config`, Wayland development files, and XCB development files. At runtime, the commands used by enabled actions must be available on the system.

On Nix-enabled systems, enter the development shell with:

```sh
nix develop
```

## Build and run

```sh
cargo build --release
cargo run
```

## Configuration

The optional configuration file is:

```text
~/.config/rupower/config.toml
```

Example:

```toml
[style]
padding = 16.0
button_width = 104.0
button_height = 124.0

[actions]
poweroff = true
reboot = true
suspend = false
logout = true
lock = true
```

The default values are used when the file does not exist. Invalid style dimensions cause configuration loading to fail with an error.

## Keyboard shortcuts

Each action displays its shortcut in the menu:

- `Shift+P`: Power off
- `Shift+R`: Reboot
- `Shift+S`: Suspend
- `Shift+L`: Log out
- `Shift+K`: Lock

Selecting an action once highlights it; selecting it again runs the system command.
The menu displays a confirmation hint after the first selection. Pressing `Enter`
confirms the selected action, while `Escape`, `q`, or `x` closes the menu.

## License

This project is licensed under the MIT License. See
[`LICENSE.txt`](LICENSE.txt) for the full license text.
