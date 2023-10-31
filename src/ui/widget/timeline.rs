use super::*;

use crate::prelude::*;

#[derive(Debug)]
pub struct TimelineWidget {
    context: UiContext,
    pub state: WidgetState,
    pub current_beat: WidgetState,
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
            context: UiContext {
                font_size: 1.0,
                cursor_position: vec2::ZERO,
                cursor_down: false,
            },
            state: default(),
            current_beat: default(),
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
        let margin_beats = r32(margin / self.scale);

        let min = margin;
        let max = self.state.position.width() - margin;
        let current = self.current_beat.position.center().x - self.state.position.min.x;
        if current < min && self.raw_current_beat > margin_beats {
            self.scroll(r32((min - current) / self.scale));
        } else if current > max {
            self.scroll(r32((max - current) / self.scale));
        }
    }

    pub fn update_selection(&mut self, selection: Option<RangeInclusive<Time>>) {
        self.raw_selection = selection;
        self.reload();
    }

    fn reload(&mut self) {
        let current = (self.raw_current_beat + self.scroll).as_f32() * self.scale;
        let current = vec2(
            self.state.position.min.x + current,
            self.state.position.center().y,
        );
        self.current_beat.position =
            Aabb2::point(current).extend_symmetric(vec2(0.1, 0.5) * self.context.font_size / 2.0);

        self.selection = self.raw_selection.clone().map(|selection| {
            let from = (*selection.start() + self.scroll).as_f32() * self.scale;
            let to = (*selection.end() + self.scroll).as_f32() * self.scale;
            from..=to
        });
    }

    pub fn get_cursor_time(&self) -> Time {
        r32((self.context.cursor_position.x - self.state.position.min.x) / self.scale) - self.scroll
    }
}

impl Widget for TimelineWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
        self.context = context.clone();
        self.reload();
    }

    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        self.state.walk_states_mut(f);
    }
}
