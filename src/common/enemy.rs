//! This module defines the `Enemy` struct and its related traits and behaviors.
//! It includes logic for enemy movement, health, attacks, and debuffs.
#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use web_time::Duration;

use rand::Rng;
use ratatui::style::Style;
use ratatui::style::Stylize;

use crate::common::upgrade::DebuffStats;
use crate::common::upgrade::Proc;
use crate::common::{
    character::*, coords::Area, coords::Direction, coords::Position, effects::DamageEffect,
    roguegame::*, weapon::DamageArea,
};

/// Represents debuffs that can be applied to enemies.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuffTypes {
    MarkedForExplosion,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Debuff {
    pub debuff_type: DebuffTypes,
    pub stats: DebuffStats,
}

impl Debuff {}

/// A trait for effects that trigger when an enemy dies.
pub trait OnDeathEffect {
    /// Called when an enemy dies, potentially creating a `DamageArea`.
    fn on_death(&self, enemy: Enemy) -> Option<DamageArea>;
}

impl OnDeathEffect for Debuff {
    /// Produces an optional area-of-effect damage specification to emit when this debuff triggers on an enemy's death.
    ///
    /// If the debuff is `MarkedForExplosion` and `stats.size` is `Some(size)`, returns a `DamageArea` describing a square area centered on the enemy with radius `size`, using `stats.damage` (or `0` if absent) as the damage amount, an `AttackMist` visual styled dark gray, a duration of 0.15 seconds, `blink = false`, and no `weapon_stats`. If `stats.size` is `None`, returns `None`.
    fn on_death(&self, enemy: Enemy) -> Option<DamageArea> {
        match self.debuff_type {
            DebuffTypes::MarkedForExplosion => {
                if let Some(size) = self.stats.size {
                    let area = Area {
                        corner1: Position(
                            enemy.position.0.saturating_sub(size),
                            enemy.position.1.saturating_sub(size),
                        ),
                        corner2: Position(
                            enemy.position.0.saturating_add(size),
                            enemy.position.1.saturating_add(size),
                        ),
                    };

                    Some(DamageArea {
                        damage_amount: self.stats.damage.unwrap_or(0),
                        area,
                        entity: EntityCharacters::AttackMist(Style::new().dark_gray()),
                        duration: Duration::from_secs_f64(0.15),
                        blink: false,
                        weapon_stats: None,
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// A trait defining the behavior of an enemy.
pub trait EnemyBehaviour {
    /// Creates a new enemy with the given properties.
    fn new(position: Position, damage: i32, health: i32, worth: u32) -> Self;

    /// Gets the amount of gold the enemy is worth.
    fn get_worth(&self) -> u32;

    /// Updates the enemy's state, including movement and attacks.
    fn update(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        damage_effects: &mut Vec<DamageEffect>,
    );
}

/// Represents an enemy in the game.
#[derive(Clone)]
pub struct Enemy {
    position: Position,
    prev_position: Position,

    pub facing: Direction,

    damage: i32,

    health: i32,
    pub max_health: i32,
    is_alive: bool,

    entitychar: EntityCharacters,

    worth: u32,

    pub debuffs: Vec<Debuff>,
}

/// A trait for entities that can have debuffs applied to them.
pub trait Debuffable {
    /// Attempts to apply a debuff with a certain chance of success.
    fn try_proc(&mut self, proc: &Proc);
    /// Counts the number of a specific debuff on the entity.
    fn count_debuff(&self, debuff: &Debuff) -> u32;
}

impl Debuffable for Enemy {
    /// Attempts to apply the given `Proc`'s debuff to the enemy based on the proc's chance; if the proc succeeds and the enemy does not already have that debuff, the debuff is appended to the enemy's debuff list.
    fn try_proc(&mut self, proc: &Proc) {
        let mut rng = rand::rng();

        let roll = rng.random_range(1..=100);

        match proc.debuff.debuff_type {
            DebuffTypes::MarkedForExplosion => {
                if roll <= proc.chance && self.count_debuff(&proc.debuff) < 1 {
                    self.debuffs.push(proc.debuff.clone());
                }
            }
        }
    }

    /// Counts how many active debuffs share the same debuff type as the provided `debuff`.
    ///
    /// # Parameters
    ///
    /// - `debuff`: The debuff whose `debuff_type` is used for matching against the enemy's active debuffs.
    ///
    /// # Returns
    ///
    /// `u32` number of debuffs in `self.debuffs` whose `debuff_type` equals `debuff.debuff_type`.
    ///
    fn count_debuff(&self, debuff: &Debuff) -> u32 {
        self.debuffs.iter().fold(0, |acc, e| {
            if e.debuff_type == debuff.debuff_type {
                acc + 1
            } else {
                acc
            }
        })
    }
}

impl Enemy {
    /// Update the enemy's visual style to reflect any active debuffs.
    ///
    /// Currently applies styling for `DebuffTypes::MarkedForExplosion` by making the
    /// enemy's character style bold and gray.
    ///
    fn change_style_with_debuff(&mut self) {
        let style = self.entitychar.style_mut();

        self.debuffs
            .iter()
            .for_each(|debuff| match debuff.debuff_type {
                DebuffTypes::MarkedForExplosion => {
                    *style = style.bold().gray();
                }
            })
    }
}

impl EnemyBehaviour for Enemy {
    fn new(position: Position, damage: i32, health: i32, worth: u32) -> Self {
        Enemy {
            position: position.clone(),
            prev_position: position,

            facing: Direction::UP,

            damage,

            health,
            max_health: health,
            is_alive: true,

            entitychar: EntityCharacters::Enemy(Style::default()),

            worth,

            debuffs: Vec::new(),
        }
    }

    fn get_worth(&self) -> u32 {
        self.worth.clone()
    }

    fn update(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        damage_effects: &mut Vec<DamageEffect>,
    ) {
        let mut rng = rand::rng();

        self.prev_position = self.position.clone();

        self.change_style_with_debuff();

        if is_next_to_character(character.get_pos(), &self.position) {
            character.take_damage(self.damage);
            damage_effects.push(DamageEffect::new(
                Area::from(character.get_pos().clone()),
                EntityCharacters::AttackBlackout(Style::new().bold().dark_gray()),
                Duration::from_secs_f64(0.2),
                true,
            ));
        }

        let (dist_x, dist_y) = self.position.get_distance(character.get_pos());
        let (x, y) = self.position.get();
        let desired_pos: Position;
        let desired_facing: Direction;

        let total_dist = dist_x.abs() + dist_y.abs();

        let choice = rng.random_ratio(dist_x.abs().max(1) as u32, total_dist.abs().max(1) as u32);

        if choice {
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

        if can_stand(layer, &desired_pos) && &desired_pos != character.get_pos() {
            self.move_to(desired_pos, desired_facing);
        }
    }
}

impl Movable for Enemy {
    fn get_facing(&self) -> Direction {
        self.facing.clone()
    }

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

    fn get_entity_char(&self) -> EntityCharacters {
        self.entitychar.clone()
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
        let normal_style = Style::default();
        let hurt_style = Style::default().gray().italic();

        self.health -= damage;

        if self.health >= self.max_health / 2 {
            self.entitychar
                .replace(EntityCharacters::Enemy(normal_style));
        }
        if self.health < self.max_health / 2 {
            self.entitychar.replace(EntityCharacters::Enemy(hurt_style));
        }
        if self.health <= 0 {
            self.die();
        }
    }
}
