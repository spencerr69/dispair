use crate::{
    common::{enemy::Enemy, stats::WeaponStats},
    target_types::Duration,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::*,
    coords::{Area, Position, SquareArea},
    roguegame::*,
    stats::{DebuffStats, Proc},
    weapons::DamageArea,
};

pub type Debuffs = Vec<Debuff>;

pub trait GetDebuffTypes {
    fn get_on_death_effects(&self) -> Vec<&Debuff>;
    fn get_on_tick_effects(&self) -> Vec<&Debuff>;
}

impl GetDebuffTypes for Debuffs {
    fn get_on_death_effects(&self) -> Vec<&Debuff> {
        self.iter().filter(|d| d.stats.on_death_effect).collect()
    }

    fn get_on_tick_effects(&self) -> Vec<&Debuff> {
        self.iter().filter(|d| d.stats.on_tick_effect).collect()
    }
}

#[derive(Serialize, Clone, Deserialize, Debug, Copy, PartialEq)]
pub enum Elements {
    Flame(f64),
}

impl Elements {
    pub fn get_honage(&self) -> f64 {
        match self {
            Elements::Flame(honage) => *honage,
        }
    }
}

/// Represents debuffs that can be applied to enemies.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuffTypes {
    MarkedForExplosion,
    FlameBurn,
    FlameIgnite,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Debuff {
    pub debuff_type: DebuffTypes,
    pub stats: DebuffStats,
    pub complete: bool,
}

impl Debuff {}

/// A trait for effects that trigger when an enemy dies.
pub trait OnDeathEffect {
    /// Called when an enemy dies, potentially creating a `DamageArea`.
    fn on_death(&self, enemy: Enemy, layer: &Layer) -> Option<DamageArea>;
}

impl OnDeathEffect for Debuff {
    /// Produces an optional area-of-effect damage specification to emit when this debuff triggers on an enemy's death.
    ///
    /// If the debuff is `MarkedForExplosion` and `stats.size` is `Some(size)`, returns a `DamageArea` describing a square area centered on the enemy with radius `size`, using `stats.damage` (or `0` if absent) as the damage amount, an `AttackMist` visual styled dark gray, a duration of 0.15 seconds, `blink = false`, and no `weapon_stats`. If `stats.size` is `None`, returns `None`.
    fn on_death(&self, enemy: Enemy, layer: &Layer) -> Option<DamageArea> {
        match self.debuff_type {
            DebuffTypes::MarkedForExplosion => {
                if let Some(size) = self.stats.size {
                    let mut area = SquareArea {
                        corner1: Position(
                            enemy.position.0.saturating_sub(size),
                            enemy.position.1.saturating_sub(size),
                        ),
                        corner2: Position(
                            enemy.position.0.saturating_add(size),
                            enemy.position.1.saturating_add(size),
                        ),
                    };

                    area.constrain(layer);

                    Some(DamageArea {
                        damage_amount: self.stats.damage.unwrap_or(0),
                        area: Rc::new(RefCell::new(area)),
                        entity: EntityCharacters::AttackMist(Style::new().dark_gray()),
                        duration: Duration::from_secs_f64(0.05),
                        blink: false,
                        weapon_stats: None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub trait OnTickEffect {
    fn on_tick(&mut self, enemy: &mut Enemy, layer: &Layer, tickcount: u64) -> Option<DamageArea>;
}

impl OnTickEffect for Debuff {
    fn on_tick(&mut self, enemy: &mut Enemy, layer: &Layer, tickcount: u64) -> Option<DamageArea> {
        match self.debuff_type {
            DebuffTypes::FlameBurn => {
                let ticks = crate::common::TICK_RATE as u64;
                if tickcount % ticks != 0 {
                    return None;
                }

                if let Some(damage) = self.stats.damage {
                    enemy.take_damage(damage);
                }
                None
            }
            DebuffTypes::FlameIgnite => {
                if tickcount % 3 != 0 || self.complete {
                    return None;
                }

                if let Some(size) = self.stats.size {
                    let mut area = SquareArea {
                        corner1: Position(
                            enemy.position.0.saturating_sub(size),
                            enemy.position.1.saturating_sub(size),
                        ),
                        corner2: Position(
                            enemy.position.0.saturating_add(size),
                            enemy.position.1.saturating_add(size),
                        ),
                    };

                    area.constrain(layer);

                    self.complete = true;

                    let proc = Proc {
                        chance: 100,
                        debuff: Debuff {
                            debuff_type: DebuffTypes::FlameBurn,
                            stats: self.stats.clone(),
                            complete: false,
                        },
                    };

                    let mut procs = HashMap::new();
                    procs.insert("burn".into(), proc);

                    Some(DamageArea {
                        damage_amount: self.stats.damage.expect("no damage?") * 10,
                        area: Rc::new(RefCell::new(area)),
                        entity: EntityCharacters::AttackMist(Style::new().red()),
                        duration: Duration::from_secs_f64(0.05),
                        blink: false,
                        weapon_stats: Some(WeaponStats {
                            procs: procs,
                            ..Default::default()
                        }),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
