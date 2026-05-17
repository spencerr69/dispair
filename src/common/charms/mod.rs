use strum::{EnumIter, EnumString, IntoStaticStr};

use crate::common::{
    PlayerStateRef,
    charms::{
        attack_speed::CharmAttackSpeed, damage_mult::CharmDamageMult, hype_time::CharmHypeTime,
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
    HypeTime(Option<CharmHypeTime>),

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
            CharmWrapper::DamageMult(damage_mult) => damage_mult.as_ref().expect("No inner charm."),
            CharmWrapper::HypeTime(offset_add) => offset_add.as_ref().expect("No inner charm."),
            CharmWrapper::AttackSpeed(attack_speed) => {
                attack_speed.as_ref().expect("No inner charm.")
            }
        }
    }
    /// Get a mutable reference to the inner weapon.
    ///
    /// # Panics
    ///
    /// Will panic if there is no inner weapon.
    pub fn get_inner_mut(&mut self) -> &mut dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult.as_mut().expect("No inner charm."),
            CharmWrapper::HypeTime(offset_add) => offset_add.as_mut().expect("No inner charm."),
            CharmWrapper::AttackSpeed(attack_speed) => {
                attack_speed.as_mut().expect("No inner charm.")
            }
        }
    }

    pub fn populate_inner(&mut self, player_state_ref: PlayerStateRef) {
        match self {
            CharmWrapper::DamageMult(damage_mult) => {
                *damage_mult = Some(CharmDamageMult::new(player_state_ref))
            }
            CharmWrapper::HypeTime(offset_add) => {
                *offset_add = Some(CharmHypeTime::new(player_state_ref))
            }
            CharmWrapper::AttackSpeed(attack_speed) => {
                *attack_speed = Some(CharmAttackSpeed::new(player_state_ref));
            }
        }
    }
}

pub trait Charm: Poweruppable {
    /// Manipulate Stats to be increased by this charm's effects. Stats should be reset before calling this method.
    fn manipulate_stats(&self, stats: &mut Stats);
}
