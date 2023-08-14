use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    /// Currently active collider.
    pub collider: Collider,
    /// The base collider used for reference.
    pub base_collider: Collider,
    pub movement: Movement,
    /// Time since creation.
    pub lifetime: Time,
}

#[derive(Debug, Clone)]
pub struct LightTelegraph {
    /// The light to telegraph.
    pub light: Light,
    /// How fast the telegraph is.
    pub speed: Coord,
    /// Time since creation.
    pub lifetime: Time,
    /// The time until the actual light is spawned.
    pub spawn_timer: Time,
}

impl Light {
    pub fn into_telegraph(self, telegraph: Telegraph, beat_time: Time) -> LightTelegraph {
        LightTelegraph {
            light: self,
            speed: telegraph.speed,
            lifetime: Time::ZERO,
            spawn_timer: telegraph.precede_time * beat_time,
        }
    }
}
