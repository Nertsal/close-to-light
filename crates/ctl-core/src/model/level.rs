use super::*;

#[derive(geng::asset::Load, Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[load(serde = "json")]
#[serde(default)]
pub struct Level {
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
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Calculate the last beat when anything happens.
    pub fn last_beat(&self) -> Time {
        self.events
            .iter()
            .map(|event| event.beat + event.duration())
            .max()
            .unwrap_or(Time::ZERO)
    }

    pub fn calculate_hash(&self) -> String {
        let bytes = bincode::serialize(self).expect("level should be serializable");
        crate::util::calculate_hash(&bytes)
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
            lifetime: Time::ZERO,
            danger: self.danger,
            event_id,
            closest_waypoint: (Time::ZERO, WaypointId::Initial),
        }
    }
}

impl Default for Telegraph {
    fn default() -> Self {
        Self {
            precede_time: TIME_IN_FLOAT_TIME,
            speed: Coord::new(1.0),
        }
    }
}
