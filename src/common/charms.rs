#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

#[cfg(target_family = "wasm")]
use web_time::Duration;

use crate::common::{powerup::Poweruppable, upgrade::Stats};

#[derive(Clone)]
pub enum CharmWrapper {
    DamageMult(CharmDamageMult),
    OffsetAdd(CharmOffsetAdd),
    AttackSpeed(CharmAttackSpeed),
}

impl CharmWrapper {
    pub fn get_inner(&self) -> &dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult,
            CharmWrapper::OffsetAdd(offset_add) => offset_add,
            CharmWrapper::AttackSpeed(attack_speed) => attack_speed,
        }
    }

    pub fn get_inner_mut(&mut self) -> &mut dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult,
            CharmWrapper::OffsetAdd(offset_add) => offset_add,
            CharmWrapper::AttackSpeed(attack_speed) => attack_speed,
        }
    }
}

pub trait Charm: Poweruppable {
    /// Manipulate Stats to be increased by this charm's effects. Stats should be reset before calling this method.
    fn manipulate_stats(&self, stats: &mut Stats);
}

#[derive(Clone)]
pub struct CharmDamageMult {
    pub stat_boost: f64,
    pub level: i32,
}

impl CharmDamageMult {
    pub fn new() -> Self {
        let out = Self {
            stat_boost: 1.25,
            level: 1,
        };

        out
    }
}

impl Charm for CharmDamageMult {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.player_stats.damage_mult *= self.stat_boost;
    }
}

impl Poweruppable for CharmDamageMult {
    fn get_powerup_type(&self) -> super::powerup::PowerupTypes {
        super::powerup::PowerupTypes::Charm
    }

    fn get_level(&self) -> i32 {
        self.level
    }

    fn get_name(&self) -> String {
        "Damage Multiplier Charm".into()
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Multiply your Damage Multiplier by 1.25".into(),
            2 => "Increase Damage Mult Mult by 0.25".into(),
            3 => "Increase Damage Mult Mult by 0.5".into(),
            4 => "Increase Damage Mult Mult by 0.75".into(),
            5 => "Increase Damage Mult Mult by 2.0".into(),
            _ => "".into(),
        }
    }

    fn upgrade_self(&mut self, powerup: &super::powerup::DynPowerup) {
        let from = powerup.get_current_level();
        let to = powerup.get_new_level();
        if to <= from {
            return;
        }
        self.level = to;

        for i in (from + 1)..=to {
            match i {
                1 => self.stat_boost = 1.25,
                2 => self.stat_boost += 0.25,
                3 => self.stat_boost += 0.5,
                4 => self.stat_boost += 0.75,
                5 => self.stat_boost += 2.,
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct CharmOffsetAdd {
    pub stat_boost: Duration,
    pub level: i32,
}

impl CharmOffsetAdd {
    pub fn new() -> Self {
        let out = Self {
            stat_boost: Duration::from_secs(1 * 60),
            level: 1,
        };

        out
    }
}

impl Charm for CharmOffsetAdd {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.game_stats.time_offset += self.stat_boost;
    }
}

impl Poweruppable for CharmOffsetAdd {
    fn get_powerup_type(&self) -> super::powerup::PowerupTypes {
        super::powerup::PowerupTypes::Charm
    }

    fn get_level(&self) -> i32 {
        self.level
    }

    fn get_name(&self) -> String {
        "Hype Time Charm".into()
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Set your Hype Time to 1 minutes.".into(),
            2 => "Set your Hype Time to 2 minutes.".into(),
            3 => "Set your Hype Time to 3 minutes.".into(),
            4 => "Set your Hype Time to 4 minutes.".into(),
            5 => "Set your Hype Time to 5 minutes. Be prepared.".into(),
            _ => "".into(),
        }
    }

    fn upgrade_self(&mut self, powerup: &super::powerup::DynPowerup) {
        let from = powerup.get_current_level();
        let to = powerup.get_new_level();
        if to <= from {
            return;
        }
        self.level = to;

        for i in (from + 1)..=to {
            match i {
                1 => self.stat_boost = Duration::from_secs(1 * 60),
                2 => self.stat_boost = Duration::from_secs((1.5 * 60.) as u64),
                3 => self.stat_boost = Duration::from_secs(2 * 60),
                4 => self.stat_boost = Duration::from_secs((2.5 * 60.) as u64),
                5 => self.stat_boost = Duration::from_secs(3 * 60),
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct CharmAttackSpeed {
    pub stat_boost: f64,
    pub level: i32,
}

impl CharmAttackSpeed {
    pub fn new() -> Self {
        let out = Self {
            stat_boost: 1.25,
            level: 1,
        };

        out
    }
}

impl Charm for CharmAttackSpeed {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.game_stats.attack_speed_mult *= self.stat_boost;
    }
}

impl Poweruppable for CharmAttackSpeed {
    fn get_powerup_type(&self) -> super::powerup::PowerupTypes {
        super::powerup::PowerupTypes::Charm
    }

    fn get_level(&self) -> i32 {
        self.level
    }

    fn get_name(&self) -> String {
        "Attack Speed Multiplier Charm".into()
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Multiply your Attack Speed by 1.25".into(),
            2 => "Increase Attack Speed Mult by 0.25".into(),
            3 => "Increase Attack Speed Mult by 0.25".into(),
            4 => "Increase Attack Speed Mult by 0.5".into(),
            5 => "Increase Attack Speed Mult by 0.75".into(),
            _ => "".into(),
        }
    }

    fn upgrade_self(&mut self, powerup: &super::powerup::DynPowerup) {
        let from = powerup.get_current_level();
        let to = powerup.get_new_level();
        if to <= from {
            return;
        }
        self.level = to;

        for i in (from + 1)..=to {
            match i {
                1 => self.stat_boost = 1.25,
                2 => self.stat_boost += 0.25,
                3 => self.stat_boost += 0.25,
                4 => self.stat_boost += 0.5,
                5 => self.stat_boost += 0.75,
                _ => {}
            }
        }
    }
}
