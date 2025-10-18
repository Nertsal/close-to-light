use super::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryLabel {
    #[default]
    Unknown,
    Merge,

    // General events
    MoveEvent(usize),
    EventDuration(usize),

    // Vfx
    CameraShakeIntensity(usize),

    // Lights
    FadeIn(LightId),
    FadeOut(LightId),
    Rotate(LightId, WaypointId),
    Scale(LightId, WaypointId),
    MoveLight(LightId),

    // Waypoints
    MoveWaypoint(LightId, WaypointId),
    MoveWaypointTime(LightId, WaypointId),

    Drag,
}

impl HistoryLabel {
    pub fn should_merge(&self, other: &Self) -> bool {
        match self {
            Self::Unknown => false,
            Self::Merge => true,
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

    pub fn undo(&mut self, level: &mut Level) {
        if let Some(mut state) = self.undo_stack.pop() {
            std::mem::swap(&mut state, level);
            self.redo_stack.push(state);
            self.buffer_state = level.clone();
            self.buffer_label = HistoryLabel::Unknown;
            log::debug!("Change undone");
        }
    }

    pub fn redo(&mut self, level: &mut Level) {
        if let Some(mut state) = self.redo_stack.pop() {
            std::mem::swap(&mut state, level);
            self.undo_stack.push(state);
            self.buffer_state = level.clone();
            self.buffer_label = HistoryLabel::Unknown;
            log::debug!("Change redone");
        }
    }

    pub fn start_merge(&mut self, level: &Level, label: HistoryLabel) {
        if self.buffer_label.should_merge(&label) {
            self.buffer_state = level.clone();
            self.buffer_label = HistoryLabel::Merge;
            return;
        }

        if Some(&self.buffer_state) != self.undo_stack.last() {
            // Push old changes
            self.save_force(level, HistoryLabel::Merge);
        } else {
            self.buffer_label = HistoryLabel::Merge;
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
    pub fn flush(&mut self, level: &Level, label: HistoryLabel) {
        self.buffer_label = label;
        self.buffer_state = level.clone();
        log::trace!("Flushed changes as {label:?}");
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

        log::debug!("Saved old state to the stack, starting new buffer {label:?}");
    }
}
