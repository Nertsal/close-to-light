use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Options {
    pub volume: VolumeOptions,
    pub theme: Theme,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Theme {
    pub dark: Color,
    pub light: Color,
    pub danger: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeOptions {
    pub master: Bounded<f32>, // TODO: impl in crate
}

impl PartialEq for VolumeOptions {
    fn eq(&self, other: &Self) -> bool {
        self.master.value() == other.master.value()
    }
}

impl VolumeOptions {
    pub fn master(&self) -> f32 {
        self.master.value()
    }

    pub fn music(&self) -> f32 {
        self.master()
    }
}

impl Default for VolumeOptions {
    fn default() -> Self {
        Self {
            master: Bounded::new(0.5, 0.0..=1.0),
        }
    }
}
