use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize, ser::Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub upgrades: CurrentUpgrades,
    pub inventory: Inventory,
    pub stats: Stats,
}

impl PlayerState {
    pub fn refresh(&mut self) {
        self.stats = Stats::default();

        //upgrades 1 PRESERVE
        //upgrade 11: PRESERVE::\conform
        if !self.upgrade_owned(&"11".to_string()) {
            self.stats.enemy_spawn_mult = 50.;
            self.stats.timer = 10;
        }

        //upgrade 12 grow
        if self.upgrade_owned(&"12".to_string()) {
            self.stats.size += 1;
        }

        //upgrade 13 become
        if self.upgrade_owned(&"13".to_string()) {
            self.stats.enemy_spawn_mult += 0.8;
            self.stats.height += 5;
            self.stats.width += 5;
        }

        //upgrades 2 STATS
        //upgrade 211 damage/flat_up
        if self.upgrade_owned(&"211".to_string()) {
            self.stats.damage_flat_boost += 1 * self.amount_owned(&"211".to_string()) as i32;
        }

        //upgrade 212 damage/mult_up
        if self.upgrade_owned(&"212".to_string()) {
            self.stats.damage_mult += 0.05 * self.amount_owned(&"212".to_string()) as f64;
        }

        //upgrade 221 health/flat_up
        if self.upgrade_owned(&"221".to_string()) {
            self.stats.base_health += 1 * self.amount_owned(&"221".to_string()) as i32;
        }

        //upgrade 222 health/mult_up
        if self.upgrade_owned(&"222".to_string()) {
            self.stats.health_mult += 0.05 * self.amount_owned(&"222".to_string()) as f64;
        }

        //upgrade 23 attack_rate
        if self.upgrade_owned(&"23".to_string()) {
            self.stats.attack_speed_mult += 0.10 * self.amount_owned(&"23".to_string()) as f64;
        }

        //cleanups
        self.stats.health = (self.stats.base_health as f64 * self.stats.health_mult).ceil() as i32;
    }

    pub fn amount_owned(&self, id: &String) -> u32 {
        self.upgrades.get(id).unwrap_or(&0).clone()
    }

    pub fn upgrade_owned(&self, id: &String) -> bool {
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
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Inventory {
    pub gold: u32,
}

impl Inventory {
    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub size: i32,

    pub width: usize,
    pub height: usize,

    pub timer: u64,
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

            width: 20,
            height: 6,

            size: 0,

            timer: 60,
        }
    }
}

pub type UpgradeTree = Vec<UpgradeNode>;

pub fn get_upgrade_tree() -> Result<Vec<UpgradeNode>, serde_json::Error> {
    let get_file = std::fs::read_to_string(Path::new("src/upgrades.json"))
        .map_err(|_| serde_json::Error::custom("naurrr"))?;
    let upgrade_tree: UpgradeTree = serde_json::from_str(get_file.as_str())?;
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
        assert_eq!(upgrade_tree[0].title, "PRESERVE")
    }

    #[test]
    fn current_upgrades_check() {
        let upgrade_tree = get_upgrade_tree().unwrap();
        let current_upgrades = get_current_upgrades(upgrade_tree, HashMap::new());
        println!("Current upgrades: {:?}", current_upgrades);
        assert!(!current_upgrades.is_empty());
    }
}
