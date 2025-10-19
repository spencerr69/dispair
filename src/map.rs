use ratatui::{
    crossterm::style::StyledContent,
    prelude::Stylize,
    text::{Line, Span, Text, ToSpan},
};

pub struct Map {
    entities: Vec<Vec<EntityCharacters>>,
}

impl Map {
    pub fn new(width: i16, height: i16) -> Self {
        let mut entities: Vec<Vec<EntityCharacters>> = Vec::from(Vec::new());

        for _ in 0..height {
            let mut line = Vec::new();
            for _ in 0..width {
                line.push(EntityCharacters::Background)
            }
            entities.push(line)
        }

        Map { entities }
    }

    pub fn to_style(&self) -> Vec<Vec<Span<'_>>> {
        self.entities
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
    pub fn to_styled(&self) -> Span {
        match self {
            EntityCharacters::Background => Span::from(".").dark_gray(),
            EntityCharacters::Character => Span::from("0").white(),
            EntityCharacters::Enemy1 => Span::from("x").white(),
            EntityCharacters::Orb => Span::from("o".magenta().rapid_blink()),
        }
    }
}
