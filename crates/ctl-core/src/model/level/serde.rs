use super::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimingPoint {
    /// The time from which this timing applies.
    pub time: Time,
    /// Time for a single beat (in seconds).
    pub beat_time: FloatTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedEvent {
    /// The time on which the event should happen.
    pub time: Time,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    Light(LightEvent),
    Effect(EffectEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectEvent {
    /// Swap light and dark colors.
    /// Time specifies the duration of the **transition**.
    PaletteSwap(Time),
    /// Apply an RGB-splitting shader to the screen.
    /// Time specifies the duration of the **effect**.
    RgbSplit(Time),
    /// Apply a screen shake effect to the camera.
    /// Time specifies the duration of the **effect**.
    /// R32 specifies the intensity/amplitude.
    CameraShake(Time, R32),
    /// Apply a CRT screen vignette effect.
    /// Time specifies the duration of the **effect**.
    /// R32 specifies the intensity/darkness.
    Vignette(Time, R32),
    /// Apply a CRT screen curvature effect.
    /// Time specifies the duration of the **effect**.
    /// R32 specifies the intensity/curvature.
    ScreenCurvature(Time, R32),
    /// Apply a horizontal noise offset effect.
    /// Time specifies the duration of the **effect**.
    /// R32 specifies the intensity/curvature.
    NoiseOffset(Time, R32),
    /// Apply a spotlight vision effect.
    /// Time specifies the duration of the **effect**.
    /// R32 specifies the dimming value (0-1).
    Spotlight(Time, R32),
    /// Apply a transform to the camera orientation.
    /// The camera is interpolated between each camera transform event.
    Camera(CameraTransform, MoveInterpolation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CameraTransform {
    pub rotation: Angle<Coord>,
    pub zoom: Coord,
}

impl Default for CameraTransform {
    fn default() -> Self {
        Self {
            rotation: Angle::ZERO,
            zoom: Coord::ONE,
        }
    }
}

impl Interpolatable for CameraTransform {
    fn add(self, other: Self) -> Self {
        Self {
            rotation: Interpolatable::add(self.rotation, other.rotation),
            zoom: Interpolatable::add(self.zoom, other.zoom),
        }
    }

    fn sub(self, other: Self) -> Self {
        Self {
            rotation: Interpolatable::sub(self.rotation, other.rotation),
            zoom: Interpolatable::sub(self.zoom, other.zoom),
        }
    }

    fn scale(self, factor: f32) -> Self {
        Self {
            rotation: Interpolatable::scale(self.rotation, factor),
            zoom: Interpolatable::scale(self.zoom, factor),
        }
    }

    fn length_sqr(self) -> f32 {
        self.rotation.length_sqr() + self.zoom.length_sqr()
    }
}

impl EffectEvent {
    pub fn duration(&self) -> Option<Time> {
        match self {
            EffectEvent::PaletteSwap(duration)
            | EffectEvent::RgbSplit(duration)
            | EffectEvent::CameraShake(duration, _)
            | EffectEvent::Vignette(duration, _)
            | EffectEvent::ScreenCurvature(duration, _)
            | EffectEvent::NoiseOffset(duration, _)
            | EffectEvent::Spotlight(duration, _) => Some(*duration),
            EffectEvent::Camera(..) => None,
        }
    }

    pub fn duration_mut(&mut self) -> Option<&mut Time> {
        match self {
            EffectEvent::PaletteSwap(duration)
            | EffectEvent::RgbSplit(duration)
            | EffectEvent::CameraShake(duration, _)
            | EffectEvent::Vignette(duration, _)
            | EffectEvent::ScreenCurvature(duration, _)
            | EffectEvent::NoiseOffset(duration, _)
            | EffectEvent::Spotlight(duration, _) => Some(duration),
            EffectEvent::Camera(..) => None,
        }
    }

    pub fn intensity_mut(&mut self) -> Option<&mut R32> {
        match self {
            EffectEvent::CameraShake(_, intensity)
            | EffectEvent::Vignette(_, intensity)
            | EffectEvent::ScreenCurvature(_, intensity)
            | EffectEvent::NoiseOffset(_, intensity)
            | EffectEvent::Spotlight(_, intensity) => Some(intensity),
            _ => None,
        }
    }
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
    pub fn new(bpm: FloatTime) -> Self {
        Self {
            events: Vec::new(),
            timing: Timing::new(bpm),
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
    pub fn new(bpm: FloatTime) -> Self {
        Self {
            points: vec![TimingPoint {
                time: Time::ZERO,
                beat_time: r32(60.0) / bpm,
            }],
        }
    }

    pub fn get_timing(&self, time: Time) -> TimingPoint {
        let i = match self
            .points
            .binary_search_by_key(&time, |timing| timing.time)
        {
            Ok(i) => i,
            Err(0) => {
                // Assume timing before 0 is the same as the first timing point
                if self.points.is_empty() {
                    // no timing points smh
                    // log::error!("level has no timing points");
                    return TimingPoint {
                        time: 0,
                        beat_time: r32(60.0 / 150.0),
                    };
                }
                0
            }
            Err(i) => i.saturating_sub(1),
        };
        self.points
            .get(i)
            .expect("already checked for no timings available")
            .clone()
    }

    pub fn get_timing_index(&self, time: Time) -> usize {
        match self.points.binary_search_by_key(&time, |point| point.time) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    }

    pub fn snap_to_beat(&self, time: Time, snap: BeatTime) -> Time {
        let timing = self.get_timing(time);
        let delta = time_to_seconds(time - timing.time);
        let snap_time = snap.as_secs(timing.beat_time);
        let delta = (delta / snap_time).round() * snap_time;
        let delta = seconds_to_time(delta);
        timing.time + delta
    }

    // /// Returns the approximation of the beat time.
    // pub fn approximate_beat(&self, time: Time) -> BeatTime {
    //     let timing = self.get_timing(time);
    //     let beat_time = time_to_seconds(time - timing.time) / timing.beat_time;
    //     BeatTime::from_beats_float(beat_time)
    // }

    pub fn is_beat_aligned(&self, time: Time) -> Option<BeatTime> {
        // let beat_ratio = num_rational::Ratio::new(
        //     self.approximate_beat(time).units(),
        //     BeatTime::UNITS_PER_BEAT,
        // );
        // // Small enough subdivision to consider it a valid beat point
        // let subdivision = *beat_ratio.denom();
        // (subdivision <= 21).then(|| BeatTime::from_units(BeatTime::UNITS_PER_BEAT / subdivision))

        let (aligned, snap) = self.snap_to_best_alignment(time);
        (aligned == time).then_some(snap)
    }

    /// Return the closest reasonable beat subdivision that this time falls onto.
    pub fn snap_to_best_alignment(&self, time: Time) -> (Time, BeatTime) {
        let max_error: Time = 1;
        let subdivisions = [1, 2, 4, 8, 16, 3, 9, 12, 5, 15, 7, 21];
        let mut best_error = max_error + 1;
        let mut best_snap = BeatTime::UNIT;
        let mut best_time = time;
        for subdiv in subdivisions {
            let snap = BeatTime::from_units(BeatTime::UNITS_PER_BEAT / subdiv);
            let aligned = self.snap_to_beat(time, snap);
            let error = (time - aligned).abs();
            if error <= max_error && error < best_error {
                best_snap = snap;
                best_error = error;
                best_time = aligned
            }
        }
        (best_time, best_snap)
    }

    /// Snaps to beat while ignoring a specified timing point.
    pub fn snap_to_beat_without(&self, ignored: usize, time: Time, snap: BeatTime) -> Time {
        let mut timing_i = self.get_timing_index(time);
        if timing_i == ignored {
            if timing_i == 0 {
                timing_i = 1;
            } else {
                timing_i -= 1;
            }
        }
        let timing = self
            .points
            .get(timing_i)
            .cloned()
            .unwrap_or_else(|| self.get_timing(time));

        let delta = time_to_seconds(time - timing.time);
        let snap_time = snap.as_secs(timing.beat_time);
        let delta = (delta / snap_time).round() * snap_time;
        let delta = seconds_to_time(delta);
        timing.time + delta
    }

    /// Calculates the beat time relative to the most recent timing point.
    pub fn get_relative_beat_time(&self, time: Time) -> BeatTime {
        let timing = self.get_timing(time);
        let delta = time_to_seconds(time - timing.time);
        BeatTime::from_beats_float(delta / timing.beat_time)
    }
}

impl TimedEvent {
    /// Returns the duration of the event.
    pub fn duration(&self) -> Time {
        match &self.event {
            Event::Light(event) => event.movement.duration(),
            Event::Effect(_) => Time::ZERO,
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
            hollow: r32(-1.0),
            event_id,
            closest_waypoint: (Time::ZERO, WaypointId::Initial),
        }
    }
}

#[test]
fn test_snap_consistency() {
    let timing = Timing::new(r32(200.0));

    assert_eq!(timing.snap_to_beat(0, BeatTime::WHOLE), 0);
    assert_eq!(timing.snap_to_beat(0, BeatTime::HALF), 0);

    for i in 0..1000 {
        for subdiv in 0..4 {
            let time =
                seconds_to_time(timing.points[0].beat_time * r32(i as f32 + subdiv as f32 / 4.0));
            let time = timing.snap_to_beat(time, BeatTime::QUARTER);
            if subdiv == 0 {
                assert_eq!(
                    time,
                    timing.snap_to_beat(time, BeatTime::WHOLE),
                    "Iteration {}, subdivision {}, time = {}, snap: WHOLE",
                    i,
                    subdiv,
                    time
                );
            }
            assert_eq!(
                time,
                timing.snap_to_beat(time, BeatTime::EIGHTH),
                "Iteration {}, subdivision {}, time = {}, snap: EIGHTH",
                i,
                subdiv,
                time
            );
            assert_eq!(
                time,
                timing.snap_to_beat(time, BeatTime::SIXTITH),
                "Iteration {}, subdivision {}, time = {}, snap: SIXTITH",
                i,
                subdiv,
                time
            );
            assert_eq!(
                time,
                timing.snap_to_beat(time, BeatTime::UNIT),
                "Iteration {}, subdivision {}, time = {}, snap: UNIT",
                i,
                subdiv,
                time
            );
        }
    }
}

#[test]
fn test_beat_alignment() {
    let timing = Timing::new(r32(180.0));
    // times off by one are fine to consider either way
    assert!(timing.is_beat_aligned(665).is_none());
    assert!(timing.is_beat_aligned(667).is_some());
    assert!(timing.is_beat_aligned(669).is_none());
}
