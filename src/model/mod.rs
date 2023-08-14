mod collider;
mod config;
mod logic;

pub use self::collider::*;
pub use self::config::*;

use geng::prelude::*;
use geng_utils::{bounded::Bounded, conversions::Vec2RealConversions};

pub type Time = R32;
pub type Coord = R32;
pub type Lifetime = Bounded<Time>;

#[derive(Debug, Clone)]
pub struct Light {
    pub collider: Collider,
    pub shape_max: Shape,
    pub lifetime: Lifetime,
}

#[derive(Debug, Clone)]
pub struct Telegraph {
    /// The light to telegraph.
    pub light: Light,
    /// Lifetime of the telegraph.
    pub lifetime: Lifetime,
    /// The time until the actual light is spawned.
    pub spawn_timer: Time,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub target_position: vec2<Coord>,
    pub shake: vec2<Coord>,
    pub collider: Collider,
    pub fear_meter: Bounded<Time>,
}

pub struct Model {
    pub config: Config,
    pub camera: Camera2d,
    /// The time until the next music beat.
    pub beat_timer: Time,
    pub player: Player,
    pub telegraphs: Vec<Telegraph>,
    pub lights: Vec<Light>,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            beat_timer: Time::ZERO,
            player: Player {
                target_position: vec2::ZERO,
                shake: vec2::ZERO,
                collider: Collider::new(vec2::ZERO, Shape::Circle { radius: r32(0.2) }),
                fear_meter: Bounded::new_max(r32(1.0)),
            },
            telegraphs: vec![],
            lights: vec![],
        }
    }
}
