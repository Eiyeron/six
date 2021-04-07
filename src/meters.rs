pub enum Meter {
    Instant(InstantMeter),
    Rolling(RollingMeter),
}
impl Meter {
    /// Implemented here to avoid dealing with the matching in higher levels.
    pub fn current_and_max(&self) -> (u16, u16) {
        match self {
            Meter::Rolling(m) => (m.current_value, m.max),
            Meter::Instant(m) => (m.current_value, m.max),
        }
    }
}

/// A stat meter without a time component.
pub struct InstantMeter {
    pub current_value: u16,
    pub max: u16,
}

impl InstantMeter {
    pub fn new(current_value: u16, max: u16) -> InstantMeter {
        InstantMeter { current_value, max }
    }
}

/// One of the two not-so-unique selling points of this battle engine
pub struct RollingMeter {
    pub current_value: u16,
    pub target_value: u16,
    pub max: u16,
    pub accumulator: f32,
    pub base_rate: f32,
    pub rate_multiplier: f32,
}

impl RollingMeter {
    pub fn new(current: u16, max: u16) -> RollingMeter {
        RollingMeter {
            current_value: current,
            max,
            target_value: current,
            accumulator: 0.,
            base_rate: 1., // value per second
            rate_multiplier: 1.,
        }
    }

    fn current_rate(&self) -> f32 {
        self.base_rate * self.rate_multiplier
    }

    pub fn update(&mut self, dt: f32) {
        if self.current_value == self.target_value {
            return;
        }

        self.accumulator += dt * self.current_rate();
        let previous_value = self.current_value;
        if self.accumulator >= 1. {
            let integer_part = self.accumulator.floor() as u16;
            self.current_value = {
                if self.current_value < self.target_value {
                    self.current_value + integer_part
                } else {
                    self.current_value - integer_part
                }
            };
            self.accumulator = self.accumulator.fract();
        }

        // If the counter went past, fix it instead of complexifying the accumulator bit.
        if previous_value < self.target_value && self.current_value > self.target_value {
            self.current_value = self.target_value;
            self.accumulator = 0.;
        } else if previous_value > self.target_value && self.current_value < self.target_value {
            self.current_value = self.target_value;
            self.accumulator = 0.;
        }
    }
}
