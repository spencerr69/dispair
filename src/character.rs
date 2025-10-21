use std::time::SystemTime;

use crate::roguegame::EntityCharacters;

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
    prev_position: Position,
    movement_speed: f32,
    last_moved: SystemTime,
}

pub trait Movable {
    fn set_pos(&mut self, new_pos: Position);
    fn get_pos(&self) -> &Position;
    fn move_to(&mut self, new_pos: Position);
    fn get_prev_pos(&self) -> &Position;
    fn get_entity_char(&self) -> EntityCharacters {
        Self::ENTITY_CHAR
    }
    const ENTITY_CHAR: EntityCharacters;
}

impl Character {
    pub fn new() -> Self {
        Character {
            prev_position: Position(0, 0),
            position: Position(0, 0),
            movement_speed: 1.,
            last_moved: SystemTime::now(),
        }
    }
}

impl Movable for Character {
    const ENTITY_CHAR: EntityCharacters = EntityCharacters::Character;

    fn set_pos(&mut self, new_pos: Position) {
        self.prev_position = self.position.clone();
        self.position = new_pos;
    }

    fn move_to(&mut self, new_pos: Position) {
        let attempt_time = SystemTime::now();
        let difference = attempt_time
            .duration_since(self.last_moved)
            .unwrap()
            .as_millis();
        // this is what movement speed controls vv
        let timeout = 100 / self.movement_speed as u128;

        if difference > timeout {
            self.set_pos(new_pos);
            self.last_moved = attempt_time;
        }
    }

    fn get_pos(&self) -> &Position {
        &self.position
    }

    fn get_prev_pos(&self) -> &Position {
        &self.prev_position
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
