use crate::common::PlayerStateRef;
use std::{cell::RefCell, rc::Rc};

use crate::{
    common::{
        coords::Area,
        debuffs::{Debuff, DebuffTypes, Elements},
        stats::{DebuffStats, Proc},
    },
    new_weapon,
    prelude::Duration,
};

use ratatui::style::Style;

use crate::common::character::{CharacterPositionData, Renderable};
use crate::common::enemies::enemy::{Enemy, get_closest_enemies, move_to_point_granular};
use crate::common::entities::EntityCharacters;
use crate::common::{
    coords::ChaosArea,
    powerup::PowerupTypes,
    powerup::{DynPowerup, Poweruppable},
    rogue::Layer,
    stats::WeaponStats,
    weapons::{DamageArea, Weapon},
};

new_weapon!(Lightning, 1, 1, 3);

impl Weapon for Lightning {
    fn attack(
        &mut self,
        wielder: CharacterPositionData,
        enemies: &[Enemy],
        layer: &Layer,
    ) -> DamageArea {
        if self.cooldown_ticks > 0 && self.cooldown_ticks < Self::BASE_COOLDOWN {
            self.cooldown_ticks += 1;
            return DamageArea::new_empty();
        } else if self.cooldown_ticks >= Self::BASE_COOLDOWN {
            self.cooldown_ticks = 0;
            return DamageArea::new_empty();
        }

        let mut begin_pos = wielder.position;

        let mut positions = Vec::new();

        let mut enemies = Vec::from(enemies);

        for _ in 0..self.stats.size {
            let closest = get_closest_enemies(&enemies, &begin_pos);

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
                    .filter_map(|e| if e != closest { Some(e.clone()) } else { None })
                    .collect();
            }
        }

        let mut area = ChaosArea::new(positions);
        area.constrain(layer);

        let mut entity = EntityCharacters::AttackMist(Style::new().gray());

        if let Some(style) = self.get_elemental_style() {
            *entity.style_mut() = style;
        }

        self.cooldown_ticks += 1;

        DamageArea {
            damage_amount: (f64::from(self.get_damage())
                * self.player_state.borrow().stats.player_stats.damage_mult)
                .ceil() as i32,
            area: Rc::new(RefCell::new(area)),
            entity,
            duration: Duration::from_secs_f64(0.1),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        (f64::from(self.base_damage) * self.damage_scalar).ceil() as i32
    }

    fn get_element(&self) -> Option<Elements> {
        None
    }
}

impl Poweruppable for Lightning {
    fn get_max_level(&self) -> i32 {
        self.player_state.borrow().stats.game_stats.max_method_level
    }

    fn get_name(&self) -> String {
        "LIGHTNING".into()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Weapon
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "LIGHTNING will seek the nearest enemy and damage them.".into(),
            2 => "Increase bounces by 1, increase base damage by 1. ".into(),
            3 => "Increase bounces by 1, increase base damage by 2".into(),
            4 => "Increase bounces by 1, increase damage scalar by 25%".into(),
            5 => "Double bounces, increase damage scalar by 75%. Imbue LIGHTNING with Shock element, giving a chance \
            to charge enemies on hit.".into(),
            _ => String::new(),
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

                    self.element = Some(Elements::Shock(self.stats.elemental_honage));
                    let honage = self.element.expect("Something crazy happened").get_honage();
                    self.stats.procs.insert(
                        "charge".into(),
                        Proc {
                            chance: (20. * honage).ceil().min(100.) as u32,
                            debuff: Debuff {
                                debuff_type: DebuffTypes::ShockCharge,
                                complete: false,
                                stats: DebuffStats {
                                    size: Some((3. * honage).ceil() as i32),
                                    damage: Some((1. * honage).ceil() as i32),
                                    misc_value: None,
                                    on_death_effect: false,
                                    on_tick_effect: false,
                                    on_damage_effect: true,
                                },
                            },
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn get_level(&self) -> i32 {
        self.stats.level
    }
}
