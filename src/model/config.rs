use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    pub player: PlayerConfig,
    pub fear: FearConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearConfig {
    /// How much the fear meter restores per second while in light.
    pub restore_speed: Time,
    /// How much the character shakes from fear.
    pub shake: Coord,
}
