use std::time::{Duration, Instant};

pub struct TimeScaler {
    pub start_time: Instant,
    pub scale_amount: f64,
}

impl TimeScaler {
    pub fn now() -> Self {
        Self {
            start_time: Instant::now(),
            scale_amount: 1.0,
        }
    }

    pub fn offset_start_time(mut self, offset: Duration) -> Self {
        self.start_time -= offset;
        self
    }

    pub fn time_in_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn scale(&mut self) -> f64 {
        self.scale_amount = (1.007_f64).powf(self.time_in_secs() as f64) * 2. - 1.;
        self.scale_amount
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

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
        scaler.start_time = Instant::now() - Duration::from_secs(10);
        println!("\n10s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 1.2);
    }

    #[test]
    fn scale_at_60s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = Instant::now() - Duration::from_secs(60);
        println!("\n60s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 2.1);
    }

    #[test]
    fn scale_at_120s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = Instant::now() - Duration::from_secs(120);
        println!("\n120s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 3.8);
    }

    #[test]
    fn scale_at_180s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = Instant::now() - Duration::from_secs(180);
        println!("\n180s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 6.1);
    }

    #[test]
    fn scale_at_300s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = Instant::now() - Duration::from_secs(300);
        println!("\n300s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 15.25)
    }

    #[test]
    fn scale_at_600s() {
        let mut scaler = TimeScaler::now();
        scaler.start_time = Instant::now() - Duration::from_secs(600);
        println!("\n600s: {}\n", scaler.scale());
        assert!(scaler.scale_amount <= 131.);
    }
}
