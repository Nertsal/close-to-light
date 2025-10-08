use super::*;

pub struct Lerp<T> {
    pub time: Bounded<f32>,
    pub smoothstep: bool,
    pub from: T,
    pub to: T,
}

impl<T> Lerp<T> {
    pub fn new(time: f32, from: T, to: T) -> Self {
        Self {
            time: Bounded::new_zero(time),
            smoothstep: false,
            from,
            to,
        }
    }

    pub fn new_smooth(time: f32, from: T, to: T) -> Self {
        Self {
            smoothstep: true,
            ..Self::new(time, from, to)
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time.change(delta_time);
    }
}

impl<T: Float> Lerp<T> {
    pub fn stop(&mut self) {
        self.to = self.current();
    }

    pub fn current(&self) -> T {
        let mut t = self.time.get_ratio();
        if self.smoothstep {
            t = smoothstep(t);
        };
        self.from + (self.to - self.from) * T::from_f32(t)
    }

    pub fn change_target(&mut self, to: T) {
        if self.to == to {
            return;
        }
        self.from = self.current();
        self.to = to;
        self.time.set_ratio(0.0);
    }

    pub fn snap_to(&mut self, to: T) {
        self.from = to;
        self.to = to;
    }
}
