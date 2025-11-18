//! This module defines weapons, damage areas, and their interactions in the game.
//! It includes a `Weapon` trait, a `Sword` implementation, and a `DamageArea` struct
//! for handling attacks and their effects on enemies.

#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

#[cfg(target_family = "wasm")]
use web_time::Duration;

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::{Character, Damageable, Movable},
    coords::{Area, Direction, Position},
    enemy::{Debuffable, Enemy},
    roguegame::EntityCharacters,
    upgrade::WeaponStats,
};

/// Represents an area where damage is applied, created by a weapon attack.
#[derive(Clone)]
pub struct DamageArea {
    pub damage_amount: i32,
    pub area: Area,
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
    ///
    /// # Examples
    ///
    /// ```
    /// // pseudocode example showing usage
    /// let mut enemies = vec![Enemy::new_at(0, 0)];
    /// let area = DamageArea { /* area covering (0,0), damage_amount: 5, weapon_stats: None, ... */ };
    /// area.deal_damage(&mut enemies);
    /// ```
    pub fn deal_damage(&self, enemies: &mut Vec<Enemy>) {
        enemies.iter_mut().for_each(|enemy| {
            if enemy.get_pos().is_in_area(&self.area) {
                enemy.take_damage(self.damage_amount);

                // if was hit by a weapon do the following
                if let Some(stats) = self.weapon_stats.clone() {
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
    fn attack(&self, wielder: &Character) -> DamageArea;

    /// Calculates and returns the base damage of the weapon.
    ///Damage should be rounded up to nearest int.
    fn get_damage(&self) -> i32;
}

/// A struct representing a sword weapon.
pub struct Flash {
    base_damage: i32,
    damage_scalar: f64,
    stats: WeaponStats,
}

impl Flash {
    const BASE_SIZE: i32 = 1;
    const BASE_DAMAGE: i32 = 2;

    /// Creates a new Flash weapon configured from the given `WeaponStats`.
    ///
    /// The resulting `Flash` has its `base_damage` set to `BASE_DAMAGE` plus
    /// `base_weapon_stats.damage_flat_boost`, its `damage_scalar` set to `1.0`,
    /// and its `stats.size` set to `BASE_SIZE` plus `base_weapon_stats.size`.
    ///
    /// # Parameters
    ///
    /// - `base_weapon_stats`: Weapon stat modifiers to apply to the new `Flash`.
    ///
    /// # Returns
    ///
    /// The configured `Flash` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let base_stats = WeaponStats::default();
    /// let flash = Flash::new(base_stats);
    /// assert!(flash.get_damage() >= Flash::BASE_DAMAGE);
    /// ```
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

impl Weapon for Flash {
    /// Creates a DamageArea representing this weapon's attack originating from the wielder's position and facing direction.
    ///
    /// The produced DamageArea is positioned immediately in front of the wielder according to their facing, carries this weapon's damage scaled by `wielder.stats.damage_mult` (rounded up to an integer), and includes this weapon's `WeaponStats`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let flash = Flash::new(WeaponStats::default());
    /// let wielder = Character::default();
    /// let area = flash.attack(&wielder);
    /// // damage_amount reflects weapon damage scaled by wielder.stats.damage_mult
    /// assert!(area.damage_amount >= 0);
    /// ```
    fn attack(&self, wielder: &Character) -> DamageArea {
        let (x, y) = wielder.get_pos().clone().get();
        let direction = wielder.facing.clone();

        let size = self.stats.size;

        let new_area: Area = match direction {
            Direction::DOWN => Area {
                corner1: Position(x + size, y + 1),
                corner2: Position(x - size, y + size),
            },
            Direction::UP => Area {
                corner1: Position(x - size, y - 1),
                corner2: Position(x + size, y - size),
            },
            Direction::LEFT => Area {
                corner1: Position(x - 1, y + size),
                corner2: Position(x - size, y - size),
            },
            Direction::RIGHT => Area {
                corner1: Position(x + 1, y + size),
                corner2: Position(x + size, y - size),
            },
        };

        DamageArea {
            area: new_area,
            damage_amount: (self.get_damage() as f64 * wielder.stats.damage_mult).ceil() as i32,
            entity: EntityCharacters::AttackBlackout(Style::new().bold().white()),
            duration: Duration::from_secs_f32(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    /// Compute the weapon's damage after applying its scalar.
    ///
    /// # Returns
    ///
    /// `i32` equal to the ceiling of `base_damage * damage_scalar`.
    ///
    /// # Examples
    ///
    /// ```
    /// let dmg = (2f64 * 1.5).ceil() as i32;
    /// assert_eq!(dmg, 3);
    /// ```
    fn get_damage(&self) -> i32 {
        return (self.base_damage as f64 * self.damage_scalar).ceil() as i32;
    }
}

pub struct Pillar {
    base_damage: i32,
    damage_scalar: f64,
    stats: WeaponStats,
}

impl Pillar {
    const BASE_SIZE: i32 = 0;
    const BASE_DAMAGE: i32 = 1;

    /// Create a new `Pillar` weapon using the provided `WeaponStats`.
    ///
    /// The returned `Pillar` has `base_damage` set to `BASE_DAMAGE` plus the
    /// `damage_flat_boost` from `base_weapon_stats`, `stats.size` set to
    /// `BASE_SIZE` plus `base_weapon_stats.size`, and `damage_scalar` initialized to `1.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// let base_stats = WeaponStats { size: 2, damage_flat_boost: 1, ..Default::default() };
    /// let pillar = Pillar::new(base_stats);
    /// ```
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
    /// Creates a DamageArea representing a vertical pillar attack centered on the wielder.
    ///
    /// The area spans horizontally according to the weapon's size and vertically from `i32::MAX` down to `0`. Damage is the weapon's base damage scaled by `wielder.stats.damage_mult` and rounded up.
    ///
    /// # Returns
    ///
    /// A `DamageArea` covering a vertical pillar centered on the wielder's x-position, with damage scaled by the wielder's damage multiplier.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let pillar = Pillar::new(WeaponStats::default());
    /// let wielder = /* construct or obtain a Character positioned where the attack should originate */;
    /// let damage_area = pillar.attack(&wielder);
    /// ```
    fn attack(&self, wielder: &Character) -> DamageArea {
        let (x, _) = wielder.get_pos().clone().get();

        let size = self.stats.size / 2;

        let area = Area {
            corner1: Position(x - size, i32::MAX),
            corner2: Position(x + size, 0),
        };

        DamageArea {
            damage_amount: (self.get_damage() as f64 * wielder.stats.damage_mult).ceil() as i32,
            area,
            entity: EntityCharacters::AttackWeak(Style::new().gray()),
            duration: Duration::from_secs_f64(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    /// Compute the weapon's base damage after applying its damage scalar and rounding up.
    ///
    /// # Returns
    ///
    /// The resulting damage as an `i32`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Equivalent to `get_damage()` for a base damage of 2 and scalar 1.5
    /// let dmg = (2.0_f64 * 1.5_f64).ceil() as i32;
    /// assert_eq!(dmg, 3);
    /// ```
    fn get_damage(&self) -> i32 {
        (self.base_damage as f64 * self.damage_scalar).ceil() as i32
    }
}