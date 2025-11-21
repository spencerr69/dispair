//! This module provides a `TimeScaler` that dynamically adjusts a scaling factor
//! over time. This is used to increase the game's difficulty as time progresses.

#[cfg(not(target_family = "wasm"))]
use std::time::{Duration, SystemTime};

#[cfg(target_family = "wasm")]
use web_time::{Duration, SystemTime};

/// Handles the scaling of game difficulty over time.
pub struct TimeScaler {
    /// The time at which the scaling began.
    pub start_time: SystemTime,
    /// The current scaling factor.
    pub scale_amount: f64,
    /// The original start time (to work out offset)
    original_start_time: SystemTime,
}

impl TimeScaler {
    /// Creates a new `TimeScaler` with the start time set to now.
    pub fn now() -> Self {
        Self {
            start_time: SystemTime::now(),
            original_start_time: SystemTime::now(),
            scale_amount: 1.0,
        }
    }

    /// Offsets the start time by a given `Duration`.
    pub fn offset_start_time(&mut self, offset: Duration) {
        self.start_time = self.original_start_time - offset;
    }

    /// Returns the elapsed time in seconds since the `start_time`.
    pub fn time_in_secs(&self) -> u64 {
        if let Ok(elapsed) = self.start_time.elapsed() {
            elapsed.as_secs()
        } else {
            0
        }
    }

    /// Calculates the new scaling factor based on the elapsed time.
    pub fn scale(&mut self) -> f64 {
        self.scale_amount = (1.007_f64).powf(self.time_in_secs() as f64) * 2. - 1.;
        self.scale_amount
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    use std::time::{Duration, SystemTime};

    #[cfg(target_family = "wasm")]
    use web_time::{Duration, SystemTime};

    use crate::common::timescaler::TimeScaler;

    #[test]
    fn scale_at_0s() {
        let mut scaler = TimeScaler::now();
        println!("\n0s: {}\n", scaler.scale());
        assert!(scaler.scale_amount == 1.0);
    }

    #[test]
    fn scale_at_10s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = SystemTime::now() - Duration::from_secs(10);
        println!("\n10s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 1.2);
    }

    #[test]
    fn scale_at_60s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = SystemTime::now() - Duration::from_secs(60);
        println!("\n60s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 2.1);
    }

    #[test]
    fn scale_at_120s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = SystemTime::now() - Duration::from_secs(120);
        println!("\n120s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 3.8);
    }

    #[test]
    fn scale_at_180s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = SystemTime::now() - Duration::from_secs(180);
        println!("\n180s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 6.1);
    }

    #[test]
    fn scale_at_300s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = SystemTime::now() - Duration::from_secs(300);
        println!("\n300s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 15.25)
    }

    #[test]
    fn scale_at_600s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = SystemTime::now() - Duration::from_secs(600);
        println!("\n600s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 131.);
    }
}
