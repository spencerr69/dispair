use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::common::{
    charms::{
        attack_speed::CharmAttackSpeed, damage_mult::CharmDamageMult, hype_time::CharmOffsetAdd,
    },
    powerup::Poweruppable,
    stats::Stats,
};

pub mod attack_speed;
pub mod damage_mult;
pub mod hype_time;

#[derive(Clone, IntoStaticStr, EnumIter, EnumString)]
pub enum CharmWrapper {
    #[strum(
        serialize = "Damage Multiplier Charm",
        serialize = "DAMAGE MULTIPLIER CHARM"
    )]
    DamageMult(Option<CharmDamageMult>),

    #[strum(serialize = "Hype Time Charm", serialize = "HYPE TIME CHARM")]
    OffsetAdd(Option<CharmOffsetAdd>),

    #[strum(serialize = "Attack Speed Charm", serialize = "ATTACK SPEED CHARM")]
    AttackSpeed(Option<CharmAttackSpeed>),
}

impl PartialEq for CharmWrapper {
    fn eq(&self, other: &Self) -> bool {
        let self_name: &'static str = self.into();
        let other_name: &'static str = other.into();
        self_name == other_name
    }
}

impl CharmWrapper {
    /// Get a reference to the inner weapon.
    ///
    /// # Panics
    ///
    /// Will panic if there is no inner weapon.
    #[must_use]
    pub fn get_inner(&self) -> &dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult.as_ref().unwrap(),
            CharmWrapper::OffsetAdd(offset_add) => offset_add.as_ref().unwrap(),
            CharmWrapper::AttackSpeed(attack_speed) => attack_speed.as_ref().unwrap(),
        }
    }
    /// Get a mutable reference to the inner weapon.
    ///
    /// # Panics
    ///
    /// Will panic if there is no inner weapon.
    pub fn get_inner_mut(&mut self) -> &mut dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult.as_mut().unwrap(),
            CharmWrapper::OffsetAdd(offset_add) => offset_add.as_mut().unwrap(),
            CharmWrapper::AttackSpeed(attack_speed) => attack_speed.as_mut().unwrap(),
        }
    }

    pub fn populate_inner(&mut self) {
        match self {
            CharmWrapper::DamageMult(damage_mult) => *damage_mult = Some(CharmDamageMult::new()),
            CharmWrapper::OffsetAdd(offset_add) => *offset_add = Some(CharmOffsetAdd::new()),
            CharmWrapper::AttackSpeed(attack_speed) => {
                *attack_speed = Some(CharmAttackSpeed::new());
            }
        }
    }
}

pub trait Charm: Poweruppable {
    /// Manipulate Stats to be increased by this charm's effects. Stats should be reset before calling this method.
    fn manipulate_stats(&self, stats: &mut Stats);
}
