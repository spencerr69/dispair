use crate::common::character::{Character, CharacterPositionData, Renderable};
use crate::common::coords::{Area, Position, SquareArea};
use crate::common::enemies::enemy::Enemy;
use crate::common::entities::EntityCharacters;
use crate::common::powerup::{DynPowerup, PowerupTypes, Poweruppable};
use crate::common::rogue::Layer;
use crate::common::weapons::Elements;
use crate::common::weapons::PlayerState;
use crate::common::weapons::{DamageArea, Weapon, WeaponStats};
use crate::new_weapon;
use crate::prelude::Duration;
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use std::cell::RefCell;
use std::rc::Rc;

new_weapon!(Row, 6, 0, 5);

impl Weapon for Row {
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

        let (_, y) = wielder.position.get();

        //size should be half the size for balancing
        let size = self.stats.size / 2;

        let mut area = SquareArea {
            corner1: Position(0, y + size),
            corner2: Position(i32::MAX, y - size),
        };

        area.constrain(layer);

        self.cooldown_ticks += 1;

        DamageArea {
            damage_amount: (f64::from(self.get_damage())
                * self.player_state.borrow().stats.player_stats.damage_mult)
                .ceil() as i32,
            area: Rc::new(RefCell::new(area)),
            entity: EntityCharacters::AttackWeak(Style::new().gray()),
            duration: Duration::from_secs_f64(0.05),
            blink: false,
            weapon_stats: Some(self.stats.clone()),
        }
    }

    fn get_damage(&self) -> i32 {
        (f64::from(self.base_damage) * self.damage_scalar).ceil() as i32
    }

    fn get_element(&self) -> Option<Elements> {
        self.element
    }
}

impl Poweruppable for Row {
    fn get_name(&self) -> String {
        "ROW".into()
    }

    fn get_powerup_type(&self) -> PowerupTypes {
        PowerupTypes::Weapon
    }

    fn upgrade_desc(&self, level: i32) -> String {
        match level {
            1 => "ROW will create a damaging beam which affects an entire row of the map.".into(),
            2 => "Increase size by 1, increase base damage by 1.".into(),
            3 => "Increase damage by 2.".into(),
            4 => "Increase damage scalar by 25%".into(),
            5 => "Increase damage scalar by 75%".into(),
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
                    self.stats.damage_flat_boost += 2;
                }
                4 => {
                    self.damage_scalar += 0.25;
                }
                5 => {
                    self.damage_scalar += 0.75;
                }
                _ => {}
            }
        }
    }

    fn get_level(&self) -> i32 {
        self.stats.level
    }
}
