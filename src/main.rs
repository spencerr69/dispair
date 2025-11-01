use std::{
    error::Error,
    fs::{File, OpenOptions},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Text,
    widgets::{Block, List, ListItem, ListState},
};
use serde::de::Error as serdeError;

use crate::{
    roguegame::RogueGame,
    tui::{Event, Tui},
    upgrade::PlayerState,
    upgrademenu::{Goto, UpgradesMenu},
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

pub struct App {
    game_view: Option<RogueGame>,
    upgrades_view: Option<UpgradesMenu>,
    exit: bool,
    player_state: Option<PlayerState>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    current_selection: ListState,
}

pub const TICK_RATE: f64 = 30.0;
pub const FRAME_RATE: f64 = 180.0;

impl App {
    pub fn new() -> Self {
        let mut out = Self {
            game_view: None,
            upgrades_view: None,
            exit: false,
            player_state: None,
            frame_rate: FRAME_RATE,
            tick_rate: TICK_RATE,
            current_selection: ListState::default(),
        };

        out.current_selection.select_first();

        out
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
        if let Some(game) = &self.game_view {
            frame.render_widget(game, frame.area());
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
                    Goto::Menu => save_progress(&self.player_state.clone().unwrap()).unwrap_or(()),
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
        frame.render_stateful_widget(options, options_area, &mut self.current_selection);
    }
}

pub fn center_vertical(area: Rect, height: u16) -> Rect {
    let [centered_area] = Layout::vertical([Constraint::Length(height)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

pub fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [centered_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
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
