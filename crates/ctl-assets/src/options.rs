use super::*;

use geng_utils::bounded::Bounded;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Options {
    pub volume: VolumeOptions,
    pub theme: Theme,
    pub graphics: GraphicsOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsOptions {
    pub crt: GraphicsCrtOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsCrtOptions {
    pub enabled: bool,
    pub curvature: f32,
    pub vignette: f32,
    pub scanlines: f32,
}

impl Default for GraphicsCrtOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            curvature: 20.0,
            vignette: 0.2,
            scanlines: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Theme {
    pub dark: Color,
    pub light: Color,
    pub danger: Color,
    pub highlight: Color,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ThemeColor {
    Dark,
    Light,
    Danger,
    Highlight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "VolumeOptionsRaw", into = "VolumeOptionsRaw")]
pub struct VolumeOptions {
    /// Volume in range `0..=100`.
    pub master: Bounded<f32>, // TODO: range should be part of the type
}

#[derive(Serialize, Deserialize)]
struct VolumeOptionsRaw {
    master: f32,
}

impl From<VolumeOptionsRaw> for VolumeOptions {
    fn from(value: VolumeOptionsRaw) -> Self {
        let VolumeOptionsRaw { master } = value;

        let mut opt = Self::default();
        opt.master.set(master);
        opt
    }
}

impl From<VolumeOptions> for VolumeOptionsRaw {
    fn from(value: VolumeOptions) -> Self {
        Self {
            master: value.master.value(),
        }
    }
}

impl PartialEq for VolumeOptions {
    fn eq(&self, other: &Self) -> bool {
        self.master.value() == other.master.value()
    }
}

impl VolumeOptions {
    pub fn master(&self) -> f32 {
        self.master.value() / 100.0
    }

    pub fn music(&self) -> f32 {
        self.master()
    }

    pub fn sfx(&self) -> f32 {
        self.master()
    }
}

impl Default for VolumeOptions {
    fn default() -> Self {
        Self {
            master: Bounded::new(50.0, 0.0..=100.0),
        }
    }
}

impl Theme {
    pub fn classic() -> Self {
        Self {
            dark: Color::BLACK,
            light: Color::WHITE,
            danger: Color::RED,
            highlight: Color::CYAN,
        }
    }

    pub fn peach_mint() -> Self {
        Self {
            dark: Color::try_from("#2B3A67").unwrap(),
            light: Color::try_from("#FFC482").unwrap(),
            danger: Color::try_from("#D34F73").unwrap(),
            highlight: Color::try_from("#61C9A8").unwrap(),
        }
    }

    pub fn corruption() -> Self {
        Self {
            dark: Color::try_from("#382637").unwrap(),
            light: Color::try_from("#DEA257").unwrap(),
            danger: Color::try_from("#A23F6D").unwrap(),
            highlight: Color::try_from("#43BCCD").unwrap(),
        }
    }

    pub fn linksider() -> Self {
        Self {
            dark: Color::try_from("#46425E").unwrap(),
            light: Color::try_from("#FFEECC").unwrap(),
            danger: Color::try_from("#FF6973").unwrap(),
            highlight: Color::try_from("#00B9BE").unwrap(),
        }
    }

    /// Make `dark` color transparent black.
    pub fn transparent(self) -> Self {
        Self {
            dark: Color::TRANSPARENT_BLACK,
            ..self
        }
    }

    pub fn get_color(&self, color: ThemeColor) -> Color {
        match color {
            ThemeColor::Dark => self.dark,
            ThemeColor::Light => self.light,
            ThemeColor::Danger => self.danger,
            ThemeColor::Highlight => self.highlight,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic()
    }
}
