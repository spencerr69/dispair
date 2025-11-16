use std::{cell::RefCell, io, rc::Rc};

use serde::de::Error;
use web_time::Instant;

use ratzilla::{
    DomBackend, WebRenderer,
    event::{KeyCode, KeyEvent},
};

use web_sys::wasm_bindgen::JsValue;

use crate::common::{TICK_RATE, center_horizontal, center_vertical};

use ratzilla::ratatui::{
    Frame, Terminal,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Text,
    widgets::{Block, List, ListItem, ListState},
};

use crate::common::{
    carnagereport::CarnageReport,
    roguegame::RogueGame,
    upgrade::PlayerState,
    upgrademenu::{Goto, UpgradesMenu},
};

pub fn save_progress(player_state: &PlayerState) -> Result<(), JsValue> {
    web_sys::window()
        .expect("Failed to get window")
        .local_storage()?
        .expect("local storage doesn't exist")
        .set_item(
            "player_state",
            &serde_json::to_string(player_state).unwrap(),
        )
}

pub fn load_progress() -> Result<PlayerState, serde_json::Error> {
    let value = web_sys::window()
        .expect("no global `window` exists")
        .local_storage()
        .map_err(|_| serde_json::Error::custom("oops!"))?
        .expect("local storage no exist")
        .get_item("player_state")
        .map_err(|_| serde_json::Error::custom("help"))?
        .unwrap_or("".into());

    let i: PlayerState = serde_json::from_str(&value)?;

    Ok(i)
}

pub struct App {
    game_view: Option<RogueGame>,
    upgrades_view: Option<UpgradesMenu>,
    exit: bool,
    player_state: Option<PlayerState>,
    current_selection: ListState,
    last_frame: Instant,
    pub tick_rate: f64,
}

impl App {
    pub fn new() -> Self {
        let mut out = Self {
            game_view: None,
            upgrades_view: None,
            exit: false,
            player_state: None,
            current_selection: ListState::default(),
            last_frame: Instant::now(),
            tick_rate: TICK_RATE,
        };

        out.current_selection.select_first();

        out
    }

    pub fn run(this: Rc<RefCell<Self>>) -> io::Result<()> {
        let backend = DomBackend::new()?;
        let terminal = Terminal::new(backend)?;

        let tick_delay = std::time::Duration::from_secs_f64(1.0 / this.borrow().tick_rate);

        let self_ref_key = this.clone();
        let self_ref_draw = self_ref_key.clone();

        terminal.on_key_event({
            move |key_event| {
                let maybe_reference = self_ref_key.try_borrow_mut();

                if let Ok(mut reference) = maybe_reference {
                    reference.handle_key_event(key_event);
                }
            }
        });

        terminal.draw_web(move |f| {
            let maybe_reference = self_ref_draw.try_borrow_mut();

            if let Ok(mut reference) = maybe_reference {
                let last_frame = reference.last_frame.clone();

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

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Some(game) = &mut self.game_view {
            game.handle_key_event(key_event);
        } else if let Some(upgrades_menu) = &mut self.upgrades_view {
            upgrades_menu.handle_key_event(key_event);
        } else {
            match key_event.code {
                KeyCode::Char('s') | KeyCode::Down => self.select_next(),
                KeyCode::Char('w') | KeyCode::Up => self.select_prev(),
                KeyCode::Enter => self.confirm_curr(),
                KeyCode::Esc => self.exit = true,
                _ => {}
            }
        }
    }

    fn select_next(&mut self) {
        self.current_selection.select_next();
    }

    fn select_prev(&mut self) {
        self.current_selection.select_previous();
    }

    fn confirm_curr(&mut self) {
        match self.current_selection.selected() {
            Some(0) => {
                self.player_state = Some(load_progress().unwrap_or_default());
                self.start_upgrades();
            }
            Some(1) => {
                self.player_state = Some(PlayerState::default());
                self.start_upgrades();
            }
            Some(2) => self.exit = true,
            _ => {}
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        if let Some(ref mut game) = self.game_view {
            game.render(frame)
        } else if let Some(ref mut upgrades_menu) = self.upgrades_view {
            upgrades_menu.render_upgrades(frame);
        } else {
            self.render_menu(frame);
        }
    }

    fn on_tick(&mut self) {
        if let Some(game) = &mut self.game_view {
            game.on_tick();
            if game.game_over {
                game.carnage_report = Some(CarnageReport::new(
                    self.player_state.clone().unwrap(),
                    game.player_state.clone(),
                ));
            }
            if game.exit {
                self.player_state = Some(game.player_state.clone());
                self.player_state.as_mut().unwrap().refresh();
                self.game_view = None;
                self.start_upgrades();
            }
        }

        if let Some(upgrades_menu) = &mut self.upgrades_view {
            if let Some(close) = upgrades_menu.close.clone() {
                self.player_state = Some(upgrades_menu.player_state.clone());
                self.player_state.as_mut().unwrap().refresh();
                self.upgrades_view = None;
                match close {
                    Goto::Game => self.start_game(),
                    Goto::Menu => {
                        save_progress(&self.player_state.clone().unwrap()).expect("couldn't save")
                    }
                }
            }
        }
    }

    fn on_frame(&mut self) {
        if let Some(game) = &mut self.game_view {
            game.on_frame();
        }
    }

    fn start_game(&mut self) {
        if let Some(player_state) = &self.player_state {
            self.game_view = Some(RogueGame::new(player_state.clone()));
        }
    }

    fn start_upgrades(&mut self) {
        if let Some(player_state) = &self.player_state {
            self.upgrades_view = Some(UpgradesMenu::new(player_state.clone()));
        }
    }

    pub fn render_menu(&mut self, frame: &mut Frame) {
        let block = Block::bordered().border_set(border::DOUBLE);

        let [top, bottom] = Layout::vertical([Constraint::Percentage(25), Constraint::Fill(1)])
            .areas(block.inner(frame.area()));

        let title_area = center_vertical(top, 1);

        let title = Text::from("Dispair").centered();

        let options_area = center_vertical(center_horizontal(bottom, 12), 3);

        let options = List::new(vec![
            ListItem::from("Continue"),
            ListItem::from("New Game"),
            ListItem::from("Quit"),
        ])
        .highlight_symbol("> ")
        .highlight_style(Style::new().bold());

        frame.render_widget(block, frame.area());

        frame.render_widget(title, title_area);
        frame.render_stateful_widget(options, options_area, &mut self.current_selection)
    }
}
