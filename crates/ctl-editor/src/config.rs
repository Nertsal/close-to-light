use super::*;

pub struct RenderOptions {
    pub hide_ui: bool,
    pub show_grid: bool,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct EditorConfig {
    pub grid: GridConfig,
    /// How much of the music to playback when scrolling (in seconds).
    pub playback_duration: FloatTime,
    pub scroll_slow: BeatTime,
    pub scroll_normal: BeatTime,
    pub scroll_fast: BeatTime,
    pub theme: EditorTheme,
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// Size of each cell in the grid.
    pub cell_size: Coord,
    /// Every n'th line of the grid is thick.
    /// If 0, then no lines are thick.
    pub thick_every: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTheme {
    pub hover: Color,
    pub select: Color,
}
