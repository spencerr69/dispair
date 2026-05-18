use ratatui::prelude::{Span, Style};
use ratatui::style::Stylize;

#[derive(PartialEq, Eq, Clone)]
pub enum EntityCharacters {
    Background1(Style),
    Background2(Style),
    Character(Style),
    Enemy(Style),
    Empty(Style),
    AttackBlackout(Style),
    AttackMist(Style),
    AttackWeak(Style),
    Orb(Style),
}

impl EntityCharacters {
    #[must_use]
    pub fn to_styled(&self) -> Span<'static> {
        match self {
            EntityCharacters::Background1(style) => Span::from(".").style(*style),
            EntityCharacters::Background2(style) => Span::from(",").style(*style),
            EntityCharacters::Character(style) => Span::from("0").style(*style),
            EntityCharacters::Enemy(style) => Span::from("x").style(*style),
            EntityCharacters::Empty(style) => Span::from(" ").style(*style),
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
    pub fn style_mut(&mut self) -> &mut Style {
        match self {
            EntityCharacters::Character(style)
            | EntityCharacters::Enemy(style)
            | EntityCharacters::Orb(style)
            | EntityCharacters::AttackBlackout(style)
            | EntityCharacters::AttackMist(style)
            | EntityCharacters::Background1(style)
            | EntityCharacters::Background2(style)
            | EntityCharacters::Empty(style)
            | EntityCharacters::AttackWeak(style) => style,
        }
    }

    /// Checks if the entity is a player character.
    #[must_use]
    pub fn is_char(&self) -> bool {
        matches!(self, EntityCharacters::Character(_))
    }
}
