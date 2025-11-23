//! This module defines the data structures for player state, upgrades, and stats.
//! It includes logic for applying upgrades and calculating player stats.

use std::{collections::HashMap, ops::Sub};

use crate::target_types::Duration;

use serde::{Deserialize, Serialize};

use crate::common::{
    debuffs::{Debuff, DebuffTypes},
    stats::{DebuffStats, GameStats, Inventory, PlayerStats, Proc, Stats, WeaponStats},
};

/// Represents the complete state of the player, including upgrades, inventory, and stats.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub upgrades: CurrentUpgrades,
    pub inventory: Inventory,
    pub stats: Stats,
}

/// Represents the difference between two `PlayerState` instances.
pub struct PlayerStateDiff {
    /// The difference in the player's inventory.
    pub inventory: Inventory,
}

impl Sub for PlayerState {
    type Output = PlayerStateDiff;

    /// Compute the difference between two player states, producing a PlayerStateDiff that contains the inventory delta.
    ///
    /// The resulting `PlayerStateDiff`'s `inventory` equals `self.inventory - other.inventory`.
    fn sub(self, other: PlayerState) -> Self::Output {
        PlayerStateDiff {
            inventory: self.inventory - other.inventory,
        }
    }
}

impl PlayerState {
    /// Refreshes the player's stats based on their current upgrades.
    pub fn refresh(&mut self) {
        let mut game_stats = GameStats::default();
        let mut player_stats = PlayerStats::default();
        let mut weapon_stats = WeaponStats::default();

        //upgrades 1 PRESERVE
        //upgrade 11: PRESERVE::\conform
        if !self.upgrade_owned("11") {
            game_stats.enemy_spawn_mult = 50.;
            game_stats.timer = 10;
        }

        //upgrade 12 grow
        if self.upgrade_owned("12") {
            weapon_stats.size += 1;
        }

        //upgrade 13 become
        if self.upgrade_owned("13") {
            game_stats.enemy_spawn_mult += 0.8;
            game_stats.height += 5;
            game_stats.width += 5;
        }

        //upgrades 2 STATS
        //upgrade 211 damage/flat_up
        if self.upgrade_owned("211") {
            weapon_stats.damage_flat_boost += self.amount_owned("211") as i32;
        }

        //upgrade 212 damage/mult_up
        if self.upgrade_owned("212") {
            player_stats.damage_mult += 0.1 * self.amount_owned("212") as f64;
        }

        //upgrade 221 health/flat_up
        if self.upgrade_owned("221") {
            player_stats.base_health += self.amount_owned("221") as i32;
        }

        //upgrade 222 health/mult_up
        if self.upgrade_owned("222") {
            player_stats.health_mult += 0.1 * self.amount_owned("222") as f64;
        }

        //upgrade 223 health/exp_mult_up
        if self.upgrade_owned("223") {
            player_stats.health_mult *= 1.5 * self.amount_owned("223") as f64;
        }

        //upgrade 23 attack_rate
        if self.upgrade_owned("23") {
            game_stats.attack_speed_mult += 0.15 * self.amount_owned("23") as f64;
        }

        //upgrade 24 timer_length
        if self.upgrade_owned("24") {
            game_stats.timer =
                (game_stats.timer as f64 * (1.5 * self.amount_owned("24") as f64)).ceil() as u64;
        }

        //upgrade 25 movement_speed
        if self.upgrade_owned("25") {
            player_stats.movement_speed_mult += 0.5 * self.amount_owned("25") as f64
        }
        //upgrade 26 gold_gain
        if self.upgrade_owned("26") {
            game_stats.gold_mult += 0.5 * self.amount_owned("26") as f64
        }
        //upgrade 27 elemental_honage
        if self.upgrade_owned("27") {
            weapon_stats.elemental_honage += 0.25 * self.amount_owned("27") as f64
        }

        //upgrade 31 MARK
        //upgrade 311 mark chance
        if self.upgrade_owned("311") {
            weapon_stats.procs.insert(
                "mark".into(),
                Proc {
                    chance: 2 * self.amount_owned("311"),

                    debuff: Debuff {
                        stats: DebuffStats {
                            size: Some(1),
                            damage: Some(6),
                            misc_value: None,
                            on_death_effect: true,
                            on_tick_effect: false,
                            on_damage_effect: false,
                        },
                        complete: false,
                        debuff_type: DebuffTypes::MarkedForExplosion,
                    },
                },
            );
        }

        //upgrade 312 mark size
        if self.upgrade_owned("312") {
            weapon_stats
                .procs
                .get_mut("mark")
                .unwrap()
                .debuff
                .stats
                .size = Some(1 + self.amount_owned("312") as i32);
        }

        //upgrade 32 shove
        //upgrade 321 shove amount
        if self.upgrade_owned("321") {
            player_stats.shove_amount += self.amount_owned("321");
        }

        //upgrade 322 shove damage
        if self.upgrade_owned("322") {
            player_stats.shove_damage += self.amount_owned("322");
        }

        // upgrade 4 GREED
        // upgrade 41 hype
        if self.upgrade_owned("41") {
            game_stats.time_offset += Duration::from_secs((30 * self.amount_owned("41")).into());
        }

        // upgrade 42 growth
        if self.upgrade_owned("42") {
            let amount_owned = self.amount_owned("42");
            let growth_amount = 2 * amount_owned;

            game_stats.width += growth_amount as usize;
            game_stats.height += growth_amount as usize;
            game_stats.enemy_spawn_mult += 0.5 * amount_owned as f64
        }

        if self.upgrade_owned("51") {
            let amount_owned = self.amount_owned("51");
            let growth_amount = 50 * amount_owned;

            game_stats.width += growth_amount as usize;
            game_stats.enemy_spawn_mult += 1.5 * amount_owned as f64;

            game_stats.gold_mult += 0.3 * amount_owned as f64;
            game_stats.enemy_move_mult += 0.05 * amount_owned as f64;
        }

        if self.upgrade_owned("52") {
            let amount_owned = self.amount_owned("52");
            let growth_amount = 50 * amount_owned;

            game_stats.height += growth_amount as usize;
            game_stats.enemy_spawn_mult += 1.5 * amount_owned as f64;
            game_stats.gold_mult += 0.3 * amount_owned as f64;
            game_stats.enemy_move_mult += 0.05 * amount_owned as f64;
        }

        //debug
        #[cfg(debug_assertions)]
        if self.upgrade_owned("9999") {
            game_stats.width = 100;
            game_stats.height = 100;
            player_stats.base_health = 10000;
            game_stats.time_offset = Duration::from_secs(60);
            self.inventory.add_gold(100000);
        }

        //cleanups
        player_stats.health =
            (player_stats.base_health as f64 * player_stats.health_mult).ceil() as i32;

        self.stats = Stats {
            game_stats,
            player_stats,
            weapon_stats,
        }
    }

