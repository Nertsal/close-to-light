use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[load(serde = "json")]
pub struct Level {
    #[serde(default)]
    pub events: Vec<TimedEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimedEvent {
    /// The beat on which the event should happen.
    pub beat: Time,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    Light(LightEvent),
    /// Swap light and dark colors.
    PaletteSwap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightSerde {
    /// Whether the light is dangerous.
    #[serde(default)]
    pub danger: bool,
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
    /// How long (in beats) before the event should the telegraph occur.
    pub precede_time: Time,
    /// How fast the telegraph is.
    pub speed: Coord,
}

impl Level {
    /// Calculate the last beat when anything happens.
    pub fn last_beat(&self) -> Time {
        self.events
            .iter()
            .map(|event| event.beat + event.duration())
            .max()
            .unwrap_or(Time::ZERO)
    }
}

impl TimedEvent {
    /// Returns the duration (in beats) of the event.
    pub fn duration(&self) -> Time {
        match &self.event {
            Event::Light(event) => event.light.movement.total_duration(),
            Event::PaletteSwap => Time::ZERO,
        }
    }
}

impl LightSerde {
    pub fn instantiate(self, event_id: Option<usize>) -> Light {
        let collider = Collider::new(vec2::ZERO, self.shape);
        Light {
            base_collider: collider.clone(),
            collider,
            // movement: self.movement,
            lifetime: Time::ZERO,
            // lifetime: Lifetime::new_max(self.lifetime * beat_time),
            danger: self.danger,
            event_id,
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
