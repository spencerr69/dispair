use crate::character::*;
use crate::roguegame::*;

pub trait EnemyBehaviour {
    fn new(position: Position, damage: i32) -> Self;

    fn update(&mut self, character: &mut Character, layer: &Layer);
}

#[derive(Clone)]
pub struct Enemy {
    position: Position,
    prev_position: Position,

    facing: Direction,

    damage: i32,

    health: i32,
    is_alive: bool,
}

impl EnemyBehaviour for Enemy {
    fn new(position: Position, damage: i32) -> Self {
        Enemy {
            position: position.clone(),
            prev_position: position,

            facing: Direction::UP,

            damage,

            health: 5,
            is_alive: true,
        }
    }

    fn update(&mut self, character: &mut Character, layer: &Layer) {
        if is_next_to_character(layer, &self.position) {
            character.take_damage(self.damage);
        }

        let (dist_x, dist_y) = self.position.get_distance(character.get_pos());
        let (x, y) = self.position.get();
        let desired_pos: Position;
        let desired_facing: Direction;

        if dist_x.abs() > dist_y.abs() {
            if dist_x > 0 {
                desired_pos = Position::new(x + 1, y);
                desired_facing = Direction::RIGHT;
            } else {
                desired_pos = Position::new(x - 1, y);
                desired_facing = Direction::LEFT;
            }
        } else {
            if dist_y > 0 {
                desired_pos = Position::new(x, y + 1);
                desired_facing = Direction::DOWN;
            } else {
                desired_pos = Position::new(x, y - 1);
                desired_facing = Direction::UP;
            }
        }

        if can_stand(layer, &desired_pos) {
            self.move_to(desired_pos, desired_facing);
        }
    }
}

impl Movable for Enemy {
    const ENTITY_CHAR: EntityCharacters = EntityCharacters::Enemy1;

    fn get_pos(&self) -> &Position {
        &self.position
    }

    fn get_prev_pos(&self) -> &Position {
        &self.prev_position
    }

    fn move_to(&mut self, new_pos: Position, facing: Direction) {
        self.facing = facing;
        self.set_pos(new_pos);
    }

    fn set_pos(&mut self, new_pos: Position) {
        self.prev_position = self.position.clone();
        self.position = new_pos;
    }
}

impl Damageable for Enemy {
    fn die(&mut self) {
        self.is_alive = false;
    }

    fn get_health(&self) -> &i32 {
        &self.health
    }

    fn is_alive(&self) -> bool {
        self.is_alive.clone()
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.health <= 0 {
            self.die();
        }
    }
}
