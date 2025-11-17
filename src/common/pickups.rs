use ratatui::style::{Color, Style};

use crate::common::{coords::Position, roguegame::EntityCharacters};

pub trait Pickupable {
    fn get_pos(&self) -> &Position;

    fn get_entity_char(&self) -> &EntityCharacters;

    fn animate(&mut self, tick: u64);
}

pub struct PowerupOrb {
    pub entity_char: EntityCharacters,
    pub position: Position,
}

impl PowerupOrb {
    pub fn new(position: Position) -> Self {
        PowerupOrb {
            entity_char: EntityCharacters::Orb(Style::new()),
            position,
        }
    }
}

impl Pickupable for PowerupOrb {
    fn get_pos(&self) -> &Position {
        &self.position
    }

    fn get_entity_char(&self) -> &EntityCharacters {
        &self.entity_char
    }

    fn animate(&mut self, tick: u64) {
        if !(tick % 5 == 0) {
            return;
        } else {
            if let EntityCharacters::Orb(style) = &mut self.entity_char {
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
    }
}
