use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    pub bpm: R32,
    /// Time before the light appears in beats.
    pub telegraph_beats: Time,
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

impl Config {
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
    }

    pub fn telegraph_time(&self) -> Time {
        self.telegraph_beats * self.beat_time()
    }
}
