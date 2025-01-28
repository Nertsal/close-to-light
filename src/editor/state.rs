use super::*;

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    /// Place a new light.
    Place {
        shape: Shape,
        danger: bool,
    },
    Playing {
        start_time: Time,
        start_target_time: Time,
        old_state: Box<State>,
    },
    /// Control waypoints of an existing event.
    Waypoints {
        light_id: LightId,
        state: WaypointsState,
    },
}

#[derive(Debug, Clone)]
pub enum WaypointsState {
    Idle,
    New,
}

#[derive(Debug, Default)]
pub struct EditorLevelState {
    /// Interactable level state representing current time.
    pub static_level: Option<LevelState>,
    /// Dynamic level state showing the upcoming animations.
    pub dynamic_level: Option<LevelState>,
    /// Index of the hovered static light.
    pub hovered_light: Option<LightId>,
    pub waypoints: Option<Waypoints>,
}

#[derive(Debug)]
pub struct Waypoints {
    /// Index of the light event.
    pub light: LightId,
    pub points: Vec<Waypoint>,
    /// Index of the hovered *rendered* waypoint.
    pub hovered: Option<usize>,
    /// Index of the selected *original* keyframe.
    pub selected: Option<WaypointId>,
}

#[derive(Debug)]
pub struct Waypoint {
    /// Whether the waypoint is rendered. Used when several waypoints overlap.
    pub visible: bool,
    /// Index of the original keyframe.
    /// `None` when placing a new waypoint.
    pub original: Option<WaypointId>,
    pub control: Collider,
    pub actual: Collider,
}

impl EditorLevelState {
    pub fn relevant(&self) -> &LevelState {
        self.static_level
            .as_ref()
            .or(self.dynamic_level.as_ref())
            .expect("level editor has no displayable state")
    }

    /// Returns the index of the hovered event (if any).
    pub fn hovered_event(&self) -> Option<usize> {
        self.hovered_light.map(|id| id.event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightId {
    // pub rendered: usize,
    pub event: usize,
}