    /// Returns the number of times an upgrade has been purchased.
    pub fn amount_owned(&self, id: &str) -> u32 {
        *self.upgrades.get(id).unwrap_or(&0)
    }

    /// Checks if the player owns at least one of a specific upgrade.
    pub fn upgrade_owned(&self, id: &str) -> bool {
        *self.upgrades.get(id).unwrap_or(&0) > 0
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        let upgrade_tree = get_upgrade_tree().unwrap();

        let mut out = Self {
            inventory: Inventory::default(),
            stats: Stats::default(),
            upgrades: get_current_upgrades(upgrade_tree, HashMap::new()),
        };

        out.refresh();

        out
    }
}

/// A type alias for a map of upgrade IDs to the number of times they have been purchased.
pub type CurrentUpgrades = HashMap<String, u32>;

/// Represents a single node in the upgrade tree.
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct UpgradeNode {
    pub title: String,
    pub description: String,
    pub id: String,
    pub cost: Option<u32>,
    pub children: Option<Vec<UpgradeNode>>,
    pub limit: u32,
    pub requires: Vec<String>,
    pub costscale_override: Option<f64>,
}

impl UpgradeNode {
    /// Checks if the upgrade node has any children.
    pub fn has_children(&self) -> bool {
        self.children.is_some() && !self.children.clone().unwrap().is_empty()
    }

    /// Returns the display title for the upgrade.
    pub fn get_display_title(&self) -> String {
        if self.children.is_some() {
            " > ".to_string() + self.title.as_str()
        } else {
            " ".to_string() + self.title.as_str()
        }
    }

    /// Calculates the cost of the next purchase of this upgrade.
    pub fn next_cost(&self, amount_owned: u32) -> u32 {
        let mut costscale = 1.2;
        if let Some(over_ride) = self.costscale_override {
            costscale = over_ride;
        }

        if self.cost.is_none() {
            0
        } else {
            (self.cost.unwrap() as f64 * (costscale.powf(amount_owned as f64))).ceil() as u32
        }
    }
}

/// A type alias for a vector of `UpgradeNode`s, representing the entire upgrade tree.
pub type UpgradeTree = Vec<UpgradeNode>;

/// Loads the upgrade tree from the `upgrades.json` file.
pub fn get_upgrade_tree() -> Result<Vec<UpgradeNode>, serde_json::Error> {
    let upgrade_tree: UpgradeTree = serde_json::from_str(include_str!("upgrades.json"))?;

    #[cfg(not(debug_assertions))]
    let upgrade_tree = upgrade_tree
        .into_iter()
        .filter(|node| node.id != "9999")
        .collect();

    Ok(upgrade_tree)
}

/// Recursively traverses the upgrade tree and creates a map of all possible upgrades, initialized to 0.
pub fn get_current_upgrades(
    upgrade_tree: UpgradeTree,
    mut acc: CurrentUpgrades,
) -> CurrentUpgrades {
    upgrade_tree.into_iter().for_each(|node| {
        acc.insert(node.id, 0);
        if let Some(children) = node.children {
            acc = get_current_upgrades(children, acc.clone());
        }
    });

    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correctly() {
        let upgrade_tree = get_upgrade_tree().unwrap();
        assert!(upgrade_tree[0].title.len() > 1)
    }

    #[test]
    fn current_upgrades_check() {
        let upgrade_tree = get_upgrade_tree().unwrap();
        let current_upgrades = get_current_upgrades(upgrade_tree, HashMap::new());
        println!("Current upgrades: {:?}", current_upgrades);
        assert!(!current_upgrades.is_empty());
    }
}
