use std::time::Instant;

pub struct Time {
    start_time: Instant,
    last_frame: Instant,

    delta_seconds: f32,
    elapsed_seconds: f32,
}

impl Time {
    pub(crate) fn new() -> Self {
        let now = Instant::now();

        Self {
            start_time: now,
            last_frame: now,

            delta_seconds: 0.0,
            elapsed_seconds: 0.0,
        }
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();

        self.delta_seconds = (now - self.last_frame).as_secs_f32();
        self.elapsed_seconds = (now - self.start_time).as_secs_f32();

        self.last_frame = now;
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    pub fn reset(&mut self) {
        let now = Instant::now();

        self.start_time = now;
        self.last_frame = now;
        self.delta_seconds = 0.0;
        self.elapsed_seconds = 0.0;
    }
}