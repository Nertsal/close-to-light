use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    pub player: PlayerConfig,
    pub health: HealthConfig,
    /// Possible light shapes to choose from.
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Max health value.
    pub max: Time,
    /// How fast health decreases per second.
    pub decrease_rate: Time,
    /// How much health restores per second while in light.
    pub restore_rate: Time,
}
