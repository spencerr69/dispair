//! This module handles visual and gameplay effects, such as damage indicators.
//! It defines the `DamageEffect` struct, which represents a temporary effect in a given area.

use crate::prelude::{Duration, Instant};

use crate::common::character::Renderable;
use crate::common::coords::AreaWrapper::Chaos;
use crate::common::coords::{AreaWrapper, ChaosArea};
use crate::common::entities::EntityCharacters;
use crate::common::{
    coords::{Area, Position, SquareArea},
    weapons::DamageArea,
};
use ratatui::prelude::Style;
use std::{cell::RefCell, rc::Rc};

/// Represents a visual effect that occurs over a specified area for a certain duration.
#[derive(Clone)]
pub struct DamageEffect {
    damage_area: DamageArea,

    start_time: Instant,
    pub complete: bool,

    pub active_area: AreaWrapper,
    pub active_entity: EntityCharacters,
}

impl From<DamageArea> for DamageEffect {
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
    pub fn new(
        area: AreaWrapper,
        entity: EntityCharacters,
        duration: Duration,
        blink: bool,
    ) -> Self {
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

    /// Postpone the effect's start time by a given duration.
    ///
    /// Advances the internal `start_time` forward by `delay`, causing the effect to begin later.
    pub fn delay(&mut self, delay: Duration) {
        self.start_time += delay;
    }

    /// Advance the effect's timing and update which area and entity should be rendered.
    ///
    /// While the effect is pending (start time is in the future), this sets `active_area` to the
    /// origin
    /// and `active_entity` to `Empty`. Once the start time has been reached, `active_area` and
    /// `active_entity` are set from the underlying `damage_area`. If the elapsed time since start
    /// is greater than or equal to the damage area's duration, the effect is marked `complete`.
    /// If the
    /// damage area is configured to blink, `active_entity` toggles between the damage entity and
    /// `Empty` while the effect is active.
    pub fn update(&mut self) {
        let now = Instant::now();

        if now < self.start_time {
            //hasn't started yet
            self.active_area = Chaos(ChaosArea::empty());
            self.active_entity = EntityCharacters::Empty(Style::new());
        } else {
            self.active_area = self.damage_area.area.clone();
            self.active_entity = self.damage_area.entity.clone();
        }

        if now.duration_since(self.start_time) >= self.damage_area.duration {
            self.complete = true;
        } else if self.damage_area.blink {
            if self.active_entity == self.damage_area.entity {
                self.active_entity = EntityCharacters::Empty(Style::new());
            } else {
                self.active_entity = self.damage_area.entity.clone();
            }
        }
    }

    /// Produce an iterator over the currently active area that pairs each position with the active entity.
    ///
    /// The returned iterator yields `(Position, EntityCharacters)` for every position in `self.active_area`.
    /// Value captures the positions and the active entity at the time of the call, so the
    /// iterator
    /// can be used independently of later mutations to the `DamageEffect`.
    pub fn get_instructions(&self) -> impl Iterator<Item = RenderPosition> {
        let active_entity = self.active_entity.clone();

        self.active_area
            .get_inner()
            .pos_iter()
            .map(move |pos| RenderPosition(pos, active_entity.clone()))
    }
}

pub struct RenderPosition(Position, EntityCharacters);

impl Renderable for RenderPosition {
    fn get_pos(&self) -> &Position {
        &self.0
    }

    fn get_entity_char(&self) -> &EntityCharacters {
        &self.1
    }
}
