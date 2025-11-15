use std::error::Error;

pub mod common;
pub mod terminal;
pub mod wasm;

#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
