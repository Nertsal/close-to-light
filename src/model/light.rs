use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    /// Currently active collider.
    pub collider: Collider,
    /// The base collider used for reference.
    pub base_collider: Collider,
    // pub movement: Movement,
    /// Time since creation.
    pub lifetime: Time,
    /// Whether the light is dangerous.
    pub danger: bool,
    /// Id of the original event in the level.
    pub event_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct LightTelegraph {
    /// The light to telegraph.
    pub light: Light,
    /// How fast the telegraph is.
    pub speed: Coord,
    /// Time since creation.
    pub lifetime: Time,
    // /// The time (in beats) until the actual light is spawned.
    // pub spawn_timer: Time,
}

impl Light {
    pub fn into_telegraph(self, telegraph: Telegraph) -> LightTelegraph {
        LightTelegraph {
            light: self,
            speed: telegraph.speed,
            lifetime: Time::ZERO,
            // spawn_timer: telegraph.precede_time,
        }
    }
}
