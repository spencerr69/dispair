use crate::gamemap::GameMap;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Position(pub usize, pub usize);

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        let new_x: usize;
        let new_y: usize;
        if x < 0 {
            new_x = 0;
        } else {
            new_x = x as usize;
        }
        if y < 0 {
            new_y = 0;
        } else {
            new_y = y as usize;
        }

        Position(new_x, new_y)
    }

    pub fn get(&self) -> (usize, usize) {
        (self.0, self.1)
    }

    pub fn set(&mut self, x: usize, y: usize) {
        self.0 = x;
        self.1 = y;
    }
}

pub enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

pub struct Character {
    position: Position,
    map: Option<Rc<RefCell<GameMap>>>,
}

impl Character {
    pub fn new() -> Self {
        Character {
            position: Position(0, 0),
            map: None,
        }
    }

    pub fn set_map(&mut self, map: Rc<RefCell<GameMap>>) {
        self.map = Some(map);
    }

    pub fn move_direction(&mut self, direction: Direction) {
        let (x, y) = self.position.get();
        let new_pos = match direction {
            Direction::LEFT => Position::new(x as i16 - 1 as i16, y as i16),
            Direction::RIGHT => Position::new(x as i16 + 1 as i16, y as i16),
            Direction::UP => Position::new(x as i16, y as i16 - 1 as i16),
            Direction::DOWN => Position::new(x as i16, y as i16 + 1 as i16),
        };

        if let Some(map) = &self.map {
            if map.borrow().can_stand(&new_pos) {
                self.position = new_pos;

                map.borrow_mut()
                    .update_character_position(self.position.get());
            }
        }
    }

    pub fn get_pos(&self) -> (usize, usize) {
        self.position.get()
    }
}
