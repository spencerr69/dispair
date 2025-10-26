use std::time::Instant;

use crate::{
    roguegame::{EntityCharacters, Layer, get_pos, set_entity},
    weapon::{Area, DamageArea},
};

#[derive(Clone)]
pub struct DamageEffect {
    damage_area: DamageArea,

    start_time: Instant,
    pub complete: bool,
}

impl DamageEffect {
    pub fn new(damage_area: DamageArea) -> Self {
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
            } else {
                change_area(
                    layer,
                    self.damage_area.area.clone(),
                    &EntityCharacters::Empty,
                );
            }
        }
    }
}

pub fn change_area(layer: &mut Layer, area: Area, entity: &EntityCharacters) {
    area.iter().for_each(|mut position| {
        position.constrain(layer);
        set_entity(layer, &position, entity.clone()).unwrap_or(())
    });
}
