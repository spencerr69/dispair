use std::time::Duration;

use crate::{
    character::{Character, Damageable, Direction, Movable, Position},
    enemy::Enemy,
    roguegame::{EntityCharacters, Layer},
};

#[derive(Clone)]
pub struct Area {
    pub corner1: Position,
    pub corner2: Position,
}

impl Area {
    pub fn new(corner1: Position, corner2: Position) -> Self {
        Area { corner1, corner2 }
    }

    pub fn iter(&self) -> impl Iterator<Item = Position> {
        let (x1, y1, x2, y2) = self.get_bounds();
        (x1..=x2).flat_map(move |x| (y1..=y2).map(move |y| Position(x, y)))
    }

    pub fn get_bounds(&self) -> (i16, i16, i16, i16) {
        let (x1, y1) = self.corner1.get();
        let (x2, y2) = self.corner2.get();

        (x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
    }

    pub fn constrain(&mut self, layer: &Layer) {
        self.corner1.constrain(layer);
        self.corner2.constrain(layer);
    }
}

#[derive(Clone)]
pub struct DamageArea {
    pub damage_amount: i32,
    pub area: Area,
    pub entity: EntityCharacters,
    pub duration: Duration,
    pub blink: bool,
}

impl DamageArea {
    pub fn deal_damage(&self, enemies: &mut Vec<Enemy>) {
        enemies.iter_mut().for_each(|enemy| {
            if enemy.get_pos().is_in_area(&self.area) {
                enemy.take_damage(self.damage_amount)
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
    size: i16,
}

impl Sword {
    pub fn new(base_damage: i32, damage_scalar: f32, size: i16) -> Self {
        Sword {
            base_damage,
            damage_scalar,
            size,
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
            damage_amount: (self.get_damage() as f32 * wielder.strength).ceil() as i32,
            entity: EntityCharacters::AttackBlackout,
            duration: Duration::from_secs_f32(0.15),
            blink: true,
        }
    }

    fn get_damage(&self) -> i32 {
        return (self.base_damage as f32 * self.damage_scalar).ceil() as i32;
    }
}
