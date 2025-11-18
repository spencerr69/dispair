//! This module handles visual and gameplay effects, such as damage indicators.
//! It defines the `DamageEffect` struct, which represents a temporary effect in a given area.

#[cfg(not(target_family = "wasm"))]
use std::time::{Duration, Instant};

#[cfg(target_family = "wasm")]
use web_time::{Duration, Instant};

use crate::common::{
    coords::{Area, Position},
    roguegame::{EntityCharacters, Layer, set_entity},
    weapon::DamageArea,
};

/// Represents a visual effect that occurs over a specified area for a certain duration.
#[derive(Clone)]
pub struct DamageEffect {
    damage_area: DamageArea,

    start_time: Instant,
    pub complete: bool,

    pub active_area: Area,
    pub active_entity: EntityCharacters,
}

impl From<DamageArea> for DamageEffect {
    /// Creates a `DamageEffect` from a `DamageArea`.
    fn from(damage_area: DamageArea) -> Self {
        Self {
            damage_area: damage_area.clone(),
            complete: false,
            start_time: Instant::now(),

            active_area: damage_area.area,
            active_entity: damage_area.entity,
        }
    }
}

impl DamageEffect {
    /// Creates a new `DamageEffect` with the specified parameters.
    pub fn new(area: Area, entity: EntityCharacters, duration: Duration, blink: bool) -> Self {
        let damage_area = DamageArea {
            damage_amount: 0,
            area: area.clone(),
            entity: entity.clone(),
            duration,
            blink,
            weapon_stats: None,
        };

        Self {
            damage_area,
            complete: false,
            start_time: Instant::now(),

            active_area: area,
            active_entity: entity,
        }
    }

    pub fn delay(&mut self, delay: Duration) {
        self.start_time += delay;
    }

    /// Updates the effect's state, handling its duration and visual representation.
    pub fn update(&mut self) {
        //handle beginning
        if self.start_time > Instant::now() {
            self.active_area = Area::origin();
            self.active_entity = EntityCharacters::Empty;
        } else if self.start_time <= Instant::now() {
            self.active_area = self.damage_area.area.clone();
            self.active_entity = self.damage_area.entity.clone();
        }

        if Instant::now().duration_since(self.start_time) >= self.damage_area.duration {
            self.complete = true
        } else if self.damage_area.blink {
            if self.active_entity == self.damage_area.entity {
                self.active_entity = EntityCharacters::Empty;
            } else {
                self.active_entity = self.damage_area.entity.clone();
            }
        }
    }

    pub fn get_instructions(&self) -> Box<dyn Iterator<Item = (Position, EntityCharacters)>> {
        let active_entity = self.active_entity.clone();

        Box::new(
            self.active_area
                .clone()
                .into_iter()
                .map(move |pos| (pos, active_entity.clone())),
        )
    }
}

/// Changes the entity character within a specified area of a layer.
pub fn change_area(layer: &mut Layer, area: Area, entity: &EntityCharacters) {
    area.clone().into_iter().for_each(|mut position| {
        position.constrain(layer);
        set_entity(layer, &position, entity.clone()).unwrap_or(())
    });
}
