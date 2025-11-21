//! This module defines weapons, damage areas, and their interactions in the game.
//! It includes a `Weapon` trait, a `Sword` implementation, and a `DamageArea` struct
//! for handling attacks and their effects on enemies.

#[cfg(not(target_family = "wasm"))]
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

#[cfg(target_family = "wasm")]
use web_time::Duration;

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::{Character, Damageable, Movable},
    coords::{Area, ChaosArea, Direction, Position, SquareArea},
    enemy::{Debuffable, Enemy, move_to_point_granular},
    powerup::{Poweruppable, WeaponPowerup},
    roguegame::{EntityCharacters, Layer},
    upgrade::WeaponStats,
};

/// Represents an area where damage is applied, created by a weapon attack.
#[derive(Clone)]
pub struct DamageArea {
    pub damage_amount: i32,
    pub area: Rc<RefCell<dyn Area>>,
    pub entity: EntityCharacters,
    pub duration: Duration,
    pub blink: bool,
    pub weapon_stats: Option<WeaponStats>,
}

impl DamageArea {
    /// Applies this damage area to every enemy whose position lies inside the area.
    ///
    /// For each affected enemy, reduces its health by `damage_amount`. If `weapon_stats` is present,
    /// iterates its `procs` and invokes each proc with `chance > 0` on the enemy.
    pub fn deal_damage(&self, enemies: &mut Vec<Enemy>) {
        enemies.iter_mut().for_each(|enemy| {
            if enemy.get_pos().is_in_area(self.area.clone()) {
                enemy.take_damage(self.damage_amount);

                // if was hit by a weapon do the following
                if let Some(stats) = &self.weapon_stats {
                    if !stats.procs.is_empty() {
                        stats.procs.iter().for_each(|(_key, proc)| {
                            if proc.chance > 0 {
                                enemy.try_proc(proc);
                            }
                        })
                    }
                }
            }
        });
    }
}

/// A trait for any weapon that can be used to attack.
pub trait Weapon {
    /// Creates a `DamageArea` representing the attack.
    fn attack(&self, wielder: &Character, enemies: &Vec<Enemy>, layer: &Layer) -> DamageArea;

    /// Calculates and returns the base damage of the weapon.
    ///Damage should be rounded up to nearest int.
    fn get_damage(&self) -> i32;
}

/// A struct representing a FLASH weapon.
#[derive(Clone)]
pub struct Flash {
    base_damage: i32,
    damage_scalar: f64,
    stats: WeaponStats,
}

impl Flash {
    const BASE_SIZE: i32 = 1;
    const BASE_DAMAGE: i32 = 2;
    const MAX_LEVEL: i32 = 5;

    /// Creates a new `Flash' with stats based on the player's current `Stats`.
    pub fn new(base_weapon_stats: WeaponStats) -> Self {
        Flash {
            base_damage: Self::BASE_DAMAGE + base_weapon_stats.damage_flat_boost,
            damage_scalar: 1.,
            stats: WeaponStats {
                size: Self::BASE_SIZE + base_weapon_stats.size,
                ..base_weapon_stats
            },
        }
    }
}

impl Poweruppable for Flash {
    fn get_level(&self) -> i32 {
        self.stats.level
    }

    fn get_next_upgrade(&self) -> Option<super::powerup::DynPowerup> {
        if self.get_level() >= Self::MAX_LEVEL {
            None
        } else {
            Some(Box::new(WeaponPowerup::new(
                "FLASH".into(),
                "Upgrade flash.".into(),
                self.get_level() + 1,
                Some(Box::new(self.clone())),
            )))
        }
    }

    fn upgrade_self(&mut self, powerup: &dyn super::powerup::Powerup) {
        self.stats.level = powerup.get_new_level();

        for i in 1..=self.stats.level {
            match i {
                1 => {}
                2 => {
                    self.stats.size += 1;
                    self.stats.damage_flat_boost += 1;
                }
                3 => {
                    self.stats.damage_flat_boost += 2;
                }
                4 => {
                    self.damage_scalar += 0.25;
                }
                5 => {
                    self.damage_scalar += 0.75;
                }
                _ => {}
            }
        }
    }
}

