use std::{cell::RefCell, rc::Rc};

use crate::{
    common::{
        character::Movable,
        coords::{Area, Position, SquareArea},
        powerup::PowerupTypes,
    },
    target_types::Duration,
};

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::Character,
    enemy::Enemy,
    powerup::{DynPowerup, Poweruppable},
    roguegame::{EntityCharacters, Layer},
    stats::WeaponStats,
    weapons::{DamageArea, Weapon},
};
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

    #[must_use]
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
    fn attack(&self, wielder: &Character, _: &[Enemy], layer: &Layer) -> DamageArea {
        let (x, _) = wielder.get_pos().clone().get();

        //size should be half the size for balancing
        let size = self.stats.size / 2;

        let mut area = SquareArea {
            corner1: Position(x - size, i32::MAX),
            corner2: Position(x + size, 0),
        };

        area.constrain(layer);

        DamageArea {
            damage_amount: (f64::from(self.get_damage()) * wielder.stats.damage_mult).ceil() as i32,
            area: Rc::new(RefCell::new(area)),
            entity: EntityCharacters::AttackWeak(Style::new().gray()),
            duration: Duration::from_secs_f64(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        (f64::from(self.base_damage) * self.damage_scalar).ceil() as i32
    }

    fn get_element(&self) -> Option<crate::common::debuffs::Elements> {
        None
    }
}

impl Poweruppable for Pillar {
    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Weapon
    }

    fn get_name(&self) -> String {
        "PILLAR".into()
    }

    fn get_level(&self) -> i32 {
        self.stats.level
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "PILLAR will create a damaging beam which affects an entire column of the map"
                .into(),
            2 => "Increase size by 1, increase base damage by 1".into(),
            3 => "Increase base damage by 2".into(),
            4 => "Increase damage scalar by 25%".into(),
            5 => "Increase damage scalar by 75%".into(),
            _ => String::new(),
        }
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
