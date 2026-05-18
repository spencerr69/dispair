//! This module defines weapons, damage areas, and their interactions in the game.
//! It includes a `Weapon` trait, a `Sword` implementation, and a `DamageArea` struct
//! for handling attacks and their effects on enemies.

use crate::{common::debuffs::Elements, prelude::Duration};

use ratatui::style::Style;
use std::cmp::Ordering;
use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::common::character::{CharacterPositionData, Renderable};
use crate::common::coords::{AreaWrapper, ChaosArea};
use crate::common::enemies::enemy::{Debuffable, Enemy};
use crate::common::entities::EntityCharacters;
use crate::common::map::Layer;

use crate::common::{
    PlayerStateRef, character::Damageable, powerup::PoweruppableWeapon, stats::WeaponStats,
};

pub mod flash;
pub mod lightning;
pub mod pillar;
pub mod row;

#[macro_export]
macro_rules! new_weapon {
    ($weapon_name: ident, $base_damage:expr, $base_size:expr, $base_cooldown:expr ) => {
        #[derive(Clone)]
        pub struct $weapon_name {
            base_damage: i32,
            damage_scalar: f64,
            stats: WeaponStats,
            element: Option<Elements>,
            cooldown_ticks: u64,
            player_state: PlayerStateRef,
        }

        impl $weapon_name {
            const BASE_DAMAGE: i32 = $base_damage;
            const BASE_SIZE: i32 = $base_size;
            const BASE_COOLDOWN: u64 = $base_cooldown;

            #[doc = concat!("Creates a new `", stringify!($weapon_name), "` with stats based on \
            the \
            player's \
            current `Stats`.")]
            #[must_use]
            pub fn new(base_weapon_stats: WeaponStats, player_state: PlayerStateRef) -> Self {
                Self {
                    base_damage: Self::BASE_DAMAGE + base_weapon_stats.damage_flat_boost,
                    damage_scalar: 1.,
                    cooldown_ticks: 0,
                    stats: WeaponStats {
                        size: Self::BASE_SIZE + base_weapon_stats.size,
                        ..base_weapon_stats
                    },
                    element: None,
                    player_state,
                }
            }
        }
    };
}

#[derive(Clone, EnumIter, IntoStaticStr, EnumString)]
pub enum WeaponWrapper {
    #[strum(serialize = "Flash", serialize = "FLASH")]
    Flash(Option<flash::Flash>),

    #[strum(serialize = "Pillar", serialize = "PILLAR")]
    Pillar(Option<pillar::Pillar>),

    #[strum(serialize = "Lightning", serialize = "LIGHTNING")]
    Lightning(Option<lightning::Lightning>),

    #[strum(serialize = "Row", serialize = "ROW")]
    Row(Option<row::Row>),
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
            WeaponWrapper::Row(row) => row.as_ref().expect("No inner weapon."),
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
            WeaponWrapper::Row(row) => row.as_mut().expect("No inner weapon."),
        }
    }

    pub fn populate_inner(&mut self, weapon_stats: WeaponStats, player_state: PlayerStateRef) {
        match self {
            WeaponWrapper::Flash(flash) => {
                *flash = Some(flash::Flash::new(weapon_stats, player_state))
            }
            WeaponWrapper::Pillar(pillar) => {
                *pillar = Some(pillar::Pillar::new(weapon_stats, player_state));
            }
            WeaponWrapper::Lightning(lightning) => {
                *lightning = Some(lightning::Lightning::new(weapon_stats, player_state));
            }
            WeaponWrapper::Row(row) => *row = Some(row::Row::new(weapon_stats, player_state)),
        }
    }

    #[must_use]
    pub fn get_damage(&self) -> i32 {
        self.get_inner().get_damage()
    }
}

/// Represents an area where damage is applied, created by a weapon attack.
#[derive(Clone)]
pub struct DamageArea {
    pub damage_amount: i32,
    pub area: AreaWrapper,
    pub entity: EntityCharacters,
    pub duration: Duration,
    pub blink: bool,
    pub weapon_stats: Option<WeaponStats>,
}

impl DamageArea {
    pub fn new_empty() -> Self {
        DamageArea {
            damage_amount: 0,
            area: AreaWrapper::Chaos(ChaosArea::new(vec![])),
            duration: Duration::from_secs_f32(0.),
            entity: EntityCharacters::Empty(Style::new()),
            blink: false,
            weapon_stats: None,
        }
    }

    /// Applies this damage area to every enemy whose position lies inside the area.
    ///
    /// For each affected enemy, reduces its health by `damage_amount`. If `weapon_stats` is present,
    /// iterates its `procs` and invokes each proc with `chance > 0` on the enemy.
    pub fn deal_damage(&self, enemies: &mut [Enemy]) {
        for enemy in enemies.iter_mut() {
            if enemy.get_pos().is_in_area(self.area.get_inner()) {
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

#[must_use]
pub fn get_strongest_weapon(weapons: &[WeaponWrapper]) -> Option<&WeaponWrapper> {
    weapons.iter().max_by(|a, b| {
        if a.get_damage() > b.get_damage() {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    })
}

/// A trait for any weapon that can be used to attack.
pub trait Weapon {
    /// Creates a `DamageArea` representing the attack.
    fn attack(
        &mut self,
        wielder: CharacterPositionData,
        enemies: &[Enemy],
        layer: &Layer,
    ) -> DamageArea;

    /// Calculates and returns the base damage of the weapon.
    ///Damage should be rounded up to the nearest int.
    fn get_damage(&self) -> i32;

    fn get_element(&self) -> Option<Elements>;

    fn get_elemental_style(&self) -> Option<Style> {
        self.get_element().map(|element| match element {
            Elements::Flame(_) => Some(Style::new().red()),
            Elements::Shock(_) => Some(Style::new().light_yellow()),
        })?
    }
}
