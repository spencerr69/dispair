# dispair

A work-in-progress TUI game being written in Rust about unpairing the [BAD] from the [GOOD].

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
