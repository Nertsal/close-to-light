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

    pub fn set(&mut self, time_left: FloatTime, target: R32) {
        self.time_left = self.time_left.max(time_left);
        self.value.target = target;
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.time_left = (self.time_left - delta_time).max(FloatTime::ZERO);
        if self.time_left.as_f32() <= 0.0 {
            self.value.target = R32::ZERO;
        }
        self.value.update(delta_time.as_f32());
    }
}

#[derive(Debug, Clone)]
pub struct Vfx {
    pub palette_swap: SecondOrderState<R32>,
    pub rgb_split: VfxValue,
    pub camera_shake: R32,
    pub vignette: VfxValue,
    pub curvature: VfxValue,
}

impl Vfx {
    pub fn new() -> Self {
        Self {
            palette_swap: SecondOrderState::new(3.0, 1.0, 0.0, R32::ZERO),
            rgb_split: VfxValue::new(2.0, 1.0, 0.0),
            camera_shake: R32::ZERO,
            vignette: VfxValue::new(2.0, 1.0, 0.0),
            curvature: VfxValue::new(2.0, 1.0, 0.0),
        }
    }

    pub fn reset(&mut self) {
        self.palette_swap.target = R32::ZERO;
        self.rgb_split.time_left = FloatTime::ZERO;
        self.vignette.time_left = FloatTime::ZERO;
        self.curvature.time_left = FloatTime::ZERO;
        self.camera_shake = R32::ZERO;
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.palette_swap.update(delta_time.as_f32());
        self.rgb_split.update(delta_time);
        self.vignette.update(delta_time);
        self.curvature.update(delta_time);
    }
}

impl Default for Vfx {
    fn default() -> Self {
        Self::new()
    }
}
