use super::*;

use geng_utils::interpolation::SecondOrderState;

#[derive(Debug, Clone)]
pub struct VfxValue {
    pub value: SecondOrderState<R32>,
    pub time_left: FloatTime,
}

impl VfxValue {
    pub fn new(frequency: f32, damping: f32, response: f32) -> Self {
        Self {
            value: SecondOrderState::new(frequency, damping, response, R32::ZERO),
            time_left: FloatTime::ZERO,
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.time_left = (self.time_left - delta_time).max(FloatTime::ZERO);
        self.value.target = if self.time_left.as_f32() > 0.0 {
            r32(1.0)
        } else {
            r32(0.0)
        };
        self.value.update(delta_time.as_f32());
    }
}

#[derive(Debug, Clone)]
pub struct Vfx {
    pub palette_swap: SecondOrderState<R32>,
    pub rgb_split: VfxValue,
    pub camera_shake: R32,
}

impl Vfx {
    pub fn new() -> Self {
        Self {
            palette_swap: SecondOrderState::new(3.0, 1.0, 0.0, R32::ZERO),
            rgb_split: VfxValue::new(2.0, 1.0, 0.0),
            camera_shake: R32::ZERO,
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.palette_swap.update(delta_time.as_f32());
        self.rgb_split.update(delta_time);
    }
}

impl Default for Vfx {
    fn default() -> Self {
        Self::new()
    }
}