impl Weapon for Flash {
    /// Creates a DamageArea representing this weapon's attack originating from the wielder's position and facing direction.
    ///
    /// The produced DamageArea is positioned immediately in front of the wielder according to their facing, carries this weapon's damage scaled by `wielder.stats.damage_mult` (rounded up to an integer), and includes this weapon's `WeaponStats`.
    fn attack(&self, wielder: &Character, _: &Vec<Enemy>, layer: &Layer) -> DamageArea {
        let (x, y) = wielder.get_pos().clone().get();
        let direction = wielder.facing.clone();

        let size = self.stats.size;

        let mut new_area: SquareArea = match direction {
            Direction::DOWN => SquareArea {
                corner1: Position(x + size, y + 1),
                corner2: Position(x - size, y + size),
            },
            Direction::UP => SquareArea {
                corner1: Position(x - size, y - 1),
                corner2: Position(x + size, y - size),
            },
            Direction::LEFT => SquareArea {
                corner1: Position(x - 1, y + size),
                corner2: Position(x - size, y - size),
            },
            Direction::RIGHT => SquareArea {
                corner1: Position(x + 1, y + size),
                corner2: Position(x + size, y - size),
            },
        };

        new_area.constrain(layer);

        DamageArea {
            area: Rc::new(RefCell::new(new_area)),
            damage_amount: (self.get_damage() as f64 * wielder.stats.damage_mult).ceil() as i32,
            entity: EntityCharacters::AttackBlackout(Style::new().bold().white()),
            duration: Duration::from_secs_f32(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    /// Returns the damage of the sword, calculated from its base damage and scalar.
    fn get_damage(&self) -> i32 {
        return (self.base_damage as f64 * self.damage_scalar).ceil() as i32;
    }
}

/// A struct representing a Pillar weapon, which attacks in a vertical column.
#[derive(Clone)]
pub struct Pillar {
    base_damage: i32,
    damage_scalar: f64,
    stats: WeaponStats,
}

impl Pillar {
    const BASE_SIZE: i32 = 0;
    const BASE_DAMAGE: i32 = 3;
    const MAX_LEVEL: i32 = 5;

    pub fn new(base_weapon_stats: WeaponStats) -> Self {
        Pillar {
            base_damage: Self::BASE_DAMAGE + base_weapon_stats.damage_flat_boost,
            damage_scalar: 1.,
            stats: WeaponStats {
                size: Self::BASE_SIZE + base_weapon_stats.size,
                ..base_weapon_stats
            },
        }
    }
}

impl Weapon for Pillar {
    fn attack(&self, wielder: &Character, _: &Vec<Enemy>, layer: &Layer) -> DamageArea {
        let (x, _) = wielder.get_pos().clone().get();

        //size should be half the size for balancing
        let size = self.stats.size / 2;

        let mut area = SquareArea {
            corner1: Position(x - size, i32::MAX),
            corner2: Position(x + size, 0),
        };

        area.constrain(layer);

        DamageArea {
            damage_amount: (self.get_damage() as f64 * wielder.stats.damage_mult).ceil() as i32,
            area: Rc::new(RefCell::new(area)),
            entity: EntityCharacters::AttackWeak(Style::new().gray()),
            duration: Duration::from_secs_f64(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        (self.base_damage as f64 * self.damage_scalar).ceil() as i32
    }
}

impl Poweruppable for Pillar {
    fn get_level(&self) -> i32 {
        self.stats.level
    }

    fn get_next_upgrade(&self) -> Option<super::powerup::DynPowerup> {
        if self.get_level() >= Self::MAX_LEVEL {
            None
        } else {
            Some(Box::new(WeaponPowerup::new(
                "FLASH".into(),
                "Upgrade flash.".into(),
                self.get_level() + 1,
                Some(Box::new(self.clone())),
            )))
        }
    }

    fn upgrade_self(&mut self, powerup: &dyn super::powerup::Powerup) {
        self.stats.level = powerup.get_new_level();

        for i in 1..=self.stats.level {
            match i {
                1 => {}
                2 => {
                    self.stats.size += 1;
                    self.stats.damage_flat_boost += 1;
                }
                3 => {
                    self.stats.damage_flat_boost += 2;
                }
                4 => {
                    self.damage_scalar += 0.25;
                }
                5 => {
                    self.damage_scalar += 0.75;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct Lightning {
    base_damage: i32,
    damage_scalar: f64,
    stats: WeaponStats,
}

impl Lightning {
    const BASE_DAMAGE: i32 = 1;
    const BASE_SIZE: i32 = 1;
    const MAX_LEVEL: i32 = 5;

    pub fn new(base_weapon_stats: WeaponStats) -> Self {
        Lightning {
            base_damage: Self::BASE_DAMAGE + base_weapon_stats.damage_flat_boost,
            damage_scalar: 1.,
            stats: WeaponStats {
                size: Self::BASE_SIZE + base_weapon_stats.size,
                ..base_weapon_stats
            },
        }
    }
}

impl Weapon for Lightning {
    fn attack(&self, wielder: &Character, enemies: &Vec<Enemy>, layer: &Layer) -> DamageArea {
        let mut begin_pos = wielder.get_pos().clone();

        let mut positions = Vec::new();

        let mut enemies = enemies.clone();

        for _ in 0..self.stats.size {
            let closest = enemies.iter().reduce(|acc, enemy| {
                let (dist_x, dist_y) = enemy.get_pos().get_distance(&begin_pos);
                let enemy_total_dist = dist_x.abs() + dist_y.abs();

                let (acc_dist_x, acc_dist_y) = acc.get_pos().get_distance(&begin_pos);
                let acc_total_dist = acc_dist_x.abs() + acc_dist_y.abs();

                if enemy_total_dist < acc_total_dist && enemy_total_dist > 2 || acc_total_dist <= 2
                {
                    enemy
                } else {
                    acc
                }
            });

            let mut current_pos = begin_pos.clone();

            if let Some(closest) = closest {
                let desired_pos = closest.get_pos().clone();

                while current_pos != desired_pos {
                    positions.push(current_pos.clone());
                    (current_pos, _) = move_to_point_granular(&current_pos, &desired_pos, false);
                }

                (current_pos, _) = move_to_point_granular(&current_pos, &desired_pos, false);
                positions.push(current_pos.clone());

                begin_pos = desired_pos;

                enemies = enemies
                    .iter()
                    .filter_map(|e| {
                        if e != closest {
                            return Some(e.clone());
                        } else {
                            return None;
                        }
                    })
                    .collect();
            }
        }

        let mut area = ChaosArea::new(positions);
        area.constrain(layer);

        DamageArea {
            damage_amount: (self.get_damage() as f64 * wielder.stats.damage_mult).ceil() as i32,
            area: Rc::new(RefCell::new(area)),
            entity: EntityCharacters::AttackMist(Style::new().white()),
            duration: Duration::from_secs_f64(0.1),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        (self.base_damage as f64 * self.damage_scalar).ceil() as i32
    }
}

impl Poweruppable for Lightning {
    fn get_level(&self) -> i32 {
        self.stats.level
    }

    fn get_next_upgrade(&self) -> Option<super::powerup::DynPowerup> {
        if self.get_level() >= Self::MAX_LEVEL {
            None
        } else {
            Some(Box::new(WeaponPowerup::new(
                "FLASH".into(),
                "Upgrade flash.".into(),
                self.get_level() + 1,
                Some(Box::new(self.clone())),
            )))
        }
    }

    fn upgrade_self(&mut self, powerup: &dyn super::powerup::Powerup) {
        self.stats.level = powerup.get_new_level();

        for i in 1..=self.stats.level {
            match i {
                1 => {}
                2 => {
                    self.stats.size += 1;
                    self.stats.damage_flat_boost += 1;
                }
                3 => {
                    self.stats.damage_flat_boost += 2;
                }
                4 => {
                    self.damage_scalar += 0.25;
                }
                5 => {
                    self.damage_scalar += 0.75;
                }
                _ => {}
            }
        }
    }
}
