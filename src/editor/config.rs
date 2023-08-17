use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct EditorConfig {
    /// How much of the music to playback when scrolling (in beats).
    pub playback_duration: Time,
}
