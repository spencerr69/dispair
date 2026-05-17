use crate::common::upgrades::upgrade::PlayerState;
use crate::common::{
    PlayerStateRef,
    charms::Charm,
    powerup::{DynPowerup, PowerupTypes, Poweruppable},
    stats::Stats,
};
use crate::prelude::Duration;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct CharmHypeTime {
    pub stat_boost: Duration,
    pub level: i32,
    pub player_state: PlayerStateRef,
}

impl CharmHypeTime {
    #[must_use]
    pub fn new(player_state_ref: PlayerStateRef) -> Self {
        Self {
            stat_boost: Duration::from_secs(60),
            level: 1,
            player_state: player_state_ref,
        }
    }
}

impl Default for CharmHypeTime {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(PlayerState::default())))
    }
}

impl Charm for CharmHypeTime {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.game_stats.time_offset += self.stat_boost;
    }
}

impl Poweruppable for CharmHypeTime {
    fn get_max_level(&self) -> i32 {
        self.player_state.borrow().stats.game_stats.max_charm_level
    }

    fn get_name(&self) -> String {
        "Hype Time Charm".into()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Charm
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Set your Hype Time to 1 minutes.".into(),
            2 => "Set your Hype Time to 1.5 minutes.".into(),
            3 => "Set your Hype Time to 2.5 minutes.".into(),
            4 => "Set your Hype Time to 3.5 minutes.".into(),
            5 => "Set your Hype Time to 5 minutes. Be prepared.".into(),
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
                1 => self.stat_boost = Duration::from_secs(60),
                2 => self.stat_boost = Duration::from_secs((1.5 * 60.) as u64),
                3 => self.stat_boost = Duration::from_secs((2.5 * 60.) as u64),
                4 => self.stat_boost = Duration::from_secs((3.5 * 60.) as u64),
                5 => self.stat_boost = Duration::from_secs(5 * 60),
                _ => {}
            }
        }
    }

    fn get_level(&self) -> i32 {
        self.level
    }
}
