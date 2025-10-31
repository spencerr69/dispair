use std::time::Duration;

use crate::{
    character::{Character, Damageable, Movable},
    coords::{Area, Direction, Position},
    enemy::Enemy,
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
    pub mark_chance: u32,
}

impl DamageArea {
    pub fn deal_damage(&self, enemies: &mut Vec<Enemy>) {
        enemies.iter_mut().for_each(|enemy| {
            if enemy.get_pos().is_in_area(&self.area) {
                enemy.take_damage(self.damage_amount);
                enemy.attempt_mark_for_explosion(self.mark_chance);
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
    mark_chance: u32,
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
            mark_chance: player_stats.mark_chance,
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
            entity: EntityCharacters::AttackBlackout,
            duration: Duration::from_secs_f32(0.01),
            blink: false,
            mark_chance: self.mark_chance,
        }
    }

    fn get_damage(&self) -> i32 {
        return (self.base_damage as f32 * self.damage_scalar).ceil() as i32;
    }
}
