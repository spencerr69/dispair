//! This module provides a `TimeScaler` that dynamically adjusts a scaling factor
//! over time. This is used to increase the game's difficulty as time progresses.

use crate::prelude::SystemTime;

/// Handles the scaling of game difficulty over time.
pub struct TimeScaler {
    /// The time at which the scaling began.
    pub start_time: SystemTime,
    /// The current scaling factor.
    pub doom: f64,

    pub doom_offset: f64,
}

impl TimeScaler {
    const SCALE_BASE: f64 = 1.007_f64;

    /// Creates a new `TimeScaler` with the start time set to now.
    #[must_use]
    pub fn now() -> Self {
        Self {
            start_time: SystemTime::now(),
            doom: 1.0,
            doom_offset: 0.,
        }
    }

    pub fn offset_doom(&mut self, offset: f64) {
        self.doom_offset = offset;
    }

    /// Returns the elapsed time in seconds since the `start_time`.
    #[must_use]
    pub fn time_in_secs(&self) -> u64 {
        if let Ok(elapsed) = self.start_time.elapsed() {
            elapsed.as_secs()
        } else {
            0
        }
    }

    /// Calculates the new scaling factor based on the elapsed time.
    pub fn scale(&mut self) -> f64 {
        let mut doom = Self::SCALE_BASE.powf(self.time_in_secs() as f64) * (self.doom_offset + 1.);
        // if self.doom > 50. {
        //     doom = convert_range(doom, 50., 150., 50., 100.);
        // }
        self.doom = doom;
        self.doom
    }
}
