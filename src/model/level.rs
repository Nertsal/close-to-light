use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Level {
    /// Beats per minute.
    pub bpm: R32,
    pub events: Vec<TimedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    /// The beat on which the event should happen.
    pub beat: Time,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Light(LightEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSerde {
    pub position: vec2<Coord>,
    /// Rotation (in degrees).
    #[serde(default = "LightSerde::default_rotation")]
    pub rotation: Coord,
    pub shape: Shape,
    #[serde(default)]
    pub movement: Movement,
    // /// Lifetime (in beats).
    // pub lifetime: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightEvent {
    pub light: LightSerde,
    pub telegraph: Telegraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telegraph {
    /// How long before the event should the telegraph occur (in beats).
    pub precede_time: Time,
    /// How fast the telegraph is.
    pub speed: Coord,
}

impl Level {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
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
