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
    pub timeline: TimelineConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineConfig {
    /// Whether you need to hold a button (Shift) to scroll at the slow speed
    /// (set by the timeline subdivision).
    /// When `false`, inverts the two modes, so the default is scrolling slowly,
    /// and holding the button makes it scroll by whole beats (like in osu!).
    pub hold_to_scroll_slow: bool,
    /// How many beat to scroll when using fast scroll mode (Alt).
    pub fast_speed: BeatTime,
}
