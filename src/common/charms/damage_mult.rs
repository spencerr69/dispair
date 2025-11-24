use crate::common::{
    charms::Charm,
    powerup::{DynPowerup, PowerupTypes, Poweruppable},
    stats::Stats,
};

#[derive(Clone)]
pub struct CharmDamageMult {
    pub stat_boost: f64,
    pub level: i32,
}

impl CharmDamageMult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stat_boost: 1.25,
            level: 1,
        }
    }
}

impl Default for CharmDamageMult {
    fn default() -> Self {
        Self::new()
    }
}

impl Charm for CharmDamageMult {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.player_stats.damage_mult *= self.stat_boost;
    }
}

impl Poweruppable for CharmDamageMult {
    fn get_name(&self) -> String {
        "Damage Multiplier Charm".into()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Charm
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Multiply your Damage Multiplier by 1.25".into(),
            2 => "Increase Damage Mult Mult by 0.25".into(),
            3 => "Increase Damage Mult Mult by 0.5".into(),
            4 => "Increase Damage Mult Mult by 0.75".into(),
            5 => "Increase Damage Mult Mult by 2.0".into(),
            _ => String::new(),
        }
    }

    fn upgrade_self(&mut self, powerup: &DynPowerup) {
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

    fn get_level(&self) -> i32 {
        self.level
    }
}
