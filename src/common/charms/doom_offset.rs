use crate::common::upgrades::upgrade::PlayerState;
use crate::common::{
    PlayerStateRef,
    charms::Charm,
    powerup::{DynPowerup, PowerupTypes, Poweruppable},
    stats::Stats,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct CharmDoomOffset {
    pub stat_boost: f64,
    pub level: i32,
    pub player_state: PlayerStateRef,
}

impl CharmDoomOffset {
    #[must_use]
    pub fn new(player_state_ref: PlayerStateRef) -> Self {
        Self {
            stat_boost: 1.,
            level: 1,
            player_state: player_state_ref,
        }
    }
}

impl Default for CharmDoomOffset {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(PlayerState::default())))
    }
}

impl Charm for CharmDoomOffset {
    fn manipulate_stats(&self, stats: &mut Stats) {
        stats.game_stats.doom_offset += self.stat_boost;
    }
}

impl Poweruppable for CharmDoomOffset {
    fn get_max_level(&self) -> i32 {
        self.player_state.borrow().stats.game_stats.max_charm_level
    }

    fn get_name(&self) -> String {
        "Doom Offset Charm".into()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Charm
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "Offset Doom by 2 (increase difficulty).".into(),
            2 => "Offset Doom by 4.".into(),
            3 => "Offset Doom by 8.".into(),
            4 => "Offset Doom by 16.".into(),
            5 => "Offset Doom by 32. Be prepared.".into(),
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
                1 => self.stat_boost = 2.,
                2 => self.stat_boost = 4.,
                3 => self.stat_boost = 8.,
                4 => self.stat_boost = 16.,
                5 => self.stat_boost = 32.,
                _ => {}
            }
        }
    }

    fn get_level(&self) -> i32 {
        self.level
    }
}
