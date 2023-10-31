use super::*;

use crate::prelude::*;

#[derive(Debug)]
pub struct TimelineWidget {
    pub state: WidgetState,
    /// Position of the current beat on the timeline relative to the left edge.
    pub current_beat: f32,
    pub selection: Option<RangeInclusive<f32>>,
    /// Render scale in pixels per beat.
    scale: f32,
    /// The scrolloff in beats.
    scroll: Time,
    raw_current_beat: Time,
    raw_selection: Option<RangeInclusive<Time>>,
}

impl TimelineWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::default(),
            current_beat: 0.0,
            selection: None,
            scale: 1.0,
            scroll: Time::ZERO,
            raw_current_beat: Time::ZERO,
            raw_selection: None,
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn rescale(&mut self, new_scale: f32) {
        self.scale = new_scale;
        self.reload();
    }

    pub fn auto_scale(&mut self, max_beat: Time) {
        let scale = self.state.position.width() / max_beat.as_f32();
        self.scale = scale;
        self.reload();
    }

    pub fn get_scroll(&self) -> Time {
        self.scroll
    }

    pub fn scroll(&mut self, delta: Time) {
        self.scroll += delta;
        self.reload();
    }

    pub fn update_time(&mut self, current_beat: Time) {
        self.raw_current_beat = current_beat;
        self.reload();

        // Auto scroll if current beat goes off screen
        let margin = 50.0;
        let min = margin;
        let max = self.state.position.width() - margin;
        if self.current_beat < min {
            self.scroll(r32((min - self.current_beat) / self.scale));
        } else if self.current_beat > max {
            self.scroll(r32((max - self.current_beat) / self.scale));
        }
    }

    pub fn update_selection(&mut self, selection: Option<RangeInclusive<Time>>) {
        self.raw_selection = selection;
        self.reload();
    }

    fn reload(&mut self) {
        self.current_beat = (self.raw_current_beat + self.scroll).as_f32() * self.scale;
        self.selection = self.raw_selection.clone().map(|selection| {
            let from = (*selection.start() + self.scroll).as_f32() * self.scale;
            let to = (*selection.end() + self.scroll).as_f32() * self.scale;
            from..=to
        });
    }
}

impl Widget for TimelineWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
