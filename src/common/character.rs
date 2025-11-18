//! This module defines the `Character` struct and related traits for movable and damageable entities.
//! It handles character movement, health, attacks, and other core gameplay mechanics.
use ratatui::style::{Style, Stylize};

use crate::common::{
    coords::{Direction, Position},
    effects::DamageEffect,
    roguegame::Layer,
    upgrade::{PlayerState, PlayerStats},
    weapon::{DamageArea, Flash, Pillar, Weapon},
};

#[cfg(not(target_family = "wasm"))]
use std::time::{Duration, Instant};

#[cfg(target_family = "wasm")]
use web_time::{Duration, Instant};

use crate::common::roguegame::EntityCharacters;

/// Represents the player character in the game.
pub struct Character {
    position: Position,
    prev_position: Position,
    last_moved: Instant,
    pub facing: Direction,

    pub stats: PlayerStats,

    health: i32,
    max_health: i32,
    is_alive: bool,

    weapons: Vec<Box<dyn Weapon>>,

    // pub player_stats: Stats,
    entitychar: EntityCharacters,
}

/// A trait for entities that can move within the game world.
pub trait Movable {
    /// Sets the new position of the entity.
    fn set_pos(&mut self, new_pos: Position);
    /// Gets the current position of the entity.
    fn get_pos(&self) -> &Position;
    /// Moves the entity to a new position with a specified facing direction.
    fn move_to(&mut self, new_pos: Position, facing: Direction);
    /// Moves the entity to a new position safely, ensuring it stays within the layer boundaries.
    fn move_to_safe(&mut self, new_pos: Position, facing: Direction, layer: &Layer) {
        let mut position = new_pos;

        position.constrain(layer);

        self.move_to(position, facing);
    }
    /// Gets the previous position of the entity.
    fn get_prev_pos(&self) -> &Position;
    /// Gets the character representation of the entity.
    fn get_entity_char(&self) -> EntityCharacters;
    /// Gets the current facing direction of the entity.
    fn get_facing(&self) -> Direction;

    /// Moves the entity back a certain number of steps from its current facing direction.
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

/// A trait for entities that have health and can take damage.
pub trait Damageable {
    /// Gets the current health of the entity.
    fn get_health(&self) -> &i32;

    /// Applies damage to the entity. Can also be used for healing by providing a negative value.
    fn take_damage(&mut self, damage: i32);

    /// Handles the death of the entity.
    fn die(&mut self);

    /// Checks if the entity is alive.
    fn is_alive(&self) -> bool;
}

impl Character {
    /// Creates a new Character initialized from the given player state.
    ///
    /// The new character starts at position (0,0), facing up, with health and stats
    /// taken from `player_state.stats.player_stats`. The character's weapon loadout
    /// is initialized from `player_state.stats.weapon_stats`.
    ///
    /// # Parameters
    ///
    /// - `player_state`: source of player stats, health, and weapon configuration.
    ///
    /// # Returns
    ///
    /// A `Character` populated with position, facing, health, stats, entity character,
    /// and weapons derived from the provided `player_state`.
    ///
    /// # Examples
    ///
    /// ```
    /// let player_state = PlayerState::default();
    /// let c = Character::new(player_state);
    /// assert_eq!(c.position, Position(0, 0));
    /// assert_eq!(c.facing, Direction::UP);
    /// assert_eq!(c.health, c.max_health);
    /// ```
    pub fn new(player_state: PlayerState) -> Self {
        let stats = player_state.stats;
        let weapon_stats = stats.weapon_stats.clone();
        let max_health = stats.player_stats.health;
        let player_stats = stats.player_stats;

        Character {
            position: Position(0, 0),
            prev_position: Position(0, 0),
            last_moved: Instant::now(),
            facing: Direction::UP,

            stats: player_stats,

            // player_stats: player_state.stats.clone(),
            health: max_health,
            max_health: max_health,
            is_alive: true,

            entitychar: EntityCharacters::Character(Style::default()),

            weapons: vec![
                Box::new(Flash::new(weapon_stats.clone())),
                Box::new(Pillar::new(weapon_stats)),
            ],
            // weapons: vec![],
        }
    }

    /// Generates damage areas for each equipped weapon and corresponding damage effects, applies each effect to the provided layer, staggers their start times, and updates them.
    ///
    /// The provided `layer_effects` is modified by constraining each damage area's region to the layer before effects are produced.
    ///
    /// # Returns
    ///
    /// A tuple where the first element is a `Vec<DamageArea>` produced by the weapons, and the second element is a `Vec<DamageEffect>` derived from those areas with staggered delays applied (`0.15` seconds multiplied by each effect's index).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Assuming `character` implements `attack` and `layer` is a mutable Layer:
    /// // let (areas, effects) = character.attack(&mut layer);
    /// ```
    pub fn attack(&self, layer: &Layer) -> (Vec<DamageArea>, Vec<DamageEffect>) {
        let damage_areas: Vec<DamageArea> = self
            .weapons
            .iter()
            .map(|weapon| weapon.attack(&self))
            .map(|mut damage_area| {
                damage_area.area.constrain(layer);
                damage_area
            })
            .collect();
        let mut damage_effects: Vec<DamageEffect> = damage_areas
            .clone()
            .into_iter()
            .map(|damage_area| DamageEffect::from(damage_area))
            .collect();
        damage_effects
            .iter_mut()
            .enumerate()
            .for_each(|(i, effect)| {
                effect.delay(Duration::from_secs_f64(0.15 * i as f64));
                effect.update();
            });
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

    /// Attempts to move the character to `new_pos` and update its facing; movement is throttled by the character's movement speed multiplier and `last_moved` is updated when the move occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// // let mut character = /* obtain Character */
    /// // character.move_to(Position { x: 1, y: 2 }, Direction::North);
    /// ```
    fn move_to(&mut self, new_pos: Position, facing: Direction) {
        self.facing = facing;

        let attempt_time = Instant::now();
        let difference = attempt_time.duration_since(self.last_moved).as_millis() as u64;
        // this is what movement speed controls vv
        let timeout = (100.0 / self.stats.movement_speed_mult.max(0.01)).round() as u64;

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
