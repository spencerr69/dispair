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
    /// Creates a DamageEffect populated from the given DamageArea.
    ///
    /// The resulting DamageEffect uses the DamageArea's area and entity as its initial
    /// active_area and active_entity, sets `complete` to `false`, and records the
    /// current time as the effect's start time.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct or obtain a DamageArea named `damage_area` before converting.
    /// let damage_area = /* a DamageArea */ unimplemented!();
    /// let effect = DamageEffect::from(damage_area);
    /// assert!(!effect.complete);
    /// ```
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
    /// Constructs a `DamageEffect` representing a temporary damage visual tied to a specific area and entity.
    ///
    /// The created effect uses the provided `area` and `entity`, applies the given `duration` and `blink` behaviour,
    /// sets `start_time` to the current instant, marks the effect as not complete, and initializes `active_area` and
    /// `active_entity` from the provided values.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// // let area = /* an Area value */;
    /// // let entity = /* an EntityCharacters value */;
    /// // let effect = DamageEffect::new(area, entity, Duration::from_secs(1), true);
    /// // assert!(!effect.complete);
    /// ```
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

    /// Postpone the effect's start time by a given duration.
    ///
    /// Advances the internal `start_time` forward by `delay`, causing the effect to begin later.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let mut effect = DamageEffect::new(/* area */ /* entity */, Duration::from_secs(0), false);
    /// effect.delay(Duration::from_millis(500));
    /// ```
    pub fn delay(&mut self, delay: Duration) {
        self.start_time += delay;
    }

    /// Advance the effect's timing and update which area and entity should be rendered.
    ///
    /// While the effect is pending (start time is in the future) this sets `active_area` to the origin
    /// and `active_entity` to `Empty`. Once the start time has been reached `active_area` and
    /// `active_entity` are set from the underlying `damage_area`. If the elapsed time since start
    /// is greater than or equal to the damage area's duration the effect is marked `complete`. If the
    /// damage area is configured to blink, `active_entity` toggles between the damage entity and
    /// `Empty` while the effect is active.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut effect = DamageEffect::new(area, entity, std::time::Duration::from_secs(1), true);
    /// effect.update();
    /// // `active_area`/`active_entity` reflect whether the effect has started and whether it is blinking.
    /// ```
    pub fn update(&mut self) {
        let now = Instant::now();

        if now < self.start_time {
            //hasn't started yet
            self.active_area = Area::origin();
            self.active_entity = EntityCharacters::Empty;
        } else {
            self.active_area = self.damage_area.area.clone();
            self.active_entity = self.damage_area.entity.clone();
        }

        if now.duration_since(self.start_time) >= self.damage_area.duration {
            self.complete = true
        } else if self.damage_area.blink {
            if self.active_entity == self.damage_area.entity {
                self.active_entity = EntityCharacters::Empty;
            } else {
                self.active_entity = self.damage_area.entity.clone();
            }
        }
    }

    /// Produce an iterator over the currently active area that pairs each position with the active entity.
    ///
    /// The returned iterator yields `(Position, EntityCharacters)` for every position in `self.active_area`.
    /// The positions and the active entity are captured by value at the time of the call so the iterator
    /// can be used independently of subsequent mutations to the `DamageEffect`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `effect` is a DamageEffect with a 2-position active_area and active_entity set.
    /// // This example demonstrates consuming the iterator and collecting its contents.
    /// let pairs: Vec<_> = effect.get_instructions().collect();
    /// for (pos, entity) in pairs {
    ///     // use `pos` and `entity`
    /// }
    /// ```
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
