use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LevelConfig {
    pub player: PlayerConfig,
    pub health: HealthConfig,
    pub waypoints: WaypointsConfig,
    pub modifiers: LevelModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerConfig {
    pub radius: Coord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct HealthConfig {
    /// Max health value.
    pub max: Time,
    /// How fast health decreases per second in darkness.
    pub dark_decrease_rate: Time,
    /// How fast health decreases per second in danger.
    pub danger_decrease_rate: Time,
    /// How much health restores per second while in light.
    pub restore_rate: Time,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WaypointsConfig {
    pub show: bool,
    pub sustain_time: Time,
    pub fade_time: Time,
    pub sustain_scale: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LevelModifiers {
    /// Play through the level without player input.
    pub clean_auto: bool,
    /// You cannot fail the level.
    pub nofail: bool,
    /// No telegraphs.
    pub sudden: bool,
    /// Don't render lights.
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, enum_iterator::Sequence)]
pub enum Modifier {
    NoFail,
    Sudden,
    Hidden,
}

impl Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::NoFail => write!(f, "Nofail"),
            Modifier::Sudden => write!(f, "Sudden"),
            Modifier::Hidden => write!(f, "Hidden"),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for LevelModifiers {
    fn default() -> Self {
        Self {
            clean_auto: false,
            nofail: false,
            sudden: false,
            hidden: false,
        }
    }
}

impl Default for WaypointsConfig {
    fn default() -> Self {
        Self {
            show: false,
            sustain_time: r32(1.0),
            fade_time: r32(0.5),
            sustain_scale: r32(0.5),
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self { radius: r32(0.5) }
    }
}

impl Theme {
    pub fn classic() -> Self {
        Self {
            dark: Color::BLACK,
            light: Color::WHITE,
            danger: Color::RED,
        }
    }

    pub fn test() -> Self {
        Self {
            dark: Color::try_from("#2B3A67").unwrap(),
            light: Color::try_from("#FFC482").unwrap(),
            danger: Color::try_from("#D34F73").unwrap(),
        }
    }

    /// Make `dark` color transparent black.
    pub fn transparent(self) -> Self {
        Self {
            dark: Color::TRANSPARENT_BLACK,
            ..self
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic()
    }
}

impl HealthConfig {
    pub fn preset_easy() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.3),
            danger_decrease_rate: r32(0.5),
            restore_rate: r32(0.5),
        }
    }

    pub fn preset_normal() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.6),
            danger_decrease_rate: r32(0.75),
            restore_rate: r32(0.35),
        }
    }

    pub fn preset_hard() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(1.0),
            danger_decrease_rate: r32(2.0),
            restore_rate: r32(0.25),
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self::preset_normal()
    }
}
