use ratatui::style::{Style, Stylize};

use crate::{
    coords::{Direction, Position},
    effects::DamageEffect,
    roguegame::Layer,
    upgrade::PlayerState,
    weapon::{DamageArea, Sword, Weapon},
};
use std::time::SystemTime;

use crate::roguegame::EntityCharacters;

pub struct Character {
    position: Position,
    prev_position: Position,
    last_moved: SystemTime,
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

impl Character {
    pub fn new(player_state: PlayerState) -> Self {
        let max_health = 10000;
        Character {
            position: Position(0, 0),
            prev_position: Position(0, 0),
            last_moved: SystemTime::now(),
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
