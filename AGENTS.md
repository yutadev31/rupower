# AGENTS.md

## Project Overview

`rupower` is a Rust-based graphical power menu for Linux. It presents buttons for powering off, rebooting, suspending, logging out, and locking the current session. The menu is rendered as a Wayland layer surface or an X11 window through `shell-surface`.

The application loads optional configuration from `~/.config/rupower/config.toml`. Missing configuration files use the built-in defaults. Actions that are disabled in the configuration are omitted from the menu.

## Repository Layout

- `src/main.rs`: Application entry point, menu layout, rendering, input handling, keyboard shortcuts, and system action dispatch.
- `src/config.rs`: TOML configuration loading, defaults, and style validation.
- `Cargo.toml`: Rust package metadata and dependencies.
- `flake.nix`: Reproducible build and development shell definition.
- `.github/workflows/ci.yml`: Formatting, compilation, test, Clippy, and release checks.
- `.github/dependabot.yml`: Dependency update configuration for Nix, Cargo, and GitHub Actions.

## Development Workflow

Use the Nix development shell when available:

```sh
nix develop
```

Format and validate the project with Cargo before submitting changes:

```sh
cargo fmt --check
cargo check --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
```

Run the application locally with:

```sh
cargo run
```

The Linux build requires Wayland and XCB development libraries, along with `pkg-config`. The runtime action commands (`poweroff`, `reboot`, `systemctl`, `loginctl`, and the compositor or window-manager command used for logout) must also be available on the system.

## Configuration

Configuration is read from:

```text
~/.config/rupower/config.toml
```

The supported options are `style.padding`, `style.button_width`, `style.button_height`, and the boolean action switches `actions.poweroff`, `actions.reboot`, `actions.suspend`, `actions.logout`, and `actions.lock`. Style dimensions are validated when the file is loaded.

## Development Principles

- Do not make temporary workarounds that prioritize immediate behavior over a fundamental solution, or introduce fixes that are not root-cause fixes.
- When improving compatibility would reduce readability, prioritize readability over compatibility.
- Keep changes as small as necessary and do not combine them with unrelated refactoring.
- Do not silently ignore errors; handle them in a way that makes their causes identifiable whenever possible.
- Prioritize maintainability and runtime performance over ease of implementation.
- Avoid relying on external command execution whenever possible; when it is necessary, prefer using an external library or IPC instead.
- If the prompt is ambiguous or lacks necessary information, ask clarifying questions before proceeding.
- After completing work, output an English commit message following the Conventional Commits specification.
