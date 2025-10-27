use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    roguegame::RogueGame,
    tui::{Event, Tui},
    upgrade::{PlayerState, UpgradesMenu},
};

mod character;
mod effects;
mod enemy;
mod roguegame;
mod tui;
mod upgrade;
mod weapon;

pub struct App {
    game_view: Option<RogueGame>,
    upgrades_view: Option<UpgradesMenu>,
    exit: bool,
    player_state: PlayerState,
    pub frame_rate: f64,
    pub tick_rate: f64,
}

pub const TICK_RATE: f64 = 30.0;

impl App {
    pub fn new() -> Self {
        Self {
            game_view: None,
            upgrades_view: None,
            exit: false,
            player_state: PlayerState::default(),
            frame_rate: 180.0,
            tick_rate: TICK_RATE,
        }
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
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
                self.game_view = None;
            }
        }

        if let Some(upgrades_menu) = &mut self.upgrades_view {
            if upgrades_menu.close {
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
        let width = self.player_state.stats.width;
        let height = self.player_state.stats.height;

        self.game_view = Some(RogueGame::new(width, height));
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
            .block(Block::bordered().title("Game"))
            .render(area, buf);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    let _ = app.run().await?;

    Ok(())
}
