use crate::common::{powerup::Poweruppable, upgrade::Stats};

pub trait Charm {
    /// Manipulate Stats to be increased by this charm's effects. Stats should be reset before calling this method.
    fn manipulate_stats(&self, stats: &mut Stats);
}

pub struct CharmDamageMult {
    pub stat_boost: f64,
    pub level: i32,
}

impl CharmDamageMult {
    pub fn new(stat_boost: f64, level: i32) -> Self {
        Self { stat_boost, level }
    }
}

impl Charm for CharmDamageMult {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.player_stats.damage_mult *= self.stat_boost;
    }
}

impl Poweruppable for CharmDamageMult {
    fn get_level(&self) -> i32 {
        self.level
    }

    fn get_name(&self) -> String {
        "Damage Multiplier".into()
    }
    
    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Multiply your Damage Multiplier by ".into()
        }
    }
    
    fn upgrade_self(&mut self, powerup: &super::powerup::DynPowerup) {
        
    }
}

pub enum CharmWrapper {
    DamageMult(CharmDamageMult),
    OffsetAdd(CharmOffsetAdd),
    AttackSpeed(CharmAttackSpeed),
}

impl CharmWrapper {
    fn get_inner(&self) -> &dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult,
            CharmWrapper::OffsetAdd(offset_add) => offset_add,
            CharmWrapper::AttackSpeed(attack_speed) => attack_speed,
        }
    }

    fn get_inner_mut(&mut self) -> &mut dyn Charm {
        match self {
            CharmWrapper::DamageMult(damage_mult) => damage_mult,
            CharmWrapper::OffsetAdd(offset_add) => offset_add,
            CharmWrapper::AttackSpeed(attack_speed) => attack_speed,
        }
    }
}
