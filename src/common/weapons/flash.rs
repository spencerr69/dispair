use crate::common::PlayerStateRef;
use std::{cell::RefCell, rc::Rc};

use crate::{
    common::{
        coords::Area,
        coords::{Direction, Position, SquareArea},
        debuffs::{Debuff, DebuffTypes, Elements},
        powerup::{DynPowerup, PowerupTypes, Poweruppable},
        rogue::Layer,
        stats::WeaponStats,
        stats::{DebuffStats, Proc},
        weapons::{DamageArea, Weapon},
    },
    new_weapon,
    prelude::Duration,
};

use crate::common::character::CharacterPositionData;
use crate::common::coords::AreaWrapper::Square;
use crate::common::enemies::enemy::Enemy;
use crate::common::entities::EntityCharacters;
use ratatui::style::Style;

new_weapon!(Flash, 2, 1, 1);

impl Poweruppable for Flash {
    fn get_max_level(&self) -> i32 {
        self.player_state.borrow().stats.game_stats.max_method_level
    }

    fn get_name(&self) -> String {
        "FLASH".into()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Weapon
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "FLASH will create a brief damaging field directly in front of you.".into(),
            2 => "Increase size by 1, increase base damage by 1.".into(),
            3 => "Increase base damage by 2".into(),
            4 => "Increase damage scalar by 25%".into(),
            5 => "Increase damage scalar by 75%. Imbue FLASH with Flame element, burning enemies when hit.".into(),
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
                    self.base_damage += 1;
                }
                3 => {
                    self.stats.damage_flat_boost += 2;
                    self.base_damage += 2;
                }
                4 => {
                    self.damage_scalar += 0.25;
                }
                5 => {
                    self.damage_scalar += 0.75;
                    self.element = Some(Elements::Flame(self.stats.elemental_honage));
                    let honage = self.element.expect("Something crazy happened").get_honage();
                    self.stats.procs.insert(
                        "burn".into(),
                        Proc {
                            chance: 80,
                            debuff: Debuff {
                                debuff_type: DebuffTypes::FlameBurn,
                                complete: false,
                                stats: DebuffStats {
                                    size: Some((1. * (honage * 0.5 + 0.5)).ceil() as i32),
                                    damage: Some((0.25 * honage).ceil() as i32),
                                    misc_value: None,
                                    on_death_effect: false,
                                    on_tick_effect: true,
                                    on_damage_effect: false,
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

impl Weapon for Flash {
    /// Creates a `DamageArea` representing this weapon's attack originating from the wielder's position and facing direction.
    ///
    /// The produced `DamageArea` is positioned immediately in front of the wielder according to their facing, carries this weapon's damage scaled by `wielder.stats.damage_mult` (rounded up to an integer), and includes this weapon's `WeaponStats`.
    fn attack(
        &mut self,
        wielder: CharacterPositionData,
        _enemies: &[Enemy],
        layer: &Layer,
    ) -> DamageArea {
        if self.cooldown_ticks > 0 && self.cooldown_ticks < Self::BASE_COOLDOWN {
            self.cooldown_ticks += 1;
            if self.cooldown_ticks == Self::BASE_COOLDOWN {
            } else {
                return DamageArea::new_empty();
            }
        } else if self.cooldown_ticks >= Self::BASE_COOLDOWN {
            self.cooldown_ticks = 0;
        }

        let (x, y) = wielder.position.get();
        let direction = wielder.facing;

        let size = self.stats.size;

        let mut new_area: SquareArea = match direction {
            Direction::DOWN => SquareArea {
                corner1: Position(x + size, y + 1),
                corner2: Position(x - size, y + size),
            },
            Direction::UP => SquareArea {
                corner1: Position(x - size, y - 1),
                corner2: Position(x + size, y - size),
            },
            Direction::LEFT => SquareArea {
                corner1: Position(x - 1, y + size),
                corner2: Position(x - size, y - size),
            },
            Direction::RIGHT => SquareArea {
                corner1: Position(x + 1, y + size),
                corner2: Position(x + size, y - size),
            },
        };

        new_area.constrain(layer);

        self.cooldown_ticks += 1;

        let mut entity = EntityCharacters::AttackBlackout(Style::new().bold().white());

        if let Some(style) = self.get_elemental_style() {
            *entity.style_mut() = style;
        }

        DamageArea {
            area: Square(new_area),
            damage_amount: (f64::from(self.get_damage())
                * self.player_state.borrow().stats.player_stats.damage_mult)
                .ceil() as i32,
            entity,
            duration: Duration::from_secs_f32(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    /// Returns the damage of the sword, calculated from its base damage and scalar.
    fn get_damage(&self) -> i32 {
        (f64::from(self.base_damage) * self.damage_scalar).ceil() as i32
    }

    fn get_element(&self) -> Option<Elements> {
        self.element
    }
}
