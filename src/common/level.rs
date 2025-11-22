pub struct Level {
    xp: u128,
    level: i32,
    xp_to_level: u128,
}

impl Level {
    const SCALE_MULT: f64 = 1.5;

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

    pub fn update(&mut self) {
        if self.xp >= self.xp_to_level {
            self.level += 1;
            self.xp = 0;
            self.xp_to_level = (self.xp_to_level as f64 * Self::SCALE_MULT).ceil() as u128;
        }
    }

    pub fn get_progress_percentage(&self) -> f64 {
        self.xp as f64 / self.xp_to_level as f64 * 100.0
    }
}
