use geng::prelude::{Angle, R32, UNum, r32, vec2};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, rc::Rc, sync::Arc};

type Id = u32;
type Coord = R32;
type Time = i64;
type FloatTime = R32;
type Name = Arc<str>;

const TIME_IN_FLOAT_TIME: Time = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LevelSet<L = Level> {
    pub levels: Vec<L>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelSetFull {
    pub meta: LevelSetInfo,
    pub data: LevelSet<Level>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LevelSetInfo {
    /// Id `0` for local groups.
    #[serde(default)]
    pub id: Id,
    pub music: MusicInfo,
    pub owner: UserInfo,
    #[serde(default)]
    pub levels: Vec<LevelInfo>,
    #[serde(default)]
    pub featured: bool,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MusicInfo {
    /// Id `0` for local music.
    #[serde(default)]
    pub id: Id,
    #[serde(default)]
    pub original: bool,
    #[serde(default)]
    pub featured: bool,
    pub name: Name,
    pub romanized: Name,
    #[serde(default)]
    pub authors: Vec<MusicianInfo>,
}

impl Default for MusicInfo {
    fn default() -> Self {
        Self {
            id: 0,
            original: false,
            featured: false,
            name: "<name>".into(),
            romanized: "<romanized>".into(),
            authors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct LevelInfo {
    /// Id `0` for local levels.
    pub id: Id,
    pub name: Name,
    pub authors: Vec<MapperInfo>,
    pub hash: String,
}

impl Default for LevelInfo {
    fn default() -> Self {
        Self {
            id: 0,
            name: "<level>".into(),
            authors: Vec::new(),
            hash: "".into(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UserInfo {
    pub id: Id,
    pub name: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MapperInfo {
    /// User id `0` for non-registered mapper.
    pub id: Id,
    pub name: Name,
    pub romanized: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicianInfo {
    /// Id `0` for non-registered musicians.
    pub id: Id,
    pub name: Name,
    pub romanized: Name,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Level {
    pub events: Vec<TimedEvent>,
    pub timing: Timing,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Timing {
    /// Points are assumed to be sorted by time.
    pub points: Vec<TimingPoint>,
}

/// A timing point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TimingPoint {
    /// The time from which this timing applies.
    pub time: Time,
    /// Time for a single beat (in seconds).
    pub beat_time: FloatTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TimedEvent {
    /// The time on which the event should happen.
    pub time: Time,
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Event {
    Light(LightEvent),
    Effect(EffectEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum EffectEvent {
    /// Swap light and dark colors.
    /// Time specifies the duration of the **transition**.
    PaletteSwap(Time),
    /// Apply an RGB-splitting shader to the screen.
    /// Time specifies the duration of the **effect**.
    RgbSplit(Time),
    /// Apply an screen shake effect to the camera.
    /// Time specifies the duration of the **effect**.
    /// R32 specifies the intensity/amplitude.
    CameraShake(Time, R32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum Shape {
    Circle { radius: Coord },
    Line { width: Coord },
    Rectangle { width: Coord, height: Coord },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LightEvent {
    /// Whether the light is dangerous.
    #[serde(default)]
    pub danger: bool,
    pub shape: Shape,
    /// Movement with timings in beats.
    #[serde(default)]
    pub movement: Movement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Movement {
    /// The spawn waypoint that is the very first transformation of the light when it appears on the screen.
    /// Typically it is used to setup the *fade in* effect from scale zero.
    pub initial: WaypointInitial,
    pub waypoints: VecDeque<Waypoint>,
    /// The final waypoint that is the very last transformation of the light when it is visible on the screen.
    /// Typically it is used to setup the *fade out* effect to scale zero.
    pub last: TransformLight,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WaypointInitial {
    /// Duration of the interpolation from this frame to the next.
    pub lerp_time: Time,
    /// Interpolation to use when moving away from this frame to the next.
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    /// The initial curve going from this frame to the next.
    pub curve: TrajectoryInterpolation,
    #[serde(default)]
    pub transform: TransformLight,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Waypoint {
    /// Duration of the interpolation from this frame to the next.
    pub lerp_time: Time,
    /// Interpolation to use when moving away from this frame to the next.
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    /// Whether to start a new curve going from this frame to the next.
    /// If set to `None`, the curve will continue as the previous type.
    pub change_curve: Option<TrajectoryInterpolation>,
    pub transform: TransformLight,
}

/// Controls the speed of the light when moving between keyframes.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum MoveInterpolation {
    Linear,
    #[default]
    Smoothstep,
    EaseIn,
    EaseOut,
}

/// Controls the overall trajectory of the light based on the keyframes.
/// Default is Linear.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum TrajectoryInterpolation {
    /// Connects keyframes in a straight line.
    #[default]
    Linear,
    /// Connects keyframes via a smooth cubic Cardinal spline.
    Spline { tension: R32 },
    /// Connects keyframes via a Bezier curve.
    Bezier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(deny_unknown_fields)]
pub enum WaypointId {
    Initial,
    Frame(usize),
    Last,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct TransformLight {
    pub translation: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub scale: Coord,
    /// Turns the light hollow (circle lights become rings),
    /// with the value indicating the level of hollowness (inner cut ratio).
    /// Expected range: `-1..=1`.
    pub hollow: R32,
}

impl Default for TransformLight {
    fn default() -> Self {
        Self {
            translation: vec2::ZERO,
            rotation: Angle::ZERO,
            scale: Coord::ONE,
            hollow: r32(-1.0),
        }
    }
}

impl Waypoint {
    pub fn new(lerp_time: Time, transform: TransformLight) -> Self {
        Self {
            lerp_time,
            interpolation: MoveInterpolation::default(),
            change_curve: None,
            transform,
        }
    }
}

impl Default for Movement {
    fn default() -> Self {
        let fade_time = TIME_IN_FLOAT_TIME / 2;
        let initial = TransformLight::default();
        Self {
            // Fade in
            initial: WaypointInitial {
                lerp_time: fade_time,
                interpolation: MoveInterpolation::default(),
                curve: TrajectoryInterpolation::default(),
                transform: TransformLight {
                    scale: R32::ZERO,
                    ..initial
                },
            },
            waypoints: vec![Waypoint::new(fade_time, initial)].into(),
            // Fade out
            last: TransformLight {
                scale: R32::ZERO,
                ..initial
            },
        }
    }
}

pub fn migrate(value: LevelSet, info: LevelSetInfo) -> (crate::LevelSet, crate::LevelSetInfo) {
    let levels_info = info
        .levels
        .iter()
        .map(|level| {
            crate::LevelInfo {
                id: level.id,
                name: level.name.clone(),
                authors: level
                    .authors
                    .iter()
                    .map(|user| crate::MapperInfo {
                        id: user.id,
                        name: user.name.clone(),
                        romanized: user.romanized.clone(),
                    })
                    .collect(),
                hash: level.hash.clone(), // TODO: should i recalculate the hash?
            }
        })
        .collect();
    let level_set = crate::LevelSet {
        levels: value
            .levels
            .into_iter()
            .map(|level| Rc::new(convert_level(level)))
            .collect(),
    };
    let hash = level_set.calculate_hash();
    (
        level_set,
        crate::LevelSetInfo {
            id: info.id,
            owner: crate::UserInfo {
                id: info.owner.id,
                name: info.owner.name,
            },
            music: crate::MusicInfo {
                id: info.music.id,
                original: info.music.original,
                featured: info.music.featured,
                name: info.music.name,
                romanized: info.music.romanized,
                authors: info
                    .music
                    .authors
                    .into_iter()
                    .map(|author| crate::MusicianInfo {
                        id: author.id,
                        name: author.name,
                        romanized: author.romanized,
                    })
                    .collect(),
            },
            levels: levels_info,
            featured: info.featured,
            hash,
        },
    )
}

fn convert_level(value: Level) -> crate::Level {
    crate::Level {
        events: value
            .events
            .into_iter()
            .map(|event| crate::TimedEvent {
                time: event.time,
                event: match event.event {
                    Event::Light(light) => crate::Event::Light(crate::LightEvent {
                        danger: light.danger,
                        shape: match light.shape {
                            Shape::Circle { radius } => crate::Shape::Circle { radius },
                            Shape::Line { width } => crate::Shape::Line { width },
                            Shape::Rectangle { width, height } => {
                                crate::Shape::Rectangle { width, height }
                            }
                        },
                        movement: crate::Movement {
                            // Fade in
                            initial: crate::WaypointInitial {
                                lerp_time: light.movement.initial.lerp_time,
                                interpolation: light.movement.initial.interpolation.into(),
                                curve: light.movement.initial.curve.into(),
                                transform: light.movement.initial.transform.into(),
                            },
                            // Fade out
                            last: light.movement.last.into(),
                            waypoints: light
                                .movement
                                .waypoints
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                        },
                    }),
                    Event::Effect(effect) => crate::Event::Effect(match effect {
                        EffectEvent::PaletteSwap(duration) => {
                            crate::model::EffectEvent::PaletteSwap(duration)
                        }
                        EffectEvent::RgbSplit(duration) => {
                            crate::model::EffectEvent::RgbSplit(duration)
                        }
                        EffectEvent::CameraShake(duration, intensity) => {
                            crate::model::EffectEvent::CameraShake(duration, intensity)
                        }
                    }),
                },
            })
            .collect(),
        timing: value.timing.into(),
    }
}

impl From<TransformLight> for crate::TransformLight {
    fn from(value: TransformLight) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation,
            scale: value.scale,
            hollow: value.hollow,
        }
    }
}

impl From<Waypoint> for crate::Waypoint {
    fn from(value: Waypoint) -> Self {
        Self {
            lerp_time: value.lerp_time,
            interpolation: value.interpolation.into(),
            change_curve: value.change_curve.map(Into::into),
            transform: value.transform.into(),
        }
    }
}

impl From<MoveInterpolation> for crate::MoveInterpolation {
    fn from(value: MoveInterpolation) -> Self {
        match value {
            MoveInterpolation::Linear => Self::Linear,
            MoveInterpolation::Smoothstep => Self::Smoothstep,
            MoveInterpolation::EaseIn => Self::EaseIn,
            MoveInterpolation::EaseOut => Self::EaseOut,
        }
    }
}

impl From<TrajectoryInterpolation> for crate::TrajectoryInterpolation {
    fn from(value: TrajectoryInterpolation) -> Self {
        match value {
            TrajectoryInterpolation::Linear => Self::Linear,
            TrajectoryInterpolation::Spline { tension } => Self::Spline { tension },
            TrajectoryInterpolation::Bezier => Self::Bezier,
        }
    }
}

impl From<Timing> for crate::Timing {
    fn from(value: Timing) -> Self {
        Self {
            points: value.points.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TimingPoint> for crate::TimingPoint {
    fn from(value: TimingPoint) -> Self {
        Self {
            time: value.time,
            beat_time: value.beat_time,
        }
    }
}

impl From<MusicInfo> for crate::MusicInfo {
    fn from(value: MusicInfo) -> Self {
        Self {
            id: value.id,
            original: value.original,
            featured: value.featured,
            name: value.name,
            romanized: value.romanized,
            authors: value.authors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<MusicianInfo> for crate::MusicianInfo {
    fn from(value: MusicianInfo) -> Self {
        Self {
            id: value.id,
            name: value.name,
            romanized: value.romanized,
        }
    }
}
