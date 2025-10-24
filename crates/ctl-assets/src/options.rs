use super::*;

use serde::Deserializer;

fn ok_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned + Default,
    D: Deserializer<'de>,
{
    Ok(T::deserialize(deserializer).unwrap_or_else(|err| {
        log::error!(
            "failed to deserialize type {}, using default, error: {:?}",
            std::any::type_name::<T>(),
            err
        );
        T::default()
    }))
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Options {
    #[serde(deserialize_with = "ok_or_default")]
    pub volume: VolumeOptions,
    #[serde(deserialize_with = "ok_or_default")]
    pub theme: Theme,
    #[serde(deserialize_with = "ok_or_default")]
    pub graphics: GraphicsOptions,
    #[serde(deserialize_with = "ok_or_default")]
    pub cursor: CursorOptions,
    #[serde(deserialize_with = "ok_or_default")]
    pub gameplay: GameplayOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsOptions {
    pub crt: GraphicsCrtOptions,
    pub lights: GraphicsLightsOptions,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsLightsOptions {
    pub telegraph_color: ThemeColor,
    pub perfect_color: ThemeColor,
}

impl Default for GraphicsLightsOptions {
    fn default() -> Self {
        Self {
            telegraph_color: ThemeColor::Light,
            perfect_color: ThemeColor::Light,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CursorOptions {
    pub show_perfect_radius: bool,
    pub inner_radius: f32,
    pub outer_radius: f32,
}

impl Default for CursorOptions {
    fn default() -> Self {
        Self {
            show_perfect_radius: true,
            inner_radius: 0.15,
            outer_radius: 0.05,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GameplayOptions {
    /// Music offset in ms.
    pub music_offset: f32,
}

impl Default for GameplayOptions {
    fn default() -> Self {
        Self { music_offset: 20.0 }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VolumeOptions {
    /// Volume in range `0..=100`.
    pub master: f32, // TODO: range should be part of the type
}

impl VolumeOptions {
    pub fn master(&self) -> f32 {
        self.master / 100.0
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
        Self { master: 50.0 }
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

    pub fn swap(self, t: f32) -> Self {
        Self {
            light: Color::lerp(self.light, self.dark, t),
            dark: Color::lerp(self.dark, self.light, t),
            ..self
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic()
    }
}
