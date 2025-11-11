use std::time::{Duration, Instant};

use crate::{
    coords::Area,
    roguegame::{EntityCharacters, Layer, get_pos, set_entity},
    weapon::DamageArea,
};

#[derive(Clone)]
pub struct DamageEffect {
    damage_area: DamageArea,

    start_time: Instant,
    pub complete: bool,
}

impl From<DamageArea> for DamageEffect {
    fn from(damage_area: DamageArea) -> Self {
        Self {
            damage_area,
            complete: false,
            start_time: Instant::now(),
        }
    }
}

impl DamageEffect {
    pub fn new(area: Area, entity: EntityCharacters, duration: Duration, blink: bool) -> Self {
        let damage_area = DamageArea {
            damage_amount: 0,
            area,
            entity,
            duration,
            blink,
            weapon_stats: None,
        };

        Self {
            damage_area,
            complete: false,
            start_time: Instant::now(),
        }
    }

    pub fn take_effect(&self, layer: &mut Layer) {
        let area = &self.damage_area.area;
        change_area(layer, area.clone(), &self.damage_area.entity);
    }

    pub fn update(&mut self, layer: &mut Layer) {
        change_area(
            layer,
            self.damage_area.area.clone(),
            &EntityCharacters::Empty,
        );
        if Instant::now().duration_since(self.start_time) >= self.damage_area.duration {
            change_area(
                layer,
                self.damage_area.area.clone(),
                &EntityCharacters::Empty,
            );
            self.complete = true
        } else if self.damage_area.blink {
            if get_pos(layer, &self.damage_area.area.corner1) == &EntityCharacters::Empty {
                change_area(
                    layer,
                    self.damage_area.area.clone(),
                    &self.damage_area.entity,
                );
            }
        }
    }
}

pub fn change_area(layer: &mut Layer, area: Area, entity: &EntityCharacters) {
    area.clone().into_iter().for_each(|mut position| {
        position.constrain(layer);
        set_entity(layer, &position, entity.clone()).unwrap_or(())
    });
}
