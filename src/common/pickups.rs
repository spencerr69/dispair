//! This module defines the `Pickupable` trait and specific pickup items like `PowerupOrb`.
//! Pickups are items that can be collected by the player to gain benefits.

use ratatui::style::{Color, Style};

use crate::common::{coords::Position, roguegame::EntityCharacters};

/// A trait for entities that can be picked up by the player.
pub trait Pickupable {
    /// Gets the position of the pickup.
    fn get_pos(&self) -> &Position;

    /// Gets the visual representation of the pickup.
    fn get_entity_char(&self) -> &EntityCharacters;

    /// Animates the pickup based on the current game tick.
    fn animate(&mut self, tick: u64);

    /// sets picked_up to true and returns pickupeffect
    fn on_pickup(&mut self) -> PickupEffect;

    fn is_picked_up(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum PickupEffect {
    PowerupOrb,
}

/// Represents a power-up orb that the player can collect.
pub struct PowerupOrb {
    /// The visual character of the orb.
    pub entity_char: EntityCharacters,
    /// The position of the orb in the game world.
    pub position: Position,

    pub pickup_effect: PickupEffect,

    pub picked_up: bool,
}

impl PowerupOrb {
    /// Creates a new `PowerupOrb` at the specified position.
    pub fn new(position: Position) -> Self {
        PowerupOrb {
            entity_char: EntityCharacters::Orb(Style::new()),
            position,
            pickup_effect: PickupEffect::PowerupOrb,
            picked_up: false,
        }
    }
}

impl Pickupable for PowerupOrb {
    fn is_picked_up(&self) -> bool {
        self.picked_up
    }

    fn get_pos(&self) -> &Position {
        &self.position
    }

    fn get_entity_char(&self) -> &EntityCharacters {
        &self.entity_char
    }

    /// Animates the orb by cycling through colors every 5 ticks.
    fn animate(&mut self, tick: u64) {
        if !tick.is_multiple_of(5) {
        } else if let EntityCharacters::Orb(style) = &mut self.entity_char {
            *style = match style.fg {
                None => style.fg(Color::LightRed),
                Some(colour) => match colour {
                    Color::LightRed => style.fg(Color::LightYellow),
                    Color::LightYellow => style.fg(Color::LightGreen),
                    Color::LightGreen => style.fg(Color::LightBlue),
                    Color::LightBlue => style.fg(Color::LightMagenta),
                    Color::LightMagenta => style.fg(Color::LightCyan),
                    Color::LightCyan => style.fg(Color::LightRed),
                    _ => style.fg(Color::LightRed),
                },
            };
        }
    }

    fn on_pickup(&mut self) -> PickupEffect {
        self.picked_up = true;
        self.pickup_effect.clone()
    }
}
