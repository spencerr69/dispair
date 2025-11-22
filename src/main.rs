//! The main entry point for the application, responsible for initializing and running
//! the appropriate version (terminal or WASM) of the game.

pub mod common;

#[cfg(not(target_family = "wasm"))]
pub mod terminal;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[cfg(target_family = "wasm")]
pub mod target_types {
    pub type KeyCode = ratzilla::event::KeyCode;
    pub type KeyEvent = ratzilla::event::KeyEvent;
    pub type Duration = web_time::Duration;
    pub type SystemTime = web_time::SystemTime;
    pub type Instant = web_time::Instant;
}

#[cfg(not(target_family = "wasm"))]
pub mod target_types {
    pub type KeyCode = crossterm::event::KeyCode;
    pub type KeyEvent = crossterm::event::KeyEvent;
    pub type Duration = std::time::Duration;
    pub type SystemTime = std::time::SystemTime;
    pub type Instant = std::time::Instant;
}

/// The main entry point for the terminal application.
#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use crate::terminal::app::App;

    color_eyre::install()?;

    let mut app = App::new();

    app.run().await?;
    if let Err(err) = crate::terminal::tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {err}"
        );
    }
    Ok(())
}

/// The main entry point for the WASM application.
#[cfg(target_family = "wasm")]
fn main() -> std::io::Result<()> {
    use std::{cell::RefCell, rc::Rc};

    use crate::wasm::app::App;

    let app = App::new();

    App::run(Rc::new(RefCell::new(app)))
}
