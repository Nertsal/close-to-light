use super::*;

#[derive(geng::asset::Load, Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[load(serde = "json")]
#[serde(default)]
pub struct Level {
    pub events: Vec<TimedEvent>,
    pub timing: Timing,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timing {
    /// Points are assumed to be sorted by time.
    pub points: Vec<TimingPoint>,
}

/// A timing point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimingPoint {
    /// The time from which this timing applies.
    pub time: Time,
    /// Time for a single beat (in seconds).
    pub beat_time: FloatTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimedEvent {
    /// The time on which the event should happen.
    pub time: Time,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    Light(LightEvent),
    /// Swap light and dark colors.
    PaletteSwap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightEvent {
    /// Whether the light is dangerous.
    #[serde(default)]
    pub danger: bool,
    pub shape: Shape,
    /// Movement with timings in beats.
    #[serde(default)]
    pub movement: Movement,
}

impl Level {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            timing: Timing::default(),
        }
    }

    /// Calculate the last time when anything happens.
    pub fn last_time(&self) -> Time {
        self.events
            .iter()
            .map(|event| event.time + event.duration())
            .max()
            .unwrap_or(Time::ZERO)
    }

    pub fn calculate_hash(&self) -> String {
        let bytes = bincode::serialize(self).expect("level should be serializable");
        crate::util::calculate_hash(&bytes)
    }
}

impl Timing {
    pub fn get_timing(&self, time: Time) -> TimingPoint {
        let i = match self
            .points
            .binary_search_by_key(&time, |timing| timing.time)
        {
            Ok(i) => i,
            Err(0) => {
                // There are no timing points smh
                return TimingPoint {
                    time: 0,
                    beat_time: r32(60.0 / 150.0),
                };
            }
            Err(i) => i.saturating_sub(1),
        };
        self.points
            .get(i)
            .expect("already checked for no timings available")
            .clone()
    }

    pub fn snap_to_beat(&self, time: Time, snap: BeatTime) -> Time {
        let timing = self.get_timing(time);
        let delta = time_to_seconds(time - timing.time);
        let snap_time = snap.as_secs(timing.beat_time);
        let delta = (delta / snap_time).round() * snap_time;
        let delta = seconds_to_time(delta);
        timing.time + delta
    }
}

impl TimedEvent {
    /// Returns the duration (in beats) of the event.
    pub fn duration(&self) -> Time {
        match &self.event {
            Event::Light(event) => event.movement.total_duration(),
            Event::PaletteSwap => Time::ZERO,
        }
    }
}

impl LightEvent {
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
