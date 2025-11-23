#[allow(clippy::struct_field_names)]
pub struct Level {
    xp: u128,
    level: i32,
    xp_to_level: u128,
}

impl Default for Level {
    fn default() -> Self {
        Level {
            xp: 0,
            level: 0,
            xp_to_level: 100,
        }
    }
}

impl Level {
    const SCALE_MULT: f64 = 1.5;

    #[must_use]
    pub fn new() -> Self {
        Level {
            xp: 0,
            level: 0,
            xp_to_level: 100,
        }
    }

    pub fn add_xp(&mut self, xp: u128) {
        self.xp += xp;
    }

    pub fn update(&mut self) -> Option<i32> {
        if self.xp >= self.xp_to_level {
            self.level += 1;
            self.xp = 0;
            self.xp_to_level = (self.xp_to_level as f64 * Self::SCALE_MULT).ceil() as u128;
            Some(self.level)
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_progress_percentage(&self) -> u16 {
        (self.xp as f64 / self.xp_to_level as f64 * 100.0)
            .floor()
            .min(100.) as u16
    }
}
