use std::{collections::HashMap, ops::Sub};

use crate::target_types::Duration;

use derive_more::Sub;
use serde::{Deserialize, Serialize};

use crate::common::debuffs::Debuff;

/// Represents the player's inventory.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Inventory {
    /// The amount of gold the player has.
    pub gold: u128,
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
    pub fn add_gold(&mut self, amount: u128) {
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

    pub elemental_honage: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DebuffStats {
    pub size: Option<i32>,
    pub damage: Option<i32>,
    pub misc_value: Option<u32>,
    pub on_death_effect: bool,
    pub on_tick_effect: bool,
    pub on_damage_effect: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proc {
    /// Chance is a int between 0-100.
    pub chance: u32,
    pub debuff: Debuff,
}

impl Default for GameStats {
    /// Baseline game-level modifiers used when no upgrades are applied.
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
    /// Constructs a `PlayerStats` with baseline health, damage, movement, and shove defaults.
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
    fn default() -> Self {
        Self {
            damage_flat_boost: 0,
            size: 0,
            procs: HashMap::new(),
            level: 1,
            elemental_honage: 1.,
        }
    }
}
