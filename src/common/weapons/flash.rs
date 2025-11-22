use std::{cell::RefCell, rc::Rc};

use crate::{
    common::{character::Movable, coords::Area},
    target_types::Duration,
};

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::Character,
    coords::{Direction, Position, SquareArea},
    enemy::Enemy,
    powerup::{DynPowerup, PowerupTypes, Poweruppable},
    roguegame::{EntityCharacters, Layer},
    stats::WeaponStats,
    weapons::{DamageArea, Weapon},
};

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
    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Weapon
    }

    fn get_name(&self) -> String {
        "FLASH".into()
    }

    fn get_level(&self) -> i32 {
        self.stats.level
    }

    fn upgrade_self(&mut self, powerup: &DynPowerup) {
        let from = powerup.get_current_level();
        let to = powerup.get_new_level();
        if to <= from {
            return;
        }
        self.stats.level = to;

        for i in (from + 1)..=to {
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

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "FLASH will create a brief damaging field directly in front of you.".into(),
            2 => "Increase size by 1, increase base damage by 1".into(),
            3 => "Increase base damage by 2".into(),
            4 => "Increase damage scalar by 25%".into(),
            5 => "Increase damage scalar by 75%".into(),
            _ => "".into(),
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
