use ratatui::{
    crossterm::style::StyledContent,
    prelude::Stylize,
    text::{Line, Span, Text, ToSpan},
};

use rand::prelude::*;

pub struct GameMap {
    layer_base: Vec<Vec<EntityCharacters>>,
    layer_entities: Vec<Vec<EntityCharacters>>,
    layer_effects: Vec<Vec<EntityCharacters>>,
}

impl GameMap {
    pub fn new(width: i16, height: i16) -> Self {
        let mut base: Vec<Vec<EntityCharacters>> = Vec::from(Vec::new());
        let mut entities: Vec<Vec<EntityCharacters>> = Vec::from(Vec::new());
        let mut effects: Vec<Vec<EntityCharacters>> = Vec::from(Vec::new());

        let mut rng = rand::rng();

        for _ in 0..height {
            let mut baseline = Vec::new();
            let mut entityline = Vec::new();
            let mut effectsline = Vec::new();
            for _ in 0..width {
                let choice = rng.random_range(0..=1);
                match choice {
                    0 => baseline.push(EntityCharacters::Background1),
                    _ => baseline.push(EntityCharacters::Background2),
                }
                entityline.push(EntityCharacters::Empty);
                effectsline.push(EntityCharacters::Empty);
            }
            base.push(baseline);
            entities.push(entityline);
            effects.push(effectsline);
        }

        entities[3][2] = EntityCharacters::Character;

        GameMap {
            layer_base: base,
            layer_entities: entities,
            layer_effects: effects,
        }
    }

    pub fn flatten_to_span(&self) -> Vec<Vec<Span<'_>>> {
        let mut out: Vec<Vec<Span<'_>>> = self
            .layer_base
            .iter()
            .map(|line| line.iter().map(|entity| entity.to_styled()).collect())
            .collect();

        for (y, row) in out.iter_mut().enumerate() {
            for (x, pos) in row.iter_mut().enumerate() {
                if self.layer_effects[y][x] != EntityCharacters::Empty
                    || self.layer_entities[y][x] != EntityCharacters::Empty
                {
                    if self.layer_effects[y][x] != EntityCharacters::Empty {
                        pos.clone_from(&self.layer_effects[y][x].to_styled());
                    } else {
                        pos.clone_from(&self.layer_entities[y][x].to_styled());
                    }
                }
            }
        }

        out
    }

    pub fn to_text(&self) -> Text<'_> {
        let map = self.flatten_to_span();

        let out: Text<'_> = map
            .into_iter()
            .map(|style_line| Line::default().spans(style_line))
            .collect();

        out
    }
}

#[derive(PartialEq, Eq)]
pub enum EntityCharacters {
    Background1,
    Background2,
    Character,
    Enemy1,
    Orb,
    Empty,
}

impl EntityCharacters {
    pub fn to_styled(&self) -> Span<'_> {
        match self {
            EntityCharacters::Background1 => Span::from(".").dark_gray(),
            EntityCharacters::Background2 => Span::from(",").dark_gray(),
            EntityCharacters::Character => Span::from("0").white().bold(),
            EntityCharacters::Enemy1 => Span::from("x").white(),
            EntityCharacters::Orb => Span::from("o".magenta().rapid_blink()),
            EntityCharacters::Empty => Span::from(" "),
        }
    }
}
