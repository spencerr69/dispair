//! This module defines the main application structure for the WASM version of the game.
//! It handles the main game loop, event handling, and rendering of the different views.

use std::{cell::RefCell, io, rc::Rc};

use crate::prelude::Instant;
use serde::de::Error;

use ratzilla::{
    DomBackend, WebRenderer,
    event::{KeyCode, KeyEvent},
};

use web_sys::wasm_bindgen::JsValue;

use crate::common::{Goto, TICK_RATE};

use ratzilla::ratatui::{
    Frame, Terminal,
    layout::{Constraint, Layout},
    style::Style,
    symbols::border,
    text::Text,
    widgets::{Block, List, ListItem, ListState},
};

use crate::common::game::Game;
use crate::common::sound::SoundWrangler;
use crate::common::upgrades::upgrade::PlayerState;
use crate::common::utils::{center_horizontal, center_vertical};

/// Saves the player's progress to local storage.
///
/// # Errors
///
/// Errors if cannot save to localstorage
pub fn save_progress(player_state: &PlayerState) -> Result<(), JsValue> {
    let window = web_sys::window();

    let mut out = Ok(());

    let value: String = serde_json::to_string(player_state)
        .map_err(|_| JsValue::from_str("Failed to serialize player state"))?;

    if let Some(window) = window {
        let local_storage = window
            .local_storage()
            .map_err(|_| JsValue::from_str("Failed to access local storage"))?;

        if let Some(storage) = local_storage {
            out = storage
                .set_item("player_state", &value)
                .map_err(|_| JsValue::from_str("Failed to save player state"));
        }
    }

    out
}

/// Loads the player's progress from local storage.
///
/// # Errors
///
/// Errors if cannot save to localstorage
pub fn load_progress() -> Result<PlayerState, serde_json::Error> {
    let window = web_sys::window();

    let mut value = String::new();

    if let Some(window) = window {
        let local_storage = window
            .local_storage()
            .map_err(|_| serde_json::Error::custom("oops!"))?;

        if let Some(storage) = local_storage {
            let out = storage
                .get_item("player_state")
                .map_err(|_| serde_json::Error::custom("local storage no exist"))?;
            value = out.unwrap_or(String::new());
        }
    }

    let i: PlayerState = serde_json::from_str(&value)?;

    Ok(i)
}

/// The main application struct, which manages the game's state and views.
pub struct App {
    game: Option<Game>,
    player_state: Option<PlayerState>,
    current_selection: ListState,
    sound_wrangler: Option<Rc<RefCell<SoundWrangler>>>,
    last_frame: Instant,
    pub tick_rate: f64,
    save_exists: bool,
}

impl App {
    /// Creates a new `App` instance.
    #[must_use]
    pub fn new() -> Self {
        let mut out = Self {
            game: None,
            player_state: None,
            current_selection: ListState::default(),
            sound_wrangler: None,
            last_frame: Instant::now(),
            tick_rate: TICK_RATE,
            save_exists: load_progress().is_ok(),
        };
        web_sys::console::log_1(&"Hello WASM!".into());

        out.current_selection.select_first();

        out
    }

    /// Runs the main application loop.
    ///
    /// # Errors
    ///
    /// Errors if any issues getting ratzilla/ratatui backend
    pub fn run(this: &Rc<RefCell<Self>>) -> io::Result<()> {
        let backend = DomBackend::new()?;
        let terminal = Terminal::new(backend)?;

        let tick_delay = std::time::Duration::from_secs_f64(1.0 / this.borrow().tick_rate);

        let self_ref_key = this.clone();
        let self_ref_draw = self_ref_key.clone();

        terminal.on_key_event({
            move |key_event| {
                let maybe_reference = self_ref_key.try_borrow_mut();

                if let Ok(mut reference) = maybe_reference {
                    reference.handle_key_event(&key_event);
                }
            }
        });

        terminal.draw_web(move |f| {
            let maybe_reference = self_ref_draw.try_borrow_mut();

            if let Ok(mut reference) = maybe_reference {
                let last_frame = reference.last_frame;

                if Instant::now().duration_since(last_frame) >= tick_delay {
                    reference.on_tick();
                    reference.last_frame = Instant::now();
                }

                reference.on_frame();
                reference.ui(f);
            }
        });

        Ok(())
    }

    /// Handles key events.
    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if self.sound_wrangler.is_none() {
            self.sound_wrangler = Some(Rc::new(RefCell::new(SoundWrangler::default())));
        }
        if let Some(game) = &mut self.game {
            game.handle_key_event(key_event);
        } else {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Down => self.select_next(),
                KeyCode::Char('w') | KeyCode::Up => self.select_prev(),
                KeyCode::Enter => self.confirm_curr(),
                _ => {}
            }
        }
    }

    /// Selects the next item in the menu.
    fn select_next(&mut self) {
        self.current_selection.select_next();
        if !self.save_exists && self.current_selection.selected().unwrap_or(0) == 1 {
            self.select_prev();
        }
    }

    /// Selects the previous item in the menu.
    fn select_prev(&mut self) {
        self.current_selection.select_previous();
    }

    /// Confirms the current selection in the menu.
    fn confirm_curr(&mut self) {
        match self.current_selection.selected() {
            Some(0) => {
                self.player_state = Some(PlayerState::default());
                self.game = Some(Game::new(
                    self.player_state.clone().unwrap(),
                    self.sound_wrangler.clone().unwrap_or_default(),
                ));
            }
            Some(1) => {
                self.player_state = Some(load_progress().unwrap_or_default());
                self.game = Some(Game::new(
                    self.player_state.clone().unwrap(),
                    self.sound_wrangler.clone().unwrap_or_default(),
                ));
            }
            _ => {}
        }
    }

    /// Renders the UI for the current view.
    fn ui(&mut self, frame: &mut Frame) {
        if let Some(ref mut game) = self.game {
            game.render(frame);
        } else {
            self.render_menu(frame);
        }
    }

    /// Called on each game tick.
    fn on_tick(&mut self) {
        if let Some(game) = &mut self.game {
            game.on_tick();
            if game.get_goto().clone() == Goto::Menu {
                self.player_state = Some(game.get_player_state());
                save_progress(self.player_state.as_ref().unwrap()).unwrap();
                self.save_exists = load_progress().is_ok();
                self.game = None;
            }
        }
    }

    /// Called on each frame.
    fn on_frame(&mut self) {
        if let Some(game) = &mut self.game {
            game.on_frame();
        }
    }

    /// Renders the main menu.
    pub fn render_menu(&mut self, frame: &mut Frame) {
        let block = Block::bordered().border_set(border::DOUBLE);

        let [top, bottom] = Layout::vertical([Constraint::Percentage(25), Constraint::Fill(1)])
            .areas(block.inner(frame.area()));

        let title_area = center_vertical(top, 1);

        let title = Text::from("Dispair").centered();

        let options_area = center_vertical(center_horizontal(bottom, 12), 3);

        let options = List::new(vec![
            ListItem::from("New Game"),
            ListItem::from("Continue").style(if !self.save_exists {
                Style::default().dark_gray()
            } else {
                Style::default()
            }),
        ])
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold());

        frame.render_widget(block, frame.area());

        frame.render_widget(title, title_area);
        frame.render_stateful_widget(options, options_area, &mut self.current_selection);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
