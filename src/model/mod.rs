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

pub struct Light {
    pub collider: Collider,
    pub shape_max: Shape,
    pub lifetime: Lifetime,
}

pub struct Player {
    pub collider: Collider,
    pub fear_meter: Bounded<Time>,
}

pub struct Model {
    pub rules: Config,
    pub camera: Camera2d,
    /// The time until the next music beat.
    pub beat_timer: Time,
    pub player: Player,
    pub lights: Vec<Light>,
}

impl Model {
    pub fn new(rules: Config) -> Self {
        Self {
            rules,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            beat_timer: Time::ONE,
            player: Player {
                collider: Collider::new(vec2::ZERO, Shape::Circle { radius: r32(0.2) }),
                fear_meter: Bounded::new_max(r32(1.0)),
            },
            lights: vec![],
        }
    }
}
