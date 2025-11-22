use crate::common::{
    charms::Charm,
    powerup::{DynPowerup, PowerupTypes, Poweruppable},
    stats::Stats,
};

#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

#[cfg(target_family = "wasm")]
use web_time::Duration;

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
    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Charm
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

    fn upgrade_self(&mut self, powerup: &DynPowerup) {
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
