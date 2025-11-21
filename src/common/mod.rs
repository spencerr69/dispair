//! This module contains common game logic and structures that are shared
//! between the terminal and WASM versions of the application. It includes
//! modules for characters, coordinates, game state, and more.

use ratatui::layout::{Constraint, Layout, Rect};

pub mod character;
pub mod charms;
pub mod coords;
pub mod effects;
pub mod enemy;
pub mod pickups;
pub mod popups;
pub mod powerup;
pub mod roguegame;
pub mod timescaler;
pub mod upgrade;
pub mod upgrademenu;
pub mod weapon;

/// Centers a `Rect` vertically within a given area.
pub fn center_vertical(area: Rect, height: u16) -> Rect {
    let [centered_area] = Layout::vertical([Constraint::Length(height)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

/// Centers a `Rect` horizontally within a given area.
pub fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [centered_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

/// Centers a `Rect` both vertically and horizontally within a given area.
pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let centered_area = center_vertical(area, height);
    center_horizontal(centered_area, width)
}

/// The number of game ticks per second.
pub const TICK_RATE: f64 = 90.0;
/// The target number of frames per second.
pub const FRAME_RATE: f64 = 120.0;
