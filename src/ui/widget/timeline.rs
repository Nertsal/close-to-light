use super::*;

use crate::prelude::*;

#[derive(Debug)]
pub struct TimelineWidget {
    context: UiContext,
    pub state: WidgetState,
    pub current_beat: WidgetState,
    /// Start of the selection.
    pub left: WidgetState,
    /// End of the selection.
    pub right: WidgetState,
    /// Replay current position.
    pub replay: WidgetState,
    /// Render scale in pixels per beat.
    scale: f32,
    /// The scrolloff in beats.
    scroll: Time,
    raw_current_beat: Time,
    raw_left: Option<Time>,
    raw_right: Option<Time>,
    raw_replay: Option<Time>,
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
            left: default(),
            right: default(),
            replay: default(),
            scale: 1.0,
            scroll: Time::ZERO,
            raw_current_beat: Time::ZERO,
            raw_left: None,
            raw_right: None,
            raw_replay: None,
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

    pub fn update_time(&mut self, current_beat: Time, replay: Option<Time>) {
        self.raw_current_beat = current_beat;
        self.raw_replay = replay;
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
        self.reload();
    }

    pub fn start_selection(&mut self) {
        self.raw_left = Some(self.raw_current_beat);
        self.reload();
    }

    /// Finishes the selection and returns the left and right boundaries in ascending order.
    pub fn end_selection(&mut self) -> (Time, Time) {
        let right = self.raw_current_beat;
        self.raw_right = Some(right);
        self.reload();

        let left = self.raw_left.unwrap_or(right);
        if left < right {
            (left, right)
        } else {
            (right, left)
        }
    }

    pub fn clear_selection(&mut self) {
        self.raw_left = None;
        self.raw_right = None;
        self.reload();
    }

    fn reload(&mut self) {
        let render_time = |time: Time| {
            let pos = (time + self.scroll).as_f32() * self.scale;
            let pos = vec2(
                self.state.position.min.x + pos,
                self.state.position.center().y,
            );
            Aabb2::point(pos).extend_symmetric(vec2(0.1, 0.5) * self.context.font_size / 2.0)
        };
        self.current_beat
            .update(render_time(self.raw_current_beat), &self.context);

        let render_option = |widget: &mut WidgetState, time: Option<Time>| match time {
            Some(time) => {
                widget.show();
                widget.update(render_time(time), &self.context);
            }
            None => widget.hide(),
        };
        render_option(&mut self.left, self.raw_left);
        render_option(&mut self.right, self.raw_right);
        render_option(&mut self.replay, self.raw_replay);
    }

    pub fn get_cursor_time(&self) -> Time {
        r32((self.context.cursor_position.x - self.state.position.min.x) / self.scale) - self.scroll
    }

    pub fn time_to_screen(&self, t: Time) -> vec2<f32> {
        let pos = (t + self.scroll).as_f32() * self.scale;
        vec2(
            self.state.position.min.x + pos,
            self.state.position.center().y,
        )
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
        self.current_beat.walk_states_mut(f);
        self.left.walk_states_mut(f);
        self.right.walk_states_mut(f);
        self.replay.walk_states_mut(f);
    }
}
