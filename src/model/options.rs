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

impl Theme {
    pub fn classic() -> Self {
        Self {
            dark: Color::BLACK,
            light: Color::WHITE,
            danger: Color::RED,
            highlight: Color::CYAN,
        }
    }

    pub fn test() -> Self {
        Self {
            dark: Color::try_from("#2B3A67").unwrap(),
            light: Color::try_from("#FFC482").unwrap(),
            danger: Color::try_from("#D34F73").unwrap(),
            highlight: Color::try_from("#61C9A8").unwrap(),
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
