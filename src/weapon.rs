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
                corner1: Position(x + 1, y - self.size),
                corner2: Position(x + self.size, y + self.size),
            },
        };

        DamageArea {
            area: new_area,
            damage_amount: (self.get_damage() as f32 * wielder.strength).ceil() as i32,
        }
    }

    fn get_damage(&self) -> i32 {
        return (self.base_damage as f32 * self.damage_scalar).ceil() as i32;
    }
}
