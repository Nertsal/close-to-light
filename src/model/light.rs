use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    /// Currently active collider.
    pub collider: Collider,
    /// The base collider used for reference.
    pub base_collider: Collider,
    pub movement: Movement,
    pub lifetime: Lifetime,
}

#[derive(Debug, Clone)]
pub struct LightTelegraph {
    /// The light to telegraph.
    pub light: Light,
    /// Lifetime of the telegraph.
    pub lifetime: Lifetime,
    /// The time until the actual light is spawned.
    pub spawn_timer: Time,
}

impl Light {
    pub fn into_telegraph(self, telegraph: Telegraph, beat_time: Time) -> LightTelegraph {
        LightTelegraph {
            light: self,
            lifetime: Lifetime::new_max(telegraph.duration * beat_time),
            spawn_timer: telegraph.precede_time * beat_time,
        }
    }
}
