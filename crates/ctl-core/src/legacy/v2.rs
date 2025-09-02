use geng::prelude::{Angle, R32, r32, vec2};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, rc::Rc, sync::Arc};

type Id = u32;
type Coord = R32;
type Time = i64;
type FloatTime = R32;
type Name = Arc<str>;

const TIME_IN_FLOAT_TIME: Time = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelSet<L = LevelFull> {
    pub id: Id,
    pub owner: UserInfo,
    pub levels: Vec<L>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelFull {
    pub meta: LevelInfo,
    pub data: Level,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LevelSetInfo {
    /// Id `0` for local groups.
    #[serde(default)]
    pub id: Id,
    pub music: MusicInfo,
    pub owner: UserInfo,
    pub levels: Vec<LevelInfo>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMeta {
    pub music: Option<MusicInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicInfo {
    /// Id `0` for local music.
    #[serde(default)]
    pub id: Id,
    pub original: bool,
    pub name: Name,
    pub romanized: Name,
    pub authors: Vec<MusicianInfo>,
}

impl Default for MusicInfo {
    fn default() -> Self {
        Self {
            id: 0,
            original: false,
            name: "<name>".into(),
            romanized: "<romanized>".into(),
            authors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MusicianInfo {
    pub id: Id,
    pub name: Name,
    pub romanized: Name,
    pub user: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LevelInfo {
    /// Id `0` for local levels.
    pub id: Id,
    pub name: Name,
    pub authors: Vec<UserInfo>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    pub id: Id,
    pub name: Name,
}

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Telegraph {
    /// How long (in beats) before the event should the telegraph occur.
    pub precede_time: Time,
    /// How fast the telegraph is.
    pub speed: Coord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Shape {
    Circle { radius: Coord },
    Line { width: Coord },
    Rectangle { width: Coord, height: Coord },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    /// Time (in milliseconds) to spend fading into the initial position.
    pub fade_in: Time,
    /// Time (in milliseconds) to spend fading out of the last keyframe.
    pub fade_out: Time,
    pub initial: Transform,
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    #[serde(default)]
    pub curve: TrajectoryInterpolation,
    pub key_frames: VecDeque<MoveFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    /// How long (in beats) should the interpolation from the last frame to this frame last.
    pub lerp_time: Time,
    /// Interpolation to use when moving away from this frame to the next.
    #[serde(default)]
    pub interpolation: MoveInterpolation,
    /// Whether to start a new curve going towards from this frame further.
    /// If set to `None`, the curve will continue as the previous type.
    pub change_curve: Option<TrajectoryInterpolation>,
    pub transform: Transform,
}

/// Controls the speed of the light when moving between keyframes.
/// Default is Smoothstep.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
pub enum TrajectoryInterpolation {
    /// Connects keyframes in a straight line.
    #[default]
    Linear,
    /// Connects keyframes via a smooth cubic Cardinal spline.
    Spline { tension: R32 },
    /// Connects keyframes via a Bezier curve.
    Bezier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaypointId {
    Initial,
    Frame(usize),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Transform {
    pub translation: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub scale: Coord,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: vec2::ZERO,
            rotation: Angle::ZERO,
            scale: r32(1.0),
        }
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self::new(TIME_IN_FLOAT_TIME / 2, Transform::default())
    }
}

impl Movement {
    pub fn new(fade_time: Time, initial: Transform) -> Self {
        Self {
            fade_in: fade_time,
            fade_out: fade_time,
            initial,
            interpolation: MoveInterpolation::default(),
            curve: TrajectoryInterpolation::default(),
            key_frames: VecDeque::new(),
        }
    }
}

pub fn convert_group(value: LevelSet) -> (crate::LevelSet, crate::LevelSetInfo) {
    let levels_info = value
        .levels
        .iter()
        .map(|level| {
            crate::LevelInfo {
                id: level.meta.id,
                name: level.meta.name.clone(),
                authors: level
                    .meta
                    .authors
                    .iter()
                    .map(|user| crate::MapperInfo {
                        id: user.id,
                        name: user.name.clone(),
                        romanized: user.name.clone(),
                    })
                    .collect(),
                hash: level.meta.hash.clone(), // TODO: should i recalculate the hash?
            }
        })
        .collect();
    let level_set = crate::LevelSet {
        levels: value
            .levels
            .into_iter()
            .map(|level| Rc::new(convert_level(level.data)))
            .collect(),
    };
    let hash = level_set.calculate_hash();
    (
        level_set,
        crate::LevelSetInfo {
            id: value.id,
            owner: crate::UserInfo {
                id: value.owner.id,
                name: value.owner.name,
            },
            music: crate::MusicInfo::default(),
            levels: levels_info,
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
                            fade_in: light.movement.fade_in,
                            fade_out: light.movement.fade_out,
                            initial: light.movement.initial.into(),
                            interpolation: crate::MoveInterpolation::default(),
                            curve: crate::TrajectoryInterpolation::default(),
                            key_frames: light
                                .movement
                                .key_frames
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                        },
                    }),
                    Event::PaletteSwap => crate::Event::PaletteSwap,
                },
            })
            .collect(),
        timing: value.timing.into(),
    }
}

impl From<Transform> for crate::Transform {
    fn from(value: Transform) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation,
            scale: value.scale,
        }
    }
}

impl From<MoveFrame> for crate::MoveFrame {
    fn from(value: MoveFrame) -> Self {
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
            featured: false,
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
