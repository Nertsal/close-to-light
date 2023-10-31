use super::*;

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    /// Place a new light.
    Place {
        shape: Shape,
        danger: bool,
    },
    /// Specify a movement path for the light.
    Movement {
        start_beat: Time,
        light: LightEvent,
        redo_stack: Vec<MoveFrame>,
    },
    Playing {
        start_beat: Time,
        old_state: Box<State>,
    },
    /// Control waypoints of an existing event.
    Waypoints {
        event: usize,
    },
}

#[derive(Default)]
pub struct EditorLevelState {
    /// Interactable level state representing current time.
    pub static_level: Option<LevelState>,
    /// Dynamic level state showing the upcoming animations.
    pub dynamic_level: Option<LevelState>,
    /// Index of the hovered static light.
    pub hovered_light: Option<usize>,
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
        self.hovered_light
            .and_then(|i| {
                self.static_level
                    .as_ref()
                    .and_then(|level| level.lights.get(i))
            })
            .and_then(|l| l.event_id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightId {
    // pub rendered: usize,
    pub event: usize,
}
