use crate::{
    common::{
        TICK_RATE,
        character::Damageable,
        coords::ChaosArea,
        enemy::{Enemy, move_to_point_granular},
        roguegame::{EntityCharacters, Layer},
        stats::WeaponStats,
    },
    target_types::Duration,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::{Deserialize, Serialize};

use ratatui::style::{Style, Stylize};

use crate::common::character::Renderable;
use crate::common::{
    coords::{Area, Position, SquareArea},
    stats::{DebuffStats, Proc},
    weapons::DamageArea,
};

pub type Debuffs = Vec<Debuff>;

pub trait GetDebuffTypes {
    fn get_on_death_effects(&self) -> Vec<&Debuff>;
    fn get_on_tick_effects(&self) -> Vec<&Debuff>;
    fn get_on_damage_effects(&self) -> Vec<&Debuff>;
}

impl GetDebuffTypes for Debuffs {
    fn get_on_death_effects(&self) -> Vec<&Debuff> {
        self.iter().filter(|d| d.stats.on_death_effect).collect()
    }

    fn get_on_tick_effects(&self) -> Vec<&Debuff> {
        self.iter().filter(|d| d.stats.on_tick_effect).collect()
    }

    fn get_on_damage_effects(&self) -> Vec<&Debuff> {
        self.iter().filter(|d| d.stats.on_damage_effect).collect()
    }
}

#[derive(Serialize, Clone, Deserialize, Debug, Copy, PartialEq)]
pub enum Elements {
    Flame(f64),
    Shock(f64),
}

impl Elements {
    #[must_use]
    pub fn get_honage(&self) -> f64 {
        match self {
            Elements::Flame(honage) | Elements::Shock(honage) => *honage,
        }
    }
}

/// Represents debuffs that can be applied to enemies.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuffTypes {
    MarkedForExplosion,
    FlameBurn,
    FlameIgnite,
    ShockCharge,
    ShockElectrocute,
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
        if self.complete {
            return None;
        }

        match self.debuff_type {
            DebuffTypes::FlameBurn => {
                let ticks = TICK_RATE as u64;
                if !tickcount.is_multiple_of(ticks) {
                    return None;
                }

                if let Some(damage) = self.stats.damage {
                    enemy.take_damage(damage);
                }
                None
            }
            DebuffTypes::FlameIgnite => {
                if !tickcount.is_multiple_of(6) || self.complete {
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
                        chance: 80,
                        debuff: Debuff {
                            debuff_type: DebuffTypes::FlameBurn,
                            stats: DebuffStats {
                                damage: Some(self.stats.damage.unwrap_or(1) * 3),
                                ..self.stats.clone()
                            },
                            complete: false,
                        },
                    };

                    let mut procs = HashMap::new();
                    procs.insert("burn".into(), proc);

                    enemy
                        .debuffs
                        .retain(|d| d.debuff_type != DebuffTypes::FlameBurn);

                    Some(DamageArea {
                        damage_amount: self.stats.damage.expect("No damage?") * 10,
                        area: Rc::new(RefCell::new(area)),
                        entity: EntityCharacters::AttackMist(Style::new().red()),
                        duration: Duration::from_secs_f64(0.05),
                        blink: false,
                        weapon_stats: Some(WeaponStats {
                            procs,
                            ..Default::default()
                        }),
                    })
                } else {
                    None
                }
            }
            DebuffTypes::ShockElectrocute => {
                if tickcount.is_multiple_of(
                    (TICK_RATE * f64::from(self.stats.size.expect("No size on electrocute")))
                        as u64,
                ) {
                    self.complete = true;
                }
                None
            }
            _ => None,
        }
    }
}

pub trait OnDamageEffect {
    fn on_damage(
        &mut self,
        enemy: &mut Enemy,
        layer: &Layer,
        enemies: &[Enemy],
    ) -> Option<DamageArea>;
}

impl OnDamageEffect for Debuff {
    fn on_damage(
        &mut self,
        enemy: &mut Enemy,
        layer: &Layer,
        enemies: &[Enemy],
    ) -> Option<DamageArea> {
        if !enemy.got_hit.0 || self.complete {
            return None;
        }

        match self.debuff_type {
            DebuffTypes::ShockCharge => {
                let begin_pos = enemy.get_pos().clone();

                let mut positions = Vec::new();

                let mut enemies = Vec::from(enemies);

                let size = self.stats.size.unwrap_or(1);

                for _ in 0..size {
                    let closest = enemies.iter().reduce(|acc, enemy| {
                        let (dist_x, dist_y) = enemy.get_pos().get_distance(&begin_pos);
                        let enemy_total_dist = dist_x.abs() + dist_y.abs();

                        let (acc_dist_x, acc_dist_y) = acc.get_pos().get_distance(&begin_pos);
                        let acc_total_dist = acc_dist_x.abs() + acc_dist_y.abs();

                        if enemy_total_dist < acc_total_dist && enemy_total_dist > 2
                            || acc_total_dist <= 2
                        {
                            enemy
                        } else {
                            acc
                        }
                    });

                    let mut current_pos = begin_pos.clone();

                    if let Some(closest) = closest {
                        let desired_pos = closest.get_pos().clone();

                        while current_pos != desired_pos {
                            positions.push(current_pos.clone());
                            (current_pos, _) =
                                move_to_point_granular(&current_pos, &desired_pos, false);
                        }

                        (current_pos, _) =
                            move_to_point_granular(&current_pos, &desired_pos, false);
                        positions.push(current_pos.clone());

                        enemies = enemies
                            .iter()
                            .filter_map(|e| if e != closest { Some(e.clone()) } else { None })
                            .collect();
                    }
                }

                positions.retain(|pos| pos != &begin_pos);

                let mut area = ChaosArea::new(positions);

                let proc = Proc {
                    chance: 100,
                    debuff: Debuff {
                        debuff_type: DebuffTypes::ShockElectrocute,
                        stats: DebuffStats {
                            size: self.stats.size,
                            damage: None,
                            on_death_effect: false,
                            on_damage_effect: false,
                            on_tick_effect: true,
                            misc_value: None,
                        },
                        complete: false,
                    },
                };

                let mut procs = HashMap::new();

                procs.insert("electrocute".into(), proc);

                area.constrain(layer);

                let out = Some(DamageArea {
                    damage_amount: enemy.got_hit.1,
                    area: Rc::new(RefCell::new(area)),
                    entity: EntityCharacters::AttackMist(Style::new().light_yellow()),
                    duration: Duration::from_secs_f64(0.01),
                    blink: false,
                    weapon_stats: Some(WeaponStats {
                        procs,
                        ..Default::default()
                    }),
                });

                enemy.got_hit = (false, 0);
                self.complete = true;

                out
            }
            _ => None,
        }
    }
}
