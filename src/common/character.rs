use ratatui::style::{Style, Stylize};

use crate::common::{
    coords::{Direction, Position},
    effects::DamageEffect,
    roguegame::Layer,
    upgrade::PlayerState,
    weapon::{DamageArea, Sword, Weapon},
};

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

#[cfg(target_family = "wasm")]
use web_time::Instant;

use crate::common::roguegame::EntityCharacters;

pub struct Character {
    position: Position,
    prev_position: Position,
    last_moved: Instant,
    pub facing: Direction,

    pub movement_speed: f64,
    pub strength: f64,
    pub attack_speed: f64,

    health: i32,
    max_health: i32,
    is_alive: bool,

    weapons: Vec<Box<dyn Weapon>>,

    // pub player_stats: Stats,
    entitychar: EntityCharacters,
}

///Trait for an entity which can move
pub trait Movable {
    fn set_pos(&mut self, new_pos: Position);
    fn get_pos(&self) -> &Position;
    fn move_to(&mut self, new_pos: Position, facing: Direction);
    fn move_to_safe(&mut self, new_pos: Position, facing: Direction, layer: &Layer) {
        let mut position = new_pos;

        position.constrain(layer);

        self.move_to(position, facing);
    }
    fn get_prev_pos(&self) -> &Position;
    fn get_entity_char(&self) -> EntityCharacters;
    fn get_facing(&self) -> Direction;

    fn move_back(&mut self, steps: i32, layer: &Layer) {
        let current_direction = self.get_facing();

        match current_direction {
            Direction::UP => self.move_to_safe(
                Position(self.get_pos().0, self.get_pos().1 + steps),
                Direction::DOWN,
                layer,
            ),
            Direction::DOWN => self.move_to_safe(
                Position(self.get_pos().0, self.get_pos().1 - steps),
                Direction::UP,
                layer,
            ),
            Direction::LEFT => self.move_to_safe(
                Position(self.get_pos().0 + steps, self.get_pos().1),
                Direction::RIGHT,
                layer,
            ),
            Direction::RIGHT => self.move_to_safe(
                Position(self.get_pos().0 - steps, self.get_pos().1),
                Direction::LEFT,
                layer,
            ),
        }
    }
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

impl Character {
    pub fn new(player_state: PlayerState) -> Self {
        let max_health = player_state.stats.health;
        Character {
            position: Position(0, 0),
            prev_position: Position(0, 0),
            last_moved: Instant::now(),
            facing: Direction::UP,
            movement_speed: player_state.stats.movement_speed_mult,
            strength: player_state.stats.damage_mult,
            attack_speed: player_state.stats.attack_speed_mult,
            // player_stats: player_state.stats.clone(),
            health: max_health,
            max_health: max_health,
            is_alive: true,

            entitychar: EntityCharacters::Character(Style::default()),

            weapons: vec![Box::new(Sword::new(player_state.stats))],
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
            .map(|damage_area| DamageEffect::from(damage_area))
            .collect();
        damage_effects
            .iter()
            .for_each(|effect| effect.take_effect(layer_effects));
        (damage_areas, damage_effects)
    }
}

impl Movable for Character {
    fn get_facing(&self) -> Direction {
        self.facing.clone()
    }

    fn set_pos(&mut self, new_pos: Position) {
        self.prev_position = self.position.clone();
        self.position = new_pos;
    }

    fn move_to(&mut self, new_pos: Position, facing: Direction) {
        self.facing = facing;

        let attempt_time = Instant::now();
        let difference = attempt_time.duration_since(self.last_moved).as_millis();
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
