# dispair

A work-in-progress TUI game being written in Rust about unpairing the [BAD] from the [GOOD].

## Overview

**dispair** is an incremental/roguelite game that runs natively in your terminal or in a web browser via WebAssembly. Fight off enemies, buy upgrades, and _l o o k  d e e p e r_

## Features

*   **Cross-Platform**: Play natively in your terminal or in the browser (WASM).
*   **Incremental Gameplay**: Earn resources to buy upgrades and improve your character.
*   **Roguelite Gameplay**: l o o k  d e e p e r  l o o k  d e e p e r  l o o k  d e e p e r  l o o k  d e e p e r  l o o k  d e e p e r  l o o k  d e e p e r  l o o k  d e e p e r  l o o k  d e e p e r 

## Controls

### In-Game

| Key | Action |
| :--- | :--- |
| `W` / `Up` | Move Up |
| `A` / `Left` | Move Left |
| `S` / `Down` | Move Down |
| `D` / `Right` | Move Right |
| `Esc` | Game Over / Exit |

### Upgrade Menu

| Key | Action |
| :--- | :--- |
| `W` / `Up` | Previous Selection |
| `S` / `Down` | Next Selection |
| `Enter` | Buy Upgrade / Enter Category |
| `Space` | Start Game |
| `Esc` | Back / Return to Menu |

## Installation & Development

### Prerequisites

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

### Native (Terminal)

To build and run the game natively in your terminal:

```bash
# Run in release mode for best performance
cargo run --release
```

### WebAssembly (Browser)

To build and run the web version, you will need [Trunk](https://trunkrs.dev/):

```bash
# Install Trunk
cargo install trunk

# Serve the application locally (in release mode for best performance)
trunk serve --release
```

Once running, open your browser to `http://127.0.0.1:8080`.

## Makes extensive use of:

*   **TUI Framework**: [Ratatui](https://ratatui.rs/)
*   **WASM Backend**: [Ratzilla](https://crates.io/crates/ratzilla)
