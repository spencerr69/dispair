use std::{collections::HashMap, ops::Sub};

#[cfg(not(target_family = "wasm"))]
use std::time::Duration;
#[cfg(target_family = "wasm")]
use web_time::Duration;

use derive_more::Sub;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub upgrades: CurrentUpgrades,
    pub inventory: Inventory,
    pub stats: Stats,
}

pub struct PlayerStateDiff {
    pub inventory: Inventory,
    pub stats: Stats,
}

impl Sub for PlayerState {
    type Output = PlayerStateDiff;

    fn sub(self, other: PlayerState) -> Self::Output {
        PlayerStateDiff {
            inventory: self.inventory - other.inventory,
            stats: self.stats - other.stats,
        }
    }
}

impl PlayerState {
    pub fn refresh(&mut self) {
        self.stats = Stats::default();

        //upgrades 1 PRESERVE
        //upgrade 11: PRESERVE::\conform
        if !self.upgrade_owned("11") {
            self.stats.enemy_spawn_mult = 50.;
            self.stats.timer = 10;
        }

        //upgrade 12 grow
        if self.upgrade_owned("12") {
            self.stats.size += 1;
        }

        //upgrade 13 become
        if self.upgrade_owned("13") {
            self.stats.enemy_spawn_mult += 0.8;
            self.stats.height += 5;
            self.stats.width += 5;
        }

        //upgrades 2 STATS
        //upgrade 211 damage/flat_up
        if self.upgrade_owned("211") {
            self.stats.damage_flat_boost += 1 * self.amount_owned("211") as i32;
        }

        //upgrade 212 damage/mult_up
        if self.upgrade_owned("212") {
            self.stats.damage_mult += 0.1 * self.amount_owned("212") as f64;
        }

        //upgrade 221 health/flat_up
        if self.upgrade_owned("221") {
            self.stats.base_health += 1 * self.amount_owned("221") as i32;
        }

        //upgrade 222 health/mult_up
        if self.upgrade_owned("222") {
            self.stats.health_mult += 0.1 * self.amount_owned("222") as f64;
        }

        //upgrade 23 attack_rate
        if self.upgrade_owned("23") {
            self.stats.attack_speed_mult += 0.15 * self.amount_owned("23") as f64;
        }

        //upgrade 24 timer_length
        if self.upgrade_owned("24") {
            self.stats.timer =
                (self.stats.timer as f64 * (1.5 * self.amount_owned("24") as f64)).ceil() as u64;
        }

        //upgrade 25 movement_speed
        if self.upgrade_owned("25") {
            self.stats.movement_speed_mult =
                self.stats.movement_speed_mult + (0.5 * self.amount_owned("25") as f64)
        }

        //upgrade 31 MARK
        //upgrade 311 mark chance
        if self.upgrade_owned("311") {
            self.stats.mark_chance += 2 * self.amount_owned("311");
        }

        //upgrade 312 mark size
        if self.upgrade_owned("312") {
            self.stats.mark_explosion_size += 1 * self.amount_owned("312");
        }

        //upgrade 32 shove
        //upgrade 321 shove amount
        if self.upgrade_owned("321") {
            self.stats.shove_amount += 1 * self.amount_owned("321");
        }

        //upgrade 322 shove damage
        if self.upgrade_owned("322") {
            self.stats.shove_damage += 1 * self.amount_owned("322");
        }

        // upgrade 4 GREED
        // upgrade 41 hype
        if self.upgrade_owned("41") {
            self.stats.time_offset += Duration::from_secs((30 * self.amount_owned("41")).into());
        }

        // upgrade 42 growth
        if self.upgrade_owned("42") {
            let amount_owned = self.amount_owned("42");
            let growth_amount = 2 * amount_owned;

            self.stats.width += growth_amount as usize;
            self.stats.height += growth_amount as usize;
            self.stats.enemy_spawn_mult += 0.5 * amount_owned as f64
        }

        if self.upgrade_owned("51") {
            let amount_owned = self.amount_owned("51");
            let growth_amount = 50 * amount_owned;

            self.stats.width += growth_amount as usize;
            self.stats.enemy_spawn_mult += 1.5 * amount_owned as f64;

            self.stats.gold_mult += 0.3 * amount_owned as f64;
            self.stats.enemy_move_mult += 0.05 * amount_owned as f64;
        }

        if self.upgrade_owned("52") {
            let amount_owned = self.amount_owned("52");
            let growth_amount = 50 * amount_owned;

            self.stats.height += growth_amount as usize;
            self.stats.enemy_spawn_mult += 1.5 * amount_owned as f64;
            self.stats.gold_mult += 0.3 * amount_owned as f64;
            self.stats.enemy_move_mult += 0.05 * amount_owned as f64;
        }

        //debug
        if self.upgrade_owned("9999") {
            self.stats.width = 200;
            self.stats.height = 100;
            self.stats.enemy_spawn_mult = 12.;
            self.stats.enemy_move_mult = 3.;
            self.stats.base_health = 10000;
        }

        //cleanups
        self.stats.health = (self.stats.base_health as f64 * self.stats.health_mult).ceil() as i32;
    }

    pub fn amount_owned(&self, id: &str) -> u32 {
        self.upgrades.get(id).unwrap_or(&0).clone()
    }

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

pub type CurrentUpgrades = HashMap<String, u32>;

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
    pub fn has_children(&self) -> bool {
        self.children.is_some() && self.children.clone().unwrap().len() > 0
    }

    pub fn get_display_title(&self) -> String {
        if self.children.is_some() {
            " > ".to_string() + self.title.as_str()
        } else {
            " ".to_string() + self.title.as_str()
        }
    }

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

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Inventory {
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
    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Sub)]
pub struct Stats {
    pub base_health: i32,
    pub health_mult: f64,

    pub health: i32,

    pub damage_mult: f64,
    pub damage_flat_boost: i32,

    pub attack_speed_mult: f64,
    pub movement_speed_mult: f64,

    pub enemy_spawn_mult: f64,
    pub enemy_move_mult: f64,

    pub gold_mult: f64,

    pub size: i32,

    pub width: usize,
    pub height: usize,

    pub timer: u64,

    pub mark_chance: u32,
    pub mark_explosion_size: u32,

    pub shove_amount: u32,
    pub shove_damage: u32,

    pub time_offset: Duration,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            base_health: 10,
            health_mult: 1.,

            health: 10,

            damage_mult: 1.,
            damage_flat_boost: 0,

            attack_speed_mult: 1.,
            movement_speed_mult: 1.,

            enemy_spawn_mult: 1.,
            enemy_move_mult: 1.,

            gold_mult: 1.,

            width: 20,
            height: 6,

            size: 0,

            timer: 60,

            mark_chance: 0,
            mark_explosion_size: 1,

            shove_amount: 0,
            shove_damage: 0,

            time_offset: Duration::from_secs(0),
        }
    }
}

pub type UpgradeTree = Vec<UpgradeNode>;

pub fn get_upgrade_tree() -> Result<Vec<UpgradeNode>, serde_json::Error> {
    let upgrade_tree: UpgradeTree = serde_json::from_str(include_str!("upgrades.json"))?;
    Ok(upgrade_tree)
}

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
