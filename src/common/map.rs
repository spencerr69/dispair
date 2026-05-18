use crate::common::entities::EntityCharacters;
use rand::Rng;
use ratatui::style::Style;

pub type Layer = Vec<Vec<EntityCharacters>>;

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub map: Layer,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut new = Self {
            height,
            width,
            map: Vec::new(),
        };

        new.fill();

        new
    }

    pub fn fill(&mut self) {
        let mut rng = rand::rng();

        self.map = Vec::new();

        for _ in 0..self.height {
            let mut row = Vec::new();
            for _ in 0..self.width {
                let choice = rng.random_range(0..=1);
                match choice {
                    0 => row.push(EntityCharacters::Background1(Style::new().dark_gray())),
                    1 => row.push(EntityCharacters::Background2(Style::new().dark_gray())),
                    _ => {}
                }
            }
            self.map.push(row);
        }

        self.map.iter_mut().for_each(|row| {
            for entity in row.iter_mut() {
                let entity_style = entity.style_mut();

                let choice = rng.random_range(0..10);

                if choice < 2 {
                    *entity_style = entity_style.black();
                } else if choice < 8 {
                    *entity_style = entity_style.dark_gray();
                }
            }
        });
    }

    pub fn get_layer(&self) -> &Layer {
        &self.map
    }
}
