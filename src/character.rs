use crate::gamemap::GameMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

#[derive(Clone)]
pub struct Position(pub i16, pub i16);

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        let new_x: i16;
        let new_y: i16;
        if x < 0 {
            new_x = 0;
        } else {
            new_x = x;
        }
        if y < 0 {
            new_y = 0;
        } else {
            new_y = y;
        }

        Position(new_x, new_y)
    }

    pub fn get(&self) -> (i16, i16) {
        (self.0, self.1)
    }

    pub fn get_as_usize(&self) -> (usize, usize) {
        (self.0 as usize, self.1 as usize)
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
    movement_speed: f32,
    last_moved: SystemTime,
}

impl Character {
    pub fn new() -> Self {
        Character {
            position: Position(0, 0),
            map: None,
            movement_speed: 1.,
            last_moved: SystemTime::now(),
        }
    }

    pub fn set_map(&mut self, map: Rc<RefCell<GameMap>>) {
        self.map = Some(map);
    }

    pub fn set_pos(&mut self, new_pos: Position) {
        self.position = new_pos;
    }

    pub fn move_direction(&mut self, direction: Direction) {
        let (x, y) = self.position.get();
        let new_pos = match direction {
            Direction::LEFT => Position::new(x - 1, y),
            Direction::RIGHT => Position::new(x + 1, y),
            Direction::UP => Position::new(x, y - 1),
            Direction::DOWN => Position::new(x, y + 1),
        };

        let attempt_time = SystemTime::now();
        let difference = attempt_time
            .duration_since(self.last_moved)
            .unwrap()
            .as_millis();
        // this is what movement speed controls vv
        let timeout = 100 / self.movement_speed as u128;

        if let Some(map) = &self.map {
            if map.borrow().can_stand(&new_pos) && difference > timeout {
                self.position = new_pos;

                map.borrow_mut().update_character_position();

                self.last_moved = attempt_time;
            }
        }
    }

    pub fn get_pos(&self) -> &Position {
        &self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_above_0() {
        let result = Position::new(4, 4);
        assert_eq!(result.get(), (4, 4));
    }

    #[test]
    fn position_of_0() {
        let result = Position::new(0, 0);
        assert_eq!(result.get(), (0, 0));
    }

    #[test]
    fn position_below_0() {
        let result = Position::new(-1, -1);
        assert_eq!(result.get(), (0, 0));
    }
}
