use geng::prelude::{Angle, R32, r32, vec2};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, rc::Rc, sync::Arc};

type Id = u32;
type Coord = R32;
type Time = R32;
type Name = Arc<str>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelSet<L = LevelFull> {
    pub id: Id,
    pub music: Id,
    pub owner: UserInfo,
    #[serde(default = "Vec::new")]
    pub levels: Vec<L>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelFull {
    pub meta: LevelInfo,
    pub data: Level,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Shape {
    Circle { radius: Coord },
    Line { width: Coord },
    Rectangle { width: Coord, height: Coord },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    /// Time (in beats) to spend fading into the initial position.
    pub fade_in: Time,
    /// Time (in beats) to spend fading out of the last keyframe.
    pub fade_out: Time,
    pub initial: Transform,
    pub key_frames: VecDeque<MoveFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    /// How long (in beats) should the interpolation from the last frame to that frame last.
    pub lerp_time: Time,
    pub transform: Transform,
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
        Self {
            fade_in: r32(1.0),
            fade_out: r32(1.0),
            initial: Transform::default(),
            key_frames: VecDeque::new(),
        }
    }
}

pub fn migrate(
    beat_time: crate::FloatTime,
    value: LevelSet,
) -> (crate::LevelSet, crate::LevelSetInfo) {
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
            .map(|level| Rc::new(convert_level(beat_time, level.data)))
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
            featured: false,
            hash,
        },
    )
}

fn convert_level(beat_time: crate::FloatTime, value: Level) -> crate::Level {
    crate::Level {
        events: value
            .events
            .into_iter()
            .map(|event| crate::TimedEvent {
                time: convert_time(
                    beat_time,
                    event.beat
                        + match &event.event {
                            Event::Light(light) => light.telegraph.precede_time,
                            Event::PaletteSwap => r32(0.0),
                        },
                ),
                event: match event.event {
                    Event::Light(light) => crate::Event::Light(crate::LightEvent {
                        danger: light.light.danger,
                        hollow: None,
                        shape: match light.light.shape {
                            Shape::Circle { radius } => crate::Shape::Circle { radius },
                            Shape::Line { width } => crate::Shape::Line { width },
                            Shape::Rectangle { width, height } => {
                                crate::Shape::Rectangle { width, height }
                            }
                        },
                        movement: crate::Movement {
                            fade_in: convert_time(beat_time, light.light.movement.fade_in),
                            fade_out: convert_time(beat_time, light.light.movement.fade_out),
                            initial: light.light.movement.initial.into(),
                            interpolation: crate::MoveInterpolation::default(),
                            curve: crate::TrajectoryInterpolation::default(),
                            key_frames: light
                                .light
                                .movement
                                .key_frames
                                .into_iter()
                                .map(|frame| convert_frame(beat_time, frame))
                                .collect(),
                        },
                    }),
                    Event::PaletteSwap => {
                        crate::Event::Effect(crate::model::EffectEvent::PaletteSwap(
                            crate::types::TIME_IN_FLOAT_TIME / 2,
                        ))
                    }
                },
            })
            .collect(),
        timing: crate::Timing {
            points: vec![crate::TimingPoint { time: 0, beat_time }],
        },
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

fn convert_frame(beat_time: crate::FloatTime, value: MoveFrame) -> crate::MoveFrame {
    crate::MoveFrame {
        lerp_time: convert_time(beat_time, value.lerp_time),
        interpolation: crate::MoveInterpolation::default(),
        change_curve: None, // Linear
        transform: value.transform.into(),
    }
}

fn convert_time(beat_time: crate::FloatTime, time: crate::FloatTime) -> crate::Time {
    crate::seconds_to_time(beat_time * time)
}
