//! This module defines the `Enemy` struct and its related traits and behaviors.
//! It includes logic for enemy movement, health, attacks, and debuffs.
use crate::{
    common::debuffs::{Debuff, DebuffTypes},
    target_types::Duration,
};

use rand::Rng;
use ratatui::style::{Style, Stylize};

use crate::common::{
    character::*,
    coords::{Direction, Position, SquareArea},
    effects::DamageEffect,
    roguegame::*,
    stats::Proc,
};

/// A trait defining the behavior of an enemy.
pub trait EnemyBehaviour {
    /// Creates a new enemy with the given properties.
    fn new(position: Position, damage: i32, health: i32, drops: EnemyDrops) -> Self;

    /// Gets the amount of gold the enemy is worth.
    fn get_drops(&self) -> EnemyDrops;

    /// Updates the enemy's state, including movement and attacks.
    fn update(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        damage_effects: &mut Vec<DamageEffect>,
    );
}

#[derive(Clone, PartialEq, Eq)]
pub struct EnemyDrops {
    pub gold: u128,
    pub xp: u128,
}

/// Represents an enemy in the game.
#[derive(Clone, PartialEq, Eq)]
pub struct Enemy {
    pub position: Position,
    prev_position: Position,

    pub facing: Direction,

    damage: i32,

    health: i32,
    pub max_health: i32,
    is_alive: bool,

    entitychar: EntityCharacters,

    drops: EnemyDrops,

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

        if roll <= proc.chance {
            match proc.debuff.debuff_type {
                DebuffTypes::FlameBurn => {
                    if self.count_debuff(&proc.debuff) < 2 {
                        self.debuffs.push(proc.debuff.clone());
                    } else {
                        self.try_proc(&Proc {
                            chance: 100,
                            debuff: Debuff {
                                debuff_type: DebuffTypes::FlameIgnite,
                                stats: proc.debuff.stats.clone(),
                                complete: false,
                            },
                        })
                    }
                }
                _ => {
                    if self.count_debuff(&proc.debuff) < 1 {
                        self.debuffs.push(proc.debuff.clone());
                    }
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

    fn change_style_with_debuff(&mut self) {
        let style = self.entitychar.style_mut();

        self.debuffs
            .iter()
            .for_each(|debuff| match debuff.debuff_type {
                DebuffTypes::MarkedForExplosion => {
                    *style = style.bold();
                }
                DebuffTypes::FlameBurn => *style = style.red(),
                _ => {}
            })
    }
}

impl EnemyBehaviour for Enemy {
    fn new(position: Position, damage: i32, health: i32, drops: EnemyDrops) -> Self {
        Enemy {
            position: position.clone(),
            prev_position: position,

            facing: Direction::UP,

            damage,

            health,
            max_health: health,
            is_alive: true,

            entitychar: EntityCharacters::Enemy(Style::default()),

            drops,

            debuffs: Vec::new(),
        }
    }

    fn get_drops(&self) -> EnemyDrops {
        self.drops.clone()
    }

    fn update(
        &mut self,
        character: &mut Character,
        layer: &Layer,
        damage_effects: &mut Vec<DamageEffect>,
    ) {
        self.prev_position = self.position.clone();

        self.change_style_with_debuff();

        if is_next_to_character(character.get_pos(), &self.position) {
            character.take_damage(self.damage);
            damage_effects.push(DamageEffect::new(
                SquareArea::from(character.get_pos().clone()),
                EntityCharacters::AttackBlackout(Style::new().bold().dark_gray()),
                Duration::from_secs_f64(0.2),
                true,
            ));
        }

        let (desired_pos, desired_facing) =
            move_to_point_granular(&self.position, character.get_pos(), true);

        if can_stand(layer, &desired_pos) && &desired_pos != character.get_pos() {
            self.move_to(desired_pos, desired_facing);
        }
    }
}

pub fn move_to_point_granular(
    self_pos: &Position,
    desired_location: &Position,
    random: bool,
) -> (Position, Direction) {
    let (dist_x, dist_y) = self_pos.get_distance(desired_location);
    let (x, y) = self_pos.get();
    let desired_pos: Position;
    let desired_facing: Direction;

    let total_dist = dist_x.abs() + dist_y.abs();

    let choice: bool;

    if random {
        let mut rng = rand::rng();
        choice = rng.random_ratio(dist_x.abs().max(1) as u32, total_dist.abs().max(1) as u32);
    } else {
        choice = dist_x.abs() > dist_y.abs();
    }

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

    (desired_pos, desired_facing)
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
