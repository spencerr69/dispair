use crate::character::{Character, Damageable, Direction, Movable, Position};

struct Area {
    corner1: Position,
    corner2: Position,
}

pub struct DamageArea {
    damage_amount: i32,
    area: Area,
}

pub trait Weapon {
    fn attack(&self, wielder: &Character) -> DamageArea;

    ///Damage should be rounded up to nearest int.
    fn get_damage(&self) -> i32;
}

pub struct Sword {
    base_damage: i32,
    damage_scalar: f32,
    size: u32,
}

impl Sword {
    pub fn new(base_damage: i32, damage_scalar: f32, size: u32) -> Self {
        Sword {
            base_damage,
            damage_scalar,
            size,
        }
    }
}

impl Weapon for Sword {
    fn attack(&self, wielder: &Character) -> DamageArea {
        let (mut x, mut y) = wielder.get_pos().clone().get();
        let direction = wielder.facing;

        let new_area: Area = match direction {
            Direction::DOWN => Area {
                corner1: Position(x + self.size, y + 1),
                corner2: Position(x - self.size, y + self.size),
            },
            Direction::UP => (x, y - 1),
            Direction::LEFT => (x - 1, y),
            Direction::RIGHT => (x + 1, y),
        };

        DamageArea {
            area: Area {
                corner1: Position(),
            },
        }
    }

    fn get_damage(&self) -> i32 {
        return (self.base_damage as f32 * self.damage_scalar).ceil() as i32;
    }
}
