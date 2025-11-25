//! This module defines weapons, damage areas, and their interactions in the game.
//! It includes a `Weapon` trait, a `Sword` implementation, and a `DamageArea` struct
//! for handling attacks and their effects on enemies.

use crate::{common::debuffs::Elements, target_types::Duration};

use std::{cell::RefCell, rc::Rc};

use ratatui::style::{Style, Stylize};
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::common::character::Renderable;
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
    /// Get a reference to the inner weapon.
    ///
    /// # Panics
    ///
    /// Will panic if there is no inner weapon.
    #[must_use]
    pub fn get_inner(&self) -> &dyn PoweruppableWeapon {
        match self {
            WeaponWrapper::Flash(flash) => flash.as_ref().expect("No inner weapon."),
            WeaponWrapper::Pillar(pillar) => pillar.as_ref().expect("No inner weapon."),
            WeaponWrapper::Lightning(lightning) => lightning.as_ref().expect("No inner weapon."),
        }
    }

    /// Get a mutable reference to the inner weapon.
    ///
    /// # Panics
    ///
    /// Will panic if there is no inner weapon.
    pub fn get_inner_mut(&mut self) -> &mut dyn PoweruppableWeapon {
        match self {
            WeaponWrapper::Flash(flash) => flash.as_mut().expect("No inner weapon."),
            WeaponWrapper::Pillar(pillar) => pillar.as_mut().expect("No inner weapon."),
            WeaponWrapper::Lightning(lightning) => lightning.as_mut().expect("No inner weapon."),
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
    pub fn deal_damage(&self, enemies: &mut [Enemy]) {
        for enemy in enemies.iter_mut() {
            if enemy.get_pos().is_in_area(&self.area) {
                enemy.take_damage(self.damage_amount);

                // if was hit by a weapon, do the following
                if let Some(stats) = &self.weapon_stats
                    && !stats.procs.is_empty()
                {
                    stats.procs.iter().for_each(|(_key, proc)| {
                        if proc.chance > 0 {
                            enemy.try_proc(proc);
                        }
                    });
                }
            }
        }
    }
}

/// A trait for any weapon that can be used to attack.
pub trait Weapon {
    /// Creates a `DamageArea` representing the attack.
    fn attack(&self, wielder: &Character, enemies: &[Enemy], layer: &Layer) -> DamageArea;

    /// Calculates and returns the base damage of the weapon.
    ///Damage should be rounded up to nearest int.
    fn get_damage(&self) -> i32;

    fn get_element(&self) -> Option<Elements>;

    fn get_elemental_style(&self) -> Option<Style> {
        self.get_element().map(|element| match element {
            Elements::Flame(_) => Some(Style::new().red()),
            Elements::Shock(_) => Some(Style::new().light_yellow()),
        })?
    }
}
