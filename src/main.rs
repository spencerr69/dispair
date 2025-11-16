pub mod common;

#[cfg(not(target_family = "wasm"))]
pub mod terminal;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[cfg(target_family = "wasm")]
pub type KeyCode = ratzilla::event::KeyCode;

#[cfg(not(target_family = "wasm"))]
pub type KeyCode = crossterm::event::KeyCode;

#[cfg(target_family = "wasm")]
pub type KeyEvent = ratzilla::event::KeyEvent;

#[cfg(not(target_family = "wasm"))]
pub type KeyEvent = crossterm::event::KeyEvent;

#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use crate::terminal::app::App;

    color_eyre::install()?;

    let mut app = App::new();

    let result = app.run().await?;
    if let Err(err) = crate::terminal::tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {err}"
        );
    }
    Ok(result)
}

#[cfg(target_family = "wasm")]
fn main() -> std::io::Result<()> {
    use std::{cell::RefCell, rc::Rc};

    use crate::wasm::app::App;

    let app = App::new();

    App::run(Rc::new(RefCell::new(app)))
}
