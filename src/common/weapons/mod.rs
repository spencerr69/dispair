//! This module defines weapons, damage areas, and their interactions in the game.
//! It includes a `Weapon` trait, a `Sword` implementation, and a `DamageArea` struct
//! for handling attacks and their effects on enemies.

use crate::target_types::Duration;

use std::{cell::RefCell, rc::Rc};

use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::common::{
    character::{Character, Damageable, Movable},
    coords::Area,
    enemy::{Debuffable, Enemy},
    powerup::PoweruppableWeapon,
    roguegame::{EntityCharacters, Layer},
    stats::WeaponStats,
    weapons::{flash::Flash, lightning::Lightning, pillar::Pillar},
};

pub mod flash;
pub mod lightning;
pub mod pillar;

#[derive(Clone, EnumIter, IntoStaticStr, EnumString)]
pub enum WeaponWrapper {
    #[strum(serialize = "Flash", serialize = "FLASH")]
    Flash(Option<Flash>),

    #[strum(serialize = "Pillar", serialize = "PILLAR")]
    Pillar(Option<Pillar>),

    #[strum(serialize = "Lightning", serialize = "LIGHTNING")]
    Lightning(Option<Lightning>),
}

impl PartialEq for WeaponWrapper {
    fn eq(&self, other: &Self) -> bool {
        let self_name: &'static str = self.into();
        let other_name: &'static str = other.into();
        self_name == other_name
    }
}

impl WeaponWrapper {
    pub fn get_inner(&self) -> &dyn PoweruppableWeapon {
        match self {
            WeaponWrapper::Flash(flash) => flash.as_ref().unwrap(),
            WeaponWrapper::Pillar(pillar) => pillar.as_ref().unwrap(),
            WeaponWrapper::Lightning(lightning) => lightning.as_ref().unwrap(),
        }
    }

    pub fn get_inner_mut(&mut self) -> &mut dyn PoweruppableWeapon {
        match self {
            WeaponWrapper::Flash(flash) => flash.as_mut().unwrap(),
            WeaponWrapper::Pillar(pillar) => pillar.as_mut().unwrap(),
            WeaponWrapper::Lightning(lightning) => lightning.as_mut().unwrap(),
        }
    }

    pub fn populate_inner(&mut self, weapon_stats: WeaponStats) {
        match self {
            WeaponWrapper::Flash(flash) => *flash = Some(Flash::new(weapon_stats)),
            WeaponWrapper::Pillar(pillar) => *pillar = Some(Pillar::new(weapon_stats)),
            WeaponWrapper::Lightning(lightning) => *lightning = Some(Lightning::new(weapon_stats)),
        }
    }
}

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
