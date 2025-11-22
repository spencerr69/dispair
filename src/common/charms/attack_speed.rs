use crate::common::{
    charms::Charm,
    powerup::{DynPowerup, PowerupTypes, Poweruppable},
    stats::Stats,
};

#[derive(Clone)]
pub struct CharmAttackSpeed {
    pub stat_boost: f64,
    pub level: i32,
}

impl CharmAttackSpeed {
    pub fn new() -> Self {
        Self {
            stat_boost: 1.25,
            level: 1,
        }
    }
}

impl Default for CharmAttackSpeed {
    fn default() -> Self {
        Self::new()
    }
}

impl Charm for CharmAttackSpeed {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.game_stats.attack_speed_mult *= self.stat_boost;
    }
}

impl Poweruppable for CharmAttackSpeed {
    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Charm
    }

    fn get_level(&self) -> i32 {
        self.level
    }

    fn get_name(&self) -> String {
        "Attack Speed Charm".into()
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
                3 => self.stat_boost += 0.25,
                4 => self.stat_boost += 0.5,
                5 => self.stat_boost += 0.75,
                _ => {}
            }
        }
    }
}
