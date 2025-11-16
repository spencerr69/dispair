#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

#[cfg(target_family = "wasm")]
use web_time::Duration;

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::{Character, Damageable, Movable},
    coords::{Area, Direction, Position},
    enemy::{Debuff, Debuffable, Enemy},
    roguegame::EntityCharacters,
    upgrade::Stats,
};

#[derive(Clone)]
pub struct DamageArea {
    pub damage_amount: i32,
    pub area: Area,
    pub entity: EntityCharacters,
    pub duration: Duration,
    pub blink: bool,
    pub weapon_stats: Option<Stats>,
}

impl DamageArea {
    pub fn deal_damage(&self, enemies: &mut Vec<Enemy>) {
        enemies.iter_mut().for_each(|enemy| {
            if enemy.get_pos().is_in_area(&self.area) {
                enemy.take_damage(self.damage_amount);

                // if was hit by a weapon do the following
                if let Some(stats) = self.weapon_stats.clone() {
                    if stats.mark_chance > 0 {
                        enemy.try_proc(
                            Debuff::MarkedForExplosion(
                                stats.mark_explosion_size,
                                (6. * stats.damage_mult).ceil() as i32,
                            ),
                            stats.mark_chance,
                        );
                    }
                }
            }
        });
    }
}

pub trait Weapon {
    fn attack(&self, wielder: &Character) -> DamageArea;

    ///Damage should be rounded up to nearest int.
    fn get_damage(&self) -> i32;
}

pub struct Sword {
    base_damage: i32,
    damage_scalar: f32,
    size: i32,
    stats: Stats,
}

impl Sword {
    pub fn new(player_stats: Stats) -> Self {
        let size_base = 1;
        let base_damage = 2;
        let damage_scalar = 1.;
        Sword {
            base_damage: base_damage + player_stats.damage_flat_boost,
            damage_scalar,
            size: size_base + player_stats.size,
            stats: player_stats,
        }
    }
}

impl Weapon for Sword {
    fn attack(&self, wielder: &Character) -> DamageArea {
        let (x, y) = wielder.get_pos().clone().get();
        let direction = wielder.facing.clone();

        let new_area: Area = match direction {
            Direction::DOWN => Area {
                corner1: Position(x + self.size, y + 1),
                corner2: Position(x - self.size, y + self.size),
            },
            Direction::UP => Area {
                corner1: Position(x - self.size, y - 1),
                corner2: Position(x + self.size, y - self.size),
            },
            Direction::LEFT => Area {
                corner1: Position(x - 1, y + self.size),
                corner2: Position(x - self.size, y - self.size),
            },
            Direction::RIGHT => Area {
                corner1: Position(x + 1, y + self.size),
                corner2: Position(x + self.size, y - self.size),
            },
        };

        DamageArea {
            area: new_area,
            damage_amount: (self.get_damage() as f64 * wielder.strength).ceil() as i32,
            entity: EntityCharacters::AttackBlackout(Style::new().bold().white()),
            duration: Duration::from_secs_f32(0.01),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        return (self.base_damage as f32 * self.damage_scalar).ceil() as i32;
    }
}
