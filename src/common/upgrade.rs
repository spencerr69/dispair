//! This module defines the data structures for player state, upgrades, and stats.
//! It includes logic for applying upgrades and calculating player stats.

use std::{collections::HashMap, ops::Sub};

#[cfg(not(target_family = "wasm"))]
use std::time::Duration;
#[cfg(target_family = "wasm")]
use web_time::Duration;

use derive_more::Sub;

use serde::{Deserialize, Serialize};

use crate::common::enemy::{Debuff, DebuffTypes};

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
    ///
    /// # Examples
    ///
    /// ```
    /// let mut a = PlayerState::default();
    /// a.inventory = Inventory { gold: 10 };
    /// let mut b = PlayerState::default();
    /// b.inventory = Inventory { gold: 4 };
    ///
    /// let diff = a - b;
    /// assert_eq!(diff.inventory.gold, 6);
    /// ```
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
            weapon_stats.damage_flat_boost += 1 * self.amount_owned("211") as i32;
        }

        //upgrade 212 damage/mult_up
        if self.upgrade_owned("212") {
            player_stats.damage_mult += 0.1 * self.amount_owned("212") as f64;
        }

        //upgrade 221 health/flat_up
        if self.upgrade_owned("221") {
            player_stats.base_health += 1 * self.amount_owned("221") as i32;
        }

        //upgrade 222 health/mult_up
        if self.upgrade_owned("222") {
            player_stats.health_mult += 0.1 * self.amount_owned("222") as f64;
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
            player_stats.movement_speed_mult =
                player_stats.movement_speed_mult + (0.5 * self.amount_owned("25") as f64)
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
                        },
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
            player_stats.shove_amount += 1 * self.amount_owned("321");
        }

        //upgrade 322 shove damage
        if self.upgrade_owned("322") {
            player_stats.shove_damage += 1 * self.amount_owned("322");
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
            game_stats.width = 400;
            game_stats.height = 400;
            weapon_stats.size *= 6;
            game_stats.enemy_spawn_mult = 12.;
            game_stats.enemy_move_mult = 3.;
            player_stats.base_health = 10000;
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
        self.upgrades.get(id).unwrap_or(&0).clone()
    }

    /// Checks if the player owns at least one of a specific upgrade.
    pub fn upgrade_owned(&self, id: &str) -> bool {
        self.upgrades.get(id).unwrap_or(&0).clone() > 0
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
        self.children.is_some() && self.children.clone().unwrap().len() > 0
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

/// Represents the player's inventory.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Inventory {
    /// The amount of gold the player has.
    pub gold: u32,
}

impl Sub for Inventory {
    type Output = Inventory;

    fn sub(self, other: Inventory) -> Self::Output {
        Inventory {
            gold: self.gold.saturating_sub(other.gold),
        }
    }
}

impl Inventory {
    /// Adds a specified amount of gold to the player's inventory.
    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }
}

/// Represents the player's stats.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Stats {
    pub game_stats: GameStats,
    pub player_stats: PlayerStats,
    pub weapon_stats: WeaponStats,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStats {
    pub enemy_spawn_mult: f64,
    pub enemy_move_mult: f64,
    pub attack_speed_mult: f64,
    pub gold_mult: f64,
    pub width: usize,
    pub height: usize,

    pub timer: u64,
    pub time_offset: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, Sub)]
pub struct PlayerStats {
    pub base_health: i32,
    pub health_mult: f64,

    pub health: i32,

    pub damage_mult: f64,

    pub shove_amount: u32,
    pub shove_damage: u32,

    pub movement_speed_mult: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponStats {
    pub damage_flat_boost: i32,

    pub procs: HashMap<String, Proc>,

    pub size: i32,

    pub level: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DebuffStats {
    pub size: Option<i32>,
    pub damage: Option<i32>,
    pub misc_value: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proc {
    /// Chance is a int between 0-100.
    pub chance: u32,
    pub debuff: Debuff,
}

impl Default for GameStats {
    /// Baseline game-level modifiers used when no upgrades are applied.
    ///
    /// The defaults are:
    /// - `enemy_spawn_mult = 1.0`
    /// - `enemy_move_mult = 1.0`
    /// - `attack_speed_mult = 1.0`
    /// - `gold_mult = 1.0`
    /// - `width = 20`
    /// - `height = 6`
    /// - `timer = 60`
    /// - `time_offset = 0s`
    ///
    /// # Examples
    ///
    /// ```
    /// let gs = GameStats::default();
    /// assert_eq!(gs.timer, 60);
    /// assert_eq!(gs.width, 20);
    /// assert_eq!(gs.enemy_spawn_mult, 1.0);
    /// ```
    fn default() -> Self {
        Self {
            enemy_spawn_mult: 1.,
            enemy_move_mult: 1.,
            attack_speed_mult: 1.,
            gold_mult: 1.,
            height: 6,
            width: 20,
            time_offset: Duration::from_secs(0),
            timer: 60,
        }
    }
}

impl Default for PlayerStats {
    /// Constructs a PlayerStats with baseline health, damage, movement, and shove defaults.
    ///
    /// Fields are initialized to:
    /// - base_health: 10
    /// - health: 10
    /// - health_mult: 1.0
    /// - damage_mult: 1.0
    /// - movement_speed_mult: 1.0
    /// - shove_amount: 0
    /// - shove_damage: 0
    ///
    /// # Examples
    ///
    /// ```
    /// let stats = PlayerStats::default();
    /// assert_eq!(stats.base_health, 10);
    /// assert_eq!(stats.health, 10);
    /// assert_eq!(stats.health_mult, 1.0);
    /// assert_eq!(stats.damage_mult, 1.0);
    /// assert_eq!(stats.movement_speed_mult, 1.0);
    /// assert_eq!(stats.shove_amount, 0);
    /// assert_eq!(stats.shove_damage, 0);
    /// ```
    fn default() -> Self {
        Self {
            base_health: 10,
            damage_mult: 1.,
            health: 10,
            health_mult: 1.,
            movement_speed_mult: 1.,
            shove_amount: 0,
            shove_damage: 0,
        }
    }
}

impl Default for WeaponStats {
    /// Creates a default `WeaponStats` with zero damage boost, zero size, and no procs.
    ///
    /// # Examples
    ///
    /// ```
    /// let ws = WeaponStats::default();
    /// assert_eq!(ws.damage_flat_boost, 0);
    /// assert_eq!(ws.size, 0);
    /// assert!(ws.procs.is_empty());
    /// ```
    fn default() -> Self {
        Self {
            damage_flat_boost: 0,
            size: 0,
            procs: HashMap::new(),
            level: 1,
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
