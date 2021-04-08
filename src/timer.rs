pub struct Timer {
    remaining: f32,
}

impl Timer {
    pub fn new(duration: f32) -> Timer {
        Timer {
            remaining: duration,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.);
    }

    pub fn tetra_tick(&mut self, ctx: &tetra::Context) {
        self.tick(tetra::time::get_delta_time(ctx).as_secs_f32())
    }

    pub fn done(&self) -> bool {
        self.remaining == 0.
    }
}
