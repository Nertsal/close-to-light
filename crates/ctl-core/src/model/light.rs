use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    /// Currently active collider.
    pub collider: Collider,
    /// The base collider used for reference.
    pub base_collider: Collider,
    /// Time since creation.
    pub lifetime: Time,
    /// Whether the light is dangerous.
    pub danger: bool,
    /// Makes the light hollow.
    pub hollow: Option<R32>,
    /// Id of the original event in the level.
    pub event_id: Option<usize>,
    /// Time delta to the closest waypoint.
    pub closest_waypoint: (Time, WaypointId),
}

#[derive(Debug, Clone)]
pub struct LightTelegraph {
    /// The light to telegraph.
    pub light: Light,
    /// Time since creation.
    pub lifetime: Time,
}

impl Light {
    pub fn into_telegraph(self) -> LightTelegraph {
        LightTelegraph {
            light: self,
            lifetime: Time::ZERO,
        }
    }
}
