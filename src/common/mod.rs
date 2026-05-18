//! This module contains common game logic and structures that are shared
//! between the terminal and WASM versions of the application. It includes
//! modules for characters, coordinates, game state, and more.

use crate::common::upgrades::upgrade::PlayerState;
use crate::prelude::KeyEvent;
use ratatui::Frame;
use std::cell::RefCell;
use std::rc::Rc;

pub mod character;
pub mod charms;
pub mod coords;
pub mod debuffs;
pub mod effects;
pub mod enemies;
pub mod entities;
pub mod game;
pub mod level;
pub mod pickups;
pub mod popups;
pub mod powerup;
pub mod render;
pub mod rogue;
pub mod stats;
pub mod timescaler;
pub mod upgrades;
pub(crate) mod utils;
pub mod weapons;
pub mod widgets;

/// The number of game ticks per second.
pub const TICK_RATE: f64 = 20.0;
/// The target number of frames per second.
pub const FRAME_RATE: f64 = 180.0;

pub type PlayerStateRef = Rc<RefCell<PlayerState>>;

/// An enum representing the possible destinations when closing the upgrade menu.
#[derive(Clone, PartialEq)]
pub enum Goto {
    Game,
    Menu,
    Upgrades,
}

pub trait Viewable {
    fn tick(&mut self);
    fn frame(&mut self) {}

    fn get_goto(&self) -> &Goto;

    fn render(&mut self, frame: &mut Frame);

    fn handle_key_event(&mut self, key_event: &KeyEvent);
}
