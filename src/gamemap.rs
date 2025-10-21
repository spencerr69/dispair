use ratatui::{
    prelude::Stylize,
    text::{Line, Span, Text},
};
use std::cell::RefCell;
use std::rc::Rc;

use rand::prelude::*;

use crate::character::{Character, Position};

pub struct GameMap {
    layer_base: Vec<Vec<EntityCharacters>>,
    layer_entities: Vec<Vec<EntityCharacters>>,
    layer_effects: Vec<Vec<EntityCharacters>>,
    character: Option<Rc<RefCell<Character>>>,
    prev_char_pos: Position,
    height: usize,
    width: usize,
}

impl GameMap {
    pub fn new(width: usize, height: usize) -> Self {
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

        GameMap {
            layer_base: base,
            layer_entities: entities,
            layer_effects: effects,
            character: None,
            prev_char_pos: Position(3, 2),
            height,
            width,
        }
    }

    pub fn set_character(&mut self, character: Rc<RefCell<Character>>) {
        self.character = Some(character);
        self.init_character();
    }

    pub fn init_character(&mut self) {
        let mut rng = rand::rng();

        let (x, y) = (
            rng.random_range(0..self.width) as i16,
            rng.random_range(0..self.height) as i16,
        );

        let mut character_ref = self.character.as_mut().unwrap().borrow_mut();

        character_ref.set_pos(Position(x, y));
    }

    pub fn update_character_position(&mut self) {
        let (old_x, old_y) = self.prev_char_pos.get_as_usize();
        let new_pos = self.character.as_ref().unwrap().borrow().get_pos().clone();
        let (new_x, new_y) = new_pos.get_as_usize();
        self.layer_entities[old_y][old_x] = EntityCharacters::Empty;
        self.layer_entities[new_y][new_x] = EntityCharacters::Character;

        self.prev_char_pos = new_pos.to_owned();
    }

    // pub fn set_entity_position(&mut self, position: &Position) {
    //     todo!()
    // }

    pub fn flatten_to_span(&self) -> Vec<Vec<Span<'static>>> {
        let mut out: Vec<Vec<Span<'static>>> = self
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

    pub fn to_text(&self) -> Text<'static> {
        let map = self.flatten_to_span();

        let out: Text<'static> = map
            .into_iter()
            .map(|style_line| Line::default().spans(style_line))
            .collect();

        out
    }

    pub fn get_pos(&self, position: &Position) -> &EntityCharacters {
        let (x, y) = position.get_as_usize();
        &self.layer_entities[y][x]
    }

    pub fn can_stand(&self, position: &Position) -> bool {
        let (x, y) = position.get();
        if x < 0 || x >= self.width as i16 || y < 0 || y >= self.height as i16 {
            return false;
        }
        self.get_pos(position) == &EntityCharacters::Empty
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
    pub fn to_styled(&self) -> Span<'static> {
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
