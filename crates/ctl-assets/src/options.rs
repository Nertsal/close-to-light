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
    pub account: AccountOptions,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AccountOptions {
    pub auto_login: bool,
}

impl Default for AccountOptions {
    fn default() -> Self {
        Self { auto_login: true }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsOptions {
    pub crt: GraphicsCrtOptions,
    pub lights: GraphicsLightsOptions,
    pub colors: GraphicsColorsOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsCrtOptions {
    pub enabled: bool,
    #[serde(skip)]
    pub curvature: f32,
    #[serde(skip)]
    pub vignette: f32,
    #[serde(skip)]
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
pub struct GraphicsColorsOptions {
    pub blue: f32,
    pub saturation: f32,
}

impl Default for GraphicsColorsOptions {
    fn default() -> Self {
        Self {
            blue: 1.0,
            saturation: 1.0,
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
            perfect_color: ThemeColor::Highlight,
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
    pub fn new(dark: &str, light: &str, danger: &str, highlight: &str) -> anyhow::Result<Self> {
        Ok(Self {
            dark: Color::try_from(dark)?,
            light: Color::try_from(light)?,
            danger: Color::try_from(danger)?,
            highlight: Color::try_from(highlight)?,
        })
    }

    pub fn classic() -> Self {
        Self::new("#000000", "#ffffff", "#ff0000", "#00ffff").unwrap()
    }

    pub fn stargazer() -> Self {
        Self::new("#162246", "#f9bc76", "#ad3454", "#1e8294").unwrap()
    }

    pub fn corruption() -> Self {
        Self::new("#2b172a", "#db9d51", "#a23f6d", "#30aabb").unwrap()
    }

    pub fn linksider() -> Self {
        Self::new("#46425E", "#FFEECC", "#FF6973", "#00B9BE").unwrap()
    }

    pub fn frostlight() -> Self {
        Self::new("#18284A", "#EBF9FF", "#D75672", "#369ADD").unwrap()
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
        if cfg!(feature = "demo") {
            Self::classic()
        } else {
            Self::frostlight()
        }
    }
}
