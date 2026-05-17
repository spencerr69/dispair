use ratatui::prelude::{Span, Style};
use ratatui::style::Stylize;

#[derive(PartialEq, Eq, Clone)]
pub enum EntityCharacters {
    Background1,
    Background2,
    Character(Style),
    Enemy(Style),
    Empty,
    AttackBlackout(Style),
    AttackMist(Style),
    AttackWeak(Style),
    Orb(Style),
}

impl EntityCharacters {
    #[must_use]
    pub fn to_styled(&self) -> Span<'static> {
        match self {
            EntityCharacters::Background1 => Span::from(".").dark_gray(),
            EntityCharacters::Background2 => Span::from(",").dark_gray(),
            EntityCharacters::Character(style) => Span::from("0").white().bold().style(*style),
            EntityCharacters::Enemy(style) => Span::from("x").white().style(*style),
            EntityCharacters::Empty => Span::from(" "),
            EntityCharacters::AttackBlackout(style) => {
                Span::from(ratatui::symbols::block::FULL).style(*style)
            }
            EntityCharacters::AttackMist(style) => {
                Span::from(ratatui::symbols::shade::MEDIUM).style(*style)
            }
            EntityCharacters::AttackWeak(style) => {
                Span::from(ratatui::symbols::shade::LIGHT).style(*style)
            }
            EntityCharacters::Orb(style) => Span::from("o").style(*style),
        }
    }

    pub fn replace(&mut self, new_entity: EntityCharacters) {
        *self = new_entity;
    }

    /// Get a mutable reference to the inner style if it exists.
    ///
    /// # Panics
    ///
    /// If trying to call `style_mut` on an `EntityCharacters` which does not have an inner style, it will panic.
    pub fn style_mut(&mut self) -> &mut Style {
        match self {
            EntityCharacters::Character(style)
            | EntityCharacters::Enemy(style)
            | EntityCharacters::Orb(style)
            | EntityCharacters::AttackBlackout(style)
            | EntityCharacters::AttackMist(style)
            | EntityCharacters::AttackWeak(style) => style,
            _ => panic!("Cannot get style_mut for a non-styled entity"),
        }
    }

    /// Checks if the entity is a player character.
    #[must_use]
    pub fn is_char(&self) -> bool {
        matches!(self, EntityCharacters::Character(_))
    }
}
