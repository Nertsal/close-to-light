use super::*;

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct EditorConfig {
    pub grid: GridConfig,
    /// How much of the music to playback when scrolling (in beats).
    pub playback_duration: Time,
    pub scroll_slow: Time,
    pub scroll_fast: Time,
    pub theme: EditorTheme,
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// How many cells are in the grid vertically.
    pub height: Coord,
    /// Every n'th line of the grid is thick.
    /// If 0, then no lines are thick.
    pub thick_every: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTheme {
    pub hover: Color,
    pub select: Color,
}
