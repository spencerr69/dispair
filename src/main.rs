use std::{
    error::Error,
    fs::{File, OpenOptions},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    widgets::{Block, Paragraph, Widget},
};
use serde::de::Error as serdeError;

use crate::{
    roguegame::RogueGame,
    tui::{Event, Tui},
    upgrade::PlayerState,
    upgrademenu::UpgradesMenu,
};

mod character;
mod coords;
mod effects;
mod enemy;
mod roguegame;
mod timescaler;
mod tui;
mod upgrade;
mod upgrademenu;
mod weapon;

pub struct App {
    game_view: Option<RogueGame>,
    upgrades_view: Option<UpgradesMenu>,
    exit: bool,
    player_state: PlayerState,
    pub frame_rate: f64,
    pub tick_rate: f64,
}

pub fn save_progress(player_state: &PlayerState) -> Result<(), serde_json::Error> {
    let path = dirs::config_dir()
        .unwrap()
        .join("dispair")
        .join("player_state.json");

    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| serde_json::Error::custom(e.to_string()))?;

    let save_file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|e| serde_json::Error::custom(e.to_string()))?;

    serde_json::to_writer(save_file, player_state)?;

    Ok(())
}

pub fn load_progress() -> Result<PlayerState, serde_json::Error> {
    let path = dirs::config_dir()
        .unwrap()
        .join("dispair")
        .join("player_state.json");

    let save_file = File::open(path).map_err(|e| serde_json::Error::custom(e.to_string()))?;

    let i: PlayerState = serde_json::from_reader(save_file)?;

    Ok(i)
}

pub const TICK_RATE: f64 = 30.0;
pub const FRAME_RATE: f64 = 60.0;

impl App {
    pub fn new() -> Self {
        let player_state: PlayerState;
        if let Ok(inner_player_state) = load_progress() {
            player_state = inner_player_state;
        } else {
            player_state = PlayerState::default();
        }

        Self {
            game_view: None,
            upgrades_view: None,
            exit: false,
            player_state: player_state,
            frame_rate: FRAME_RATE,
            tick_rate: TICK_RATE,
        }
    }

    async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = Tui::new()?
            .frame_rate(self.frame_rate)
            .tick_rate(self.tick_rate);

        tui.enter()?;

        loop {
            tui.draw(|f| self.ui(f))?;

            if let Some(event) = tui.next().await {
                self.handle_event(event);
            }

            if self.exit {
                save_progress(&self.player_state)?;
                break;
            }
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Tick => {
                self.on_tick();
            }
            Event::Render => {
                self.on_frame();
            }
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        if !key_event.is_press() {
            return;
        }
        if let Some(game) = &mut self.game_view {
            game.handle_key_event(key_event);
        } else if let Some(upgrades_menu) = &mut self.upgrades_view {
            upgrades_menu.handle_key_event(key_event);
        } else {
            match key_event.code {
                KeyCode::Enter => self.start_game(),
                KeyCode::Tab => self.start_upgrades(),
                KeyCode::Esc => self.exit = true,
                _ => {}
            }
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        frame.render_widget(&*self, frame.area());
        if let Some(game) = &self.game_view {
            frame.render_widget(game, frame.area());
        }
        if let Some(ref mut upgrades_menu) = self.upgrades_view {
            upgrades_menu.render_upgrades(frame);
        }
    }

    fn on_tick(&mut self) {
        if let Some(game) = &mut self.game_view {
            game.on_tick();
            if game.game_over {
                self.player_state = game.player_state.clone();
                self.player_state.refresh();
                self.game_view = None;
            }
        }

        if let Some(upgrades_menu) = &mut self.upgrades_view {
            if upgrades_menu.close {
                self.player_state = upgrades_menu.player_state.clone();
                self.player_state.refresh();
                self.upgrades_view = None
            }
        }
    }

    fn on_frame(&mut self) {
        if let Some(game) = &mut self.game_view {
            game.on_frame();
        }
    }

    fn start_game(&mut self) {
        self.game_view = Some(RogueGame::new(self.player_state.clone()));
    }

    fn start_upgrades(&mut self) {
        self.upgrades_view = Some(UpgradesMenu::new(self.player_state.clone()));
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Paragraph::new("")
            .block(Block::bordered().title("dispair"))
            .render(area, buf);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let mut app = App::new();

    let result = app.run().await?;
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {err}"
        );
    }
    Ok(result)
}
