use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LevelConfig {
    pub theme: Theme,
    pub player: PlayerConfig,
    pub health: HealthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Theme {
    pub dark: Color,
    pub light: Color,
    pub danger: Color,
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

impl LevelConfig {
    pub fn preset_easy() -> Self {
        Self {
            health: HealthConfig {
                max: r32(1.0),
                dark_decrease_rate: r32(0.3),
                danger_decrease_rate: r32(0.5),
                restore_rate: r32(0.5),
            },
            ..default()
        }
    }

    pub fn preset_normal() -> Self {
        Self {
            health: HealthConfig::preset_normal(),
            ..default()
        }
    }

    pub fn preset_hard() -> Self {
        Self {
            health: HealthConfig {
                max: r32(1.0),
                dark_decrease_rate: r32(0.7),
                danger_decrease_rate: r32(1.0),
                restore_rate: r32(0.25),
            },
            ..default()
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self { radius: r32(0.5) }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            dark: Color::BLACK,
            light: Color::WHITE,
            danger: Color::RED,
        }
    }
}

impl HealthConfig {
    pub fn preset_normal() -> Self {
        Self {
            max: r32(1.0),
            dark_decrease_rate: r32(0.6),
            danger_decrease_rate: r32(0.75),
            restore_rate: r32(0.35),
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self::preset_normal()
    }
}
