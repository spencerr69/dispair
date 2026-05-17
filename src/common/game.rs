use crate::common;
use crate::common::roguegame::RogueGame;
use crate::common::upgrades::upgrade::PlayerState;
use crate::common::upgrades::upgrademenu::UpgradesMenu;
use crate::common::{Goto, Viewable};
use crate::prelude::{KeyEvent, save_progress};
use ratatui::Frame;
use std::cell::RefCell;
use std::rc::Rc;

pub enum View {
    Game(RogueGame),
    Upgrades(UpgradesMenu),
}

impl View {
    pub fn get_view_mut(&mut self) -> &mut dyn Viewable {
        match self {
            View::Game(rogue_game) => rogue_game,
            View::Upgrades(upgrades_menu) => upgrades_menu,
        }
    }
    pub fn get_view_ref(&self) -> &dyn Viewable {
        match self {
            View::Game(rogue_game) => rogue_game,
            View::Upgrades(upgrades_menu) => upgrades_menu,
        }
    }

    pub fn get_goto(&self) -> &Goto {
        self.get_view_ref().get_goto()
    }
}

pub struct Game {
    view: View,

    //TODO: make views work for any view, not each individual one
    // game_view: Option<RogueGame>,
    // upgrades_view: Option<UpgradesMenu>,
    pub player_state: PlayerState,
}

impl Game {
    pub fn new(player_state: PlayerState) -> Self {
        Self {
            view: View::Upgrades(UpgradesMenu::new(player_state.clone())),
            player_state,
        }
    }

    pub fn go_to(&mut self, goto: &Goto) {
        match goto {
            Goto::Upgrades => {
                self.view = View::Upgrades(UpgradesMenu::new(self.player_state.clone()));
            }
            Goto::Game => self.view = View::Game(RogueGame::new(&self.player_state)),
            _ => {}
        }
    }

    pub fn is_correct_view(&self) -> bool {
        let goto = self.view.get_goto().clone();

        match self.view {
            View::Upgrades(_) => goto == Goto::Upgrades,
            View::Game(_) => goto == Goto::Game,
        }
    }

    pub fn get_goto(&self) -> &Goto {
        self.view.get_goto()
    }

    pub fn get_player_state(&self) -> PlayerState {
        self.player_state.clone()
    }

    pub fn on_tick(&mut self) {
        let goto = self.view.get_goto().clone();

        if self.is_correct_view() {
            self.view.get_view_mut().tick();
        } else {
            self.player_state.refresh();
            save_progress(&self.player_state).expect("");
            self.go_to(&goto);
        }
    }

    pub fn on_frame(&mut self) {
        self.view.get_view_mut().frame();
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if !key_event.is_press() {
            return;
        }
        self.view.get_view_mut().handle_key_event(key_event);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.view.get_view_mut().render(frame);
    }
}
