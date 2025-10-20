use ratatui::{
    crossterm::style::StyledContent,
    prelude::Stylize,
    text::{Line, Span, Text, ToSpan},
};

use rand::prelude::*;

pub struct GameMap {
    map_contents: Vec<Vec<EntityCharacters>>,
}

impl GameMap {
    pub fn new(width: i16, height: i16) -> Self {
        let mut entities: Vec<Vec<EntityCharacters>> = Vec::from(Vec::new());

        for _ in 0..height {
            let mut line = Vec::new();
            for _ in 0..width {
                line.push(EntityCharacters::Background)
            }
            entities.push(line)
        }

        GameMap {
            map_contents: entities,
        }
    }

    pub fn to_style(&self) -> Vec<Vec<Span<'_>>> {
        self.map_contents
            .iter()
            .map(|line| line.iter().map(|entity| entity.to_styled()).collect())
            .collect()
    }

    pub fn to_text(&self) -> Text<'_> {
        let map = self.to_style();

        let out: Text<'_> = map
            .into_iter()
            .map(|style_line| Line::default().spans(style_line))
            .collect();

        out
    }
}

pub enum EntityCharacters {
    Background,
    Character,
    Enemy1,
    Orb,
}

impl EntityCharacters {
    pub fn to_styled(&self) -> Span<'_> {
        let mut rng = rand::rng();

        match self {
            EntityCharacters::Background => {
                let choice = rng.random_range(0..=1);
                match choice {
                    0 => Span::from(".").dark_gray(),
                    _ => Span::from(",").dark_gray(),
                }
            }
            EntityCharacters::Character => Span::from("0").white(),
            EntityCharacters::Enemy1 => Span::from("x").white(),
            EntityCharacters::Orb => Span::from("o".magenta().rapid_blink()),
        }
    }
}
