use std::{cell::RefCell, rc::Rc};

use crate::{
    Duration,
    common::{character::Movable, coords::Area},
};

use ratatui::style::{Style, Stylize};

use crate::common::{
    character::Character,
    coords::ChaosArea,
    enemy::Enemy,
    enemy::move_to_point_granular,
    powerup::PowerupTypes,
    powerup::{DynPowerup, Poweruppable},
    roguegame::{EntityCharacters, Layer},
    stats::WeaponStats,
    weapons::{DamageArea, Weapon},
};

#[derive(Clone)]
pub struct Lightning {
    base_damage: i32,
    damage_scalar: f64,
    stats: WeaponStats,
}

impl Lightning {
    const BASE_DAMAGE: i32 = 1;
    const BASE_SIZE: i32 = 1;

    pub fn new(base_weapon_stats: WeaponStats) -> Self {
        Lightning {
            base_damage: Self::BASE_DAMAGE + base_weapon_stats.damage_flat_boost,
            damage_scalar: 1.,
            stats: WeaponStats {
                size: Self::BASE_SIZE + base_weapon_stats.size,
                ..base_weapon_stats
            },
        }
    }
}

impl Weapon for Lightning {
    fn attack(&self, wielder: &Character, enemies: &Vec<Enemy>, layer: &Layer) -> DamageArea {
        let mut begin_pos = wielder.get_pos().clone();

        let mut positions = Vec::new();

        let mut enemies = enemies.clone();

        for _ in 0..self.stats.size {
            let closest = enemies.iter().reduce(|acc, enemy| {
                let (dist_x, dist_y) = enemy.get_pos().get_distance(&begin_pos);
                let enemy_total_dist = dist_x.abs() + dist_y.abs();

                let (acc_dist_x, acc_dist_y) = acc.get_pos().get_distance(&begin_pos);
                let acc_total_dist = acc_dist_x.abs() + acc_dist_y.abs();

                if enemy_total_dist < acc_total_dist && enemy_total_dist > 2 || acc_total_dist <= 2
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
                    (current_pos, _) = move_to_point_granular(&current_pos, &desired_pos, false);
                }

                (current_pos, _) = move_to_point_granular(&current_pos, &desired_pos, false);
                positions.push(current_pos.clone());

                begin_pos = desired_pos;

                enemies = enemies
                    .iter()
                    .filter_map(|e| {
                        if e != closest {
                            return Some(e.clone());
                        } else {
                            return None;
                        }
                    })
                    .collect();
            }
        }

        let mut area = ChaosArea::new(positions);
        area.constrain(layer);

        DamageArea {
            damage_amount: (self.get_damage() as f64 * wielder.stats.damage_mult).ceil() as i32,
            area: Rc::new(RefCell::new(area)),
            entity: EntityCharacters::AttackMist(Style::new().white()),
            duration: Duration::from_secs_f64(0.1),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        (self.base_damage as f64 * self.damage_scalar).ceil() as i32
    }
}

impl Poweruppable for Lightning {
    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Weapon
    }

    fn get_name(&self) -> String {
        "LIGHTNING".into()
    }

    fn get_level(&self) -> i32 {
        self.stats.level
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "LIGHTNING will seek the nearest enemy and damage them.".into(),
            2 => "Increase bounces by 1, increase base damage by 1".into(),
            3 => "Increase bounces by 1, increase base damage by 2".into(),
            4 => "Increase bounces by 1, increase damage scalar by 25%".into(),
            5 => "Double bounces, increase damage scalar by 75%".into(),
            _ => "".into(),
        }
    }

    fn upgrade_self(&mut self, powerup: &DynPowerup) {
        let from = powerup.get_current_level();
        let to = powerup.get_new_level();
        if to <= from {
            return;
        }
        self.stats.level = to;

        for i in (from + 1)..=to {
            match i {
                1 => {}
                2 => {
                    self.stats.size += 1;
                    self.stats.damage_flat_boost += 1;
                }
                3 => {
                    self.stats.size += 1;
                    self.stats.damage_flat_boost += 2;
                }
                4 => {
                    self.stats.size += 1;
                    self.damage_scalar += 0.25;
                }
                5 => {
                    self.stats.size *= 2;
                    self.damage_scalar += 0.75;
                }
                _ => {}
            }
        }
    }
}
