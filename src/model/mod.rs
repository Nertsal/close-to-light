mod logic;

use geng::prelude::*;

pub type Time = R32;
pub type Coord = R32;

pub struct Light {
    pub position: vec2<Coord>,
    pub radius: Coord,
}

pub struct Model {
    pub camera: Camera2d,
    /// The time until the next music beat.
    pub beat_timer: Time,
    pub lights: Vec<Light>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            beat_timer: Time::ONE,
            lights: vec![],
        }
    }
}
