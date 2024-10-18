use super::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryLabel {
    #[default]
    Unknown,
    FadeIn(LightId),
    FadeOut(LightId),
    Rotate(LightId, WaypointId),
    Scale(LightId, WaypointId),
    MoveLight(LightId),
    MoveWaypoint(LightId, WaypointId),
    MoveWaypointTime(LightId, WaypointId),
    // Drag,
}

impl HistoryLabel {
    pub fn should_merge(&self, other: &Self) -> bool {
        match self {
            Self::Unknown => false,
            _ => self == other,
        }
    }
}

pub struct History {
    /// State that will be saved in the undo stack.
    /// (Not every operation gets saved)
    pub buffer_state: Level,
    pub buffer_label: HistoryLabel,

    pub undo_stack: Vec<Level>,
    pub redo_stack: Vec<Level>,
}

impl History {
    pub fn new(level: &Level) -> Self {
        Self {
            buffer_state: level.clone(),
            buffer_label: HistoryLabel::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn save_state(&mut self, level: &Level, label: HistoryLabel) {
        if *level == self.buffer_state {
            // State did not change
            return;
        }

        if self.buffer_label.should_merge(&label) {
            // Merge changes
            self.buffer_state = level.clone();
            return;
        }

        self.save_force(level, label)
    }

    /// Flush all buffered changes, if there are any.
    pub fn flush(&mut self, level: &Level) {
        self.buffer_label = HistoryLabel::Unknown;
        self.buffer_state = level.clone();
    }

    /// Save the level without doing any checks.
    fn save_force(&mut self, level: &Level, label: HistoryLabel) {
        // Push the change
        self.buffer_label = label;
        let mut state = level.clone();
        std::mem::swap(&mut state, &mut self.buffer_state);

        self.undo_stack.push(state);
        // TODO: limit capacity
        self.redo_stack.clear();

        log::debug!(
            "Saved old state to the stack, starting new buffer {:?}",
            label
        );
    }
}
