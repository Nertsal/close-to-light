use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[load(serde = "json")]
pub struct Level {
    pub config: LevelConfig,
    // /// Whether to start rng after the predefined level is finished.
    // #[serde(default)]
    // pub rng_end: bool,
    pub events: Vec<TimedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelConfig {
    /// Beats per minute.
    pub bpm: R32,
    #[serde(default)]
    pub health: HealthConfig,
    #[serde(default)]
    pub theme: LevelTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelTheme {
    pub player: Color,
    pub dark: Color,
    pub light: Color,
    pub danger: Color,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            bpm: r32(150.0),
            health: default(),
            theme: default(),
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            max: r32(1.5),
            decrease_rate: r32(1.0),
            restore_rate: r32(0.5),
        }
    }
}

impl Default for LevelTheme {
    fn default() -> Self {
        Self {
            player: Color::WHITE,
            dark: Color::BLACK,
            light: Color::WHITE,
            danger: Color::RED,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthConfig {
    /// Max health value.
    pub max: Time,
    /// How fast health decreases per second.
    pub decrease_rate: Time,
    /// How much health restores per second while in light.
    pub restore_rate: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedEvent {
    /// The beat on which the event should happen.
    pub beat: Time,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    Light(LightEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightSerde {
    pub position: vec2<Coord>,
    /// Rotation (in degrees).
    #[serde(default = "LightSerde::default_rotation")]
    pub rotation: Coord,
    pub shape: Shape,
    /// Movement with timings in beats.
    #[serde(default)]
    pub movement: Movement,
    // /// Lifetime (in beats).
    // pub lifetime: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightEvent {
    pub light: LightSerde,
    pub telegraph: Telegraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Telegraph {
    /// How long before the event should the telegraph occur (in beats).
    pub precede_time: Time,
    /// How fast the telegraph is.
    pub speed: Coord,
}

impl Level {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.config.bpm
    }
}

impl LightSerde {
    fn default_rotation() -> Coord {
        Coord::ZERO
    }

    pub fn instantiate(self, beat_time: Time) -> Light {
        let collider = Collider {
            position: self.position,
            rotation: Angle::from_degrees(self.rotation),
            shape: self.shape,
        };
        Light {
            base_collider: collider.clone(),
            collider,
            movement: self.movement.with_beat_time(beat_time),
            lifetime: Time::ZERO,
            // lifetime: Lifetime::new_max(self.lifetime * beat_time),
        }
    }
}

impl Default for Telegraph {
    fn default() -> Self {
        Self {
            precede_time: Time::new(1.0),
            speed: Coord::new(1.0),
        }
    }
}
