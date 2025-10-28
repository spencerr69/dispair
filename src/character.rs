use ratatui::style::{Style, Stylize};

use crate::{
    effects::DamageEffect,
    roguegame::Layer,
    upgrade::PlayerState,
    weapon::{Area, DamageArea, Sword, Weapon},
};
use std::time::SystemTime;

use crate::roguegame::EntityCharacters;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Position(pub i32, pub i32);

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        let new_x: i32;
        let new_y: i32;
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

    pub fn get(&self) -> (i32, i32) {
        (self.0, self.1)
    }

    pub fn get_as_usize(&self) -> (usize, usize) {
        (self.0.max(0) as usize, self.1.max(0) as usize)
    }

    pub fn constrain(&mut self, layer: &Layer) {
        self.0 = self.0.max(0).min(layer[0].len() as i32 - 1);
        self.1 = self.1.max(0).min(layer.len() as i32 - 1);
    }

    pub fn get_distance(&self, other: &Position) -> (i32, i32) {
        let (self_x, self_y) = self.get();
        let (other_x, other_y) = other.get();
        (other_x - self_x, other_y - self_y)
    }

    pub fn is_in_area(&self, area: &Area) -> bool {
        let (x, y) = self.get();
        let (min_x, min_y, max_x, max_y) = area.get_bounds();
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }
}

#[derive(Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

pub struct Character<'a> {
    position: Position,
    prev_position: Position,
    last_moved: SystemTime,
    pub facing: Direction,

    pub movement_speed: f32,
    pub strength: f32,
    pub attack_speed: f32,

    health: i32,
    max_health: i32,
    is_alive: bool,

    weapons: Vec<Box<dyn Weapon>>,

    pub player_state: &'a PlayerState,

    entitychar: EntityCharacters,
}

///Trait for an entity which can move
pub trait Movable {
    fn set_pos(&mut self, new_pos: Position);
    fn get_pos(&self) -> &Position;
    fn move_to(&mut self, new_pos: Position, facing: Direction);
    fn get_prev_pos(&self) -> &Position;
    fn get_entity_char(&self) -> EntityCharacters;
}

///Trait for an entity which has health and can be damaged
pub trait Damageable {
    fn get_health(&self) -> &i32;

    /// take_damage can also heal if damage is provided as negative
    fn take_damage(&mut self, damage: i32);

    /// Function to be called when entity dies.
    fn die(&mut self);

    /// return if entity is alive
    fn is_alive(&self) -> bool;
}

impl<'a> Character<'a> {
    pub fn new(player_state: &'a PlayerState) -> Self {
        let max_health = 10000;
        Character {
            position: Position(0, 0),
            movement_speed: 1.,
            prev_position: Position(0, 0),
            last_moved: SystemTime::now(),
            facing: Direction::UP,
            strength: 1.,
            attack_speed: 1.,
            player_state,

            health: max_health,
            max_health: max_health,
            is_alive: true,

            entitychar: EntityCharacters::Character(Style::default()),

            weapons: vec![Box::new(Sword::new(player_state))],
            // weapons: vec![],
        }
    }

    pub fn attack(&self, layer_effects: &mut Layer) -> (Vec<DamageArea>, Vec<DamageEffect>) {
        let damage_areas: Vec<DamageArea> = self
            .weapons
            .iter()
            .map(|weapon| weapon.attack(&self))
            .map(|mut damage_area| {
                damage_area.area.constrain(layer_effects);
                damage_area
            })
            .collect();
        let damage_effects: Vec<DamageEffect> = damage_areas
            .clone()
            .into_iter()
            .map(|damage_area| DamageEffect::new(damage_area))
            .collect();
        damage_effects
            .iter()
            .for_each(|effect| effect.take_effect(layer_effects));
        (damage_areas, damage_effects)
    }
}

impl Movable for Character {
    fn set_pos(&mut self, new_pos: Position) {
        self.prev_position = self.position.clone();
        self.position = new_pos;
    }

    fn move_to(&mut self, new_pos: Position, facing: Direction) {
        self.facing = facing;

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

    fn get_entity_char(&self) -> EntityCharacters {
        self.entitychar.clone()
    }

    fn get_prev_pos(&self) -> &Position {
        &self.prev_position
    }
}

impl Damageable for Character {
    fn die(&mut self) {
        self.is_alive = false;
    }

    fn get_health(&self) -> &i32 {
        &self.health
    }

    fn take_damage(&mut self, damage: i32) {
        let normal_style = Style::default();
        let hurt_style = Style::default().gray().italic();

        self.health -= damage;

        if self.health >= self.max_health / 2 {
            self.entitychar
                .replace(EntityCharacters::Character(normal_style));
        }
        if self.health < self.max_health / 2 {
            self.entitychar
                .replace(EntityCharacters::Character(hurt_style));
        }
        if self.health <= 0 {
            self.die();
        }
    }

    fn is_alive(&self) -> bool {
        self.is_alive.clone()
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
