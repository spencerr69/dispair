use crate::character::{Character, Direction, Movable, Position};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

type Layer = Vec<Vec<EntityCharacters>>;

pub struct RogueGame {
    character: Character,
    layer_base: Layer,
    layer_entities: Layer,
    layer_effects: Layer,
    prev_char_pos: Position,
    height: usize,
    width: usize,
}

impl RogueGame {
    pub fn new(width: usize, height: usize) -> Self {
        let mut base: Layer = Vec::from(Vec::new());
        let mut entities: Layer = Vec::from(Vec::new());
        let mut effects: Layer = Vec::from(Vec::new());

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

        let mut game = RogueGame {
            character: Character::new(),
            layer_base: base,
            layer_entities: entities,
            layer_effects: effects,
            prev_char_pos: Position(3, 2),
            height,
            width,
        };

        game.init_character();

        game
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') => self.move_character(Direction::DOWN),
            KeyCode::Char('w') => self.move_character(Direction::UP),
            KeyCode::Char('d') => self.move_character(Direction::RIGHT),
            KeyCode::Char('a') => self.move_character(Direction::LEFT),
            _ => {}
        }
    }
    pub fn update_character_position(&mut self) {
        let (old_x, old_y) = self.prev_char_pos.get_as_usize();
        let new_pos = self.character.get_pos();
        let (new_x, new_y) = new_pos.get_as_usize();
        self.layer_entities[old_y][old_x] = EntityCharacters::Empty;
        self.layer_entities[new_y][new_x] = EntityCharacters::Character;

        self.prev_char_pos = new_pos.clone();
    }

    pub fn update_entity_positions(layer: &mut Layer, entity: &impl Movable) {
        Self::set_entity(layer, entity.get_prev_pos(), EntityCharacters::Empty);
        Self::set_entity(layer, entity.get_pos(), entity.get_entity_char())
    }

    pub fn set_entity(
        layer: &mut Vec<Vec<EntityCharacters>>,
        position: &Position,
        entity: EntityCharacters,
    ) {
        let (x, y) = position.get_as_usize();
        layer[y][x] = entity;
    }

    pub fn move_character(&mut self, direction: Direction) {
        let (x, y) = self.character.get_pos().get();
        let new_pos = match direction {
            Direction::LEFT => Position::new(x - 1, y),
            Direction::RIGHT => Position::new(x + 1, y),
            Direction::UP => Position::new(x, y - 1),
            Direction::DOWN => Position::new(x, y + 1),
        };

        if self.can_stand(&new_pos) {
            self.character.move_to(new_pos);
            Self::update_entity_positions(self.layer_entities.as_mut(), &mut self.character);
        }
    }

    pub fn init_character(&mut self) {
        let mut rng = rand::rng();

        let (x, y) = (
            rng.random_range(0..self.width) as i16,
            rng.random_range(0..self.height) as i16,
        );

        self.character.set_pos(Position(x, y));
        self.update_character_position();
    }

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

impl Widget for &RogueGame {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" idle game yass ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Paragraph::new(self.to_text())
            .centered()
            .block(block)
            .render(area, buf);
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
