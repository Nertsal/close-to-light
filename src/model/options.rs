use super::*;

#[derive(Default, Debug, Clone)]
pub struct Options {
    pub volume: VolumeOptions,
    pub theme: Theme,
}

#[derive(Debug, Clone)]
pub struct VolumeOptions {
    pub master: Bounded<f32>,
}

impl Default for VolumeOptions {
    fn default() -> Self {
        Self {
            master: Bounded::new(0.5, 0.0..=1.0),
        }
    }
}
