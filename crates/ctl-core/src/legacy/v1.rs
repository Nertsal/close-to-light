use geng::prelude::{r32, vec2, Angle, R32};
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

impl From<LevelSet> for crate::LevelSet {
    fn from(value: LevelSet) -> Self {
        Self {
            id: value.id,
            music: value.music,
            owner: crate::UserInfo {
                id: value.owner.id,
                name: value.owner.name,
            },
            levels: value
                .levels
                .into_iter()
                .map(|level| {
                    Rc::new(crate::LevelFull {
                        meta: crate::LevelInfo {
                            id: level.meta.id,
                            name: level.meta.name,
                            authors: level
                                .meta
                                .authors
                                .into_iter()
                                .map(|user| crate::UserInfo {
                                    id: user.id,
                                    name: user.name,
                                })
                                .collect(),
                            hash: level.meta.hash, // TODO: should i recalculate the hash?
                        },
                        data: level.data.into(),
                    })
                })
                .collect(),
        }
    }
}

impl From<Level> for crate::Level {
    fn from(value: Level) -> Self {
        Self {
            events: value
                .events
                .into_iter()
                .map(|event| crate::TimedEvent {
                    beat: event.beat,
                    event: match event.event {
                        Event::Light(light) => crate::Event::Light(crate::LightEvent {
                            light: crate::LightSerde {
                                danger: light.light.danger,
                                shape: match light.light.shape {
                                    Shape::Circle { radius } => crate::Shape::Circle { radius },
                                    Shape::Line { width } => crate::Shape::Line { width },
                                    Shape::Rectangle { width, height } => {
                                        crate::Shape::Rectangle { width, height }
                                    }
                                },
                                movement: crate::Movement {
                                    fade_in: light.light.movement.fade_in,
                                    fade_out: light.light.movement.fade_out,
                                    initial: light.light.movement.initial.into(),
                                    curve: crate::TrajectoryInterpolation::default(),
                                    key_frames: light
                                        .light
                                        .movement
                                        .key_frames
                                        .into_iter()
                                        .map(From::from)
                                        .collect(),
                                },
                            },
                            telegraph: crate::Telegraph {
                                precede_time: light.telegraph.precede_time,
                                speed: light.telegraph.speed,
                            },
                        }),
                        Event::PaletteSwap => crate::Event::PaletteSwap,
                    },
                })
                .collect(),
        }
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
            interpolation: crate::MoveInterpolation::default(),
            change_curve: None, // Linear
            transform: value.transform.into(),
        }
    }
}
